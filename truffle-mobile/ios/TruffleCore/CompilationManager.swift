//
//  CompilationManager.swift
//  TruffleCore
//
//  Project Truffle - Compilation Manager
//  SPEC v2.0 Section 2.2: Compilation Pipeline
//
//  Manages the screenshot-to-wiki compilation process:
//  - Queue management with priorities
//  - Checkpoint/resume for 30s limit
//  - Gemma 4 integration (via Rust bridge)
//

import Foundation
import BackgroundTasks

// MARK: - Compilation Manager
class CompilationManager {
    
    // MARK: - Singleton
    static let shared = CompilationManager()
    
    // MARK: - Dependencies
    private let database = CompilationDatabase.shared
    private let checkpointManager = CheckpointManager.shared
    private let gemmaBridge = GemmaBridge.shared
    private let notificationCenter = NotificationCenter.default
    
    // MARK: - Constants
    private let maxConcurrentCompilations = 1
    private let compilationTimeout: TimeInterval = 30.0
    private let checkpointInterval: TimeInterval = 10.0
    
    // MARK: - State
    private var compilationQueue: [CompilationQueueItem] = []
    private var isProcessing = false
    private var currentCompilation: CompilationQueueItem?
    private var backgroundTask: UIBackgroundTaskIdentifier = .invalid
    
    // MARK: - Types
    struct CompilationQueueItem: Identifiable, Codable {
        let id: UUID
        let rawUUID: UUID
        var status: CompilationStatus
        var retryCount: Int
        let priority: CompilationPriority
        let createdAt: Date
        var startedAt: Date?
        var completedAt: Date?
        var errorMessage: String?
        var checkpoint: CompilationCheckpoint?
    }
    
    enum CompilationStatus: String, Codable {
        case pending
        case processing
        case completed
        case failed
        case quarantined
        case paused
    }
    
    enum CompilationPriority: Int, Codable, Comparable {
        case p0 = 0 // User-initiated "Compile Now"
        case p1 = 1 // Share extension capture
        case p2 = 2 // Background batch
        
        static func < (lhs: CompilationPriority, rhs: CompilationPriority) -> Bool {
            return lhs.rawValue < rhs.rawValue
        }
    }
    
    struct CompilationCheckpoint: Codable {
        let stage: CompilationStage
        let progress: Double
        let partialResult: Data?
        let timestamp: Date
    }
    
    enum CompilationStage: String, Codable {
        case preprocessing
        case ocr
        case gemmaInference
        case entityResolution
        case linkInjection
        case vectorIndexing
        case persistence
    }
    
    enum CompilationError: Error {
        case timeout
        case gemmaFailure
        case invalidImage
        case storageFull
        case maxRetriesExceeded
        case checkpointCorrupted
        
        var localizedDescription: String {
            switch self {
            case .timeout:
                return "Compilation timed out. Will resume in background."
            case .gemmaFailure:
                return "AI processing failed. Image may be corrupted."
            case .invalidImage:
                return "Invalid image format."
            case .storageFull:
                return "Storage full. Please free up space."
            case .maxRetriesExceeded:
                return "Max retries exceeded. Image quarantined."
            case .checkpointCorrupted:
                return "Checkpoint corrupted. Restarting compilation."
            }
        }
    }
    
    // MARK: - Initialization
    private init() {
        loadPendingCompilations()
        registerBackgroundTasks()
    }
    
    // MARK: - Public API
    
    /// Add item to compilation queue
    func enqueue(item: CompilationQueueItem) {
        DispatchQueue.main.async { [weak self] in
            self?.compilationQueue.append(item)
            self?.sortQueue()
            self?.saveQueue()
            
            // Start processing if not already running
            if !(self?.isProcessing ?? true) {
                self?.processQueue()
            }
            
            // Post notification
            self?.notificationCenter.post(
                name: .compilationQueued,
                object: nil,
                userInfo: ["itemId": item.id]
            )
        }
    }
    
    /// Schedule background compilation processing
    func scheduleBackgroundProcessing() {
        let request = BGProcessingTaskRequest(identifier: "com.truffle.compilation.background")
        request.requiresNetworkConnectivity = false
        request.requiresExternalPower = true // Only run when charging
        
        do {
            try BGTaskScheduler.shared.submit(request)
            print("[CompilationManager] Background compilation scheduled")
        } catch {
            print("[CompilationManager] Failed to schedule background compilation: \(error)")
        }
    }
    
    /// Get queue status
    func getQueueStatus() -> QueueStatus {
        let pending = compilationQueue.filter { $0.status == .pending }.count
        let processing = compilationQueue.filter { $0.status == .processing }.count
        let completed = compilationQueue.filter { $0.status == .completed }.count
        let failed = compilationQueue.filter { $0.status == .failed }.count
        
        return QueueStatus(
            pending: pending,
            processing: processing,
            completed: completed,
            failed: failed,
            total: compilationQueue.count
        )
    }
    
    /// Retry failed compilation
    func retryCompilation(itemId: UUID) {
        guard let index = compilationQueue.firstIndex(where: { $0.id == itemId }) else {
            return
        }
        
        compilationQueue[index].status = .pending
        compilationQueue[index].retryCount = 0
        compilationQueue[index].errorMessage = nil
        
        sortQueue()
        processQueue()
    }
    
    /// Cancel compilation
    func cancelCompilation(itemId: UUID) {
        guard let index = compilationQueue.firstIndex(where: { $0.id == itemId }) else {
            return
        }
        
        if compilationQueue[index].status == .processing {
            // Save checkpoint before canceling
            saveCheckpoint(for: compilationQueue[index])
        }
        
        compilationQueue.remove(at: index)
        saveQueue()
    }
    
    /// Clear completed compilations
    func clearCompleted() {
        compilationQueue.removeAll { $0.status == .completed }
        saveQueue()
    }
    
    // MARK: - Private Implementation
    
    private func processQueue() {
        guard !isProcessing else { return }
        
        // Get next pending item
        guard let nextItem = compilationQueue.first(where: { $0.status == .pending }) else {
            isProcessing = false
            return
        }
        
        isProcessing = true
        currentCompilation = nextItem
        
        // Update status
        if let index = compilationQueue.firstIndex(where: { $0.id == nextItem.id }) {
            compilationQueue[index].status = .processing
            compilationQueue[index].startedAt = Date()
        }
        
        // Begin background task
        backgroundTask = UIApplication.shared.beginBackgroundTask { [weak self] in
            self?.handleBackgroundExpiration()
        }
        
        // Check for checkpoint
        if let checkpoint = nextItem.checkpoint {
            resumeFromCheckpoint(checkpoint, item: nextItem)
        } else {
            compile(item: nextItem)
        }
    }
    
    private func compile(item: CompilationQueueItem) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }
            
            do {
                // Step 1: Load raw image
                guard let rawImage = self.loadRawImage(uuid: item.rawUUID) else {
                    throw CompilationError.invalidImage
                }
                
                // Step 2: Preprocess
                self.updateCheckpoint(item: item, stage: .preprocessing, progress: 0.1)
                let preprocessed = try self.preprocess(image: rawImage)
                
                // Step 3: OCR (optional - for text extraction)
                self.updateCheckpoint(item: item, stage: .ocr, progress: 0.2)
                let ocrText = self.performOCR(on: preprocessed)
                
                // Step 4: Gemma 4 inference
                self.updateCheckpoint(item: item, stage: .gemmaInference, progress: 0.3)
                let wikiNode = try self.gemmaBridge.compile(
                    image: preprocessed,
                    ocrText: ocrText,
                    timeout: self.compilationTimeout
                )
                
                // Step 5: Entity resolution
                self.updateCheckpoint(item: item, stage: .entityResolution, progress: 0.6)
                let resolvedNode = try self.resolveEntities(wikiNode)
                
                // Step 6: Link injection
                self.updateCheckpoint(item: item, stage: .linkInjection, progress: 0.7)
                try self.injectLinks(resolvedNode)
                
                // Step 7: Vector indexing
                self.updateCheckpoint(item: item, stage: .vectorIndexing, progress: 0.8)
                try self.indexVectors(resolvedNode)
                
                // Step 8: Persistence
                self.updateCheckpoint(item: item, stage: .persistence, progress: 0.9)
                try self.saveWikiNode(resolvedNode)
                
                // Success
                self.handleCompilationSuccess(item: item, node: resolvedNode)
                
            } catch {
                self.handleCompilationError(item: item, error: error)
            }
        }
    }
    
    private func resumeFromCheckpoint(_ checkpoint: CompilationCheckpoint, item: CompilationQueueItem) {
        print("[CompilationManager] Resuming from checkpoint: \(checkpoint.stage)")
        
        // Resume from saved stage
        switch checkpoint.stage {
        case .preprocessing, .ocr:
            // Restart from beginning
            compile(item: item)
            
        case .gemmaInference:
            // Resume Gemma inference
            compile(item: item) // Simplified - would need partial result handling
            
        default:
            // Continue from current stage
            compile(item: item)
        }
    }
    
    private func preprocess(image: UIImage) throws -> UIImage {
        // Resize to max 896px dimension
        let maxDimension: CGFloat = 896.0
        let size = image.size
        
        var newSize: CGSize
        if size.width > size.height {
            let ratio = maxDimension / size.width
            newSize = CGSize(width: maxDimension, height: size.height * ratio)
        } else {
            let ratio = maxDimension / size.height
            newSize = CGSize(width: size.width * ratio, height: maxDimension)
        }
        
        UIGraphicsBeginImageContextWithOptions(newSize, false, 1.0)
        image.draw(in: CGRect(origin: .zero, size: newSize))
        let resized = UIGraphicsGetImageFromCurrentImageContext()
        UIGraphicsEndImageContext()
        
        guard let result = resized else {
            throw CompilationError.invalidImage
        }
        
        return result
    }
    
    private func performOCR(on image: UIImage) -> String? {
        // Use Vision framework for OCR
        // Return extracted text or nil if OCR fails
        return nil // Placeholder
    }
    
    private func resolveEntities(_ node: WikiNode) throws -> WikiNode {
        // Fuzzy match against existing entities
        // Merge if similar entity exists
        return node
    }
    
    private func injectLinks(_ node: WikiNode) throws {
        // Update backlinks in referenced nodes
        // ACID transaction
    }
    
    private func indexVectors(_ node: WikiNode) throws {
        // Generate embedding for node
        // Insert into HNSW index
    }
    
    private func saveWikiNode(_ node: WikiNode) throws {
        // Save to SQLite
        // Trigger sync
    }
    
    private func loadRawImage(uuid: UUID) -> UIImage? {
        // Load from raw storage
        return RawStorageManager.shared.loadImage(uuid: uuid)
    }
    
    private func updateCheckpoint(item: CompilationQueueItem, stage: CompilationStage, progress: Double) {
        guard let index = compilationQueue.firstIndex(where: { $0.id == item.id }) else {
            return
        }
        
        let checkpoint = CompilationCheckpoint(
            stage: stage,
            progress: progress,
            partialResult: nil,
            timestamp: Date()
        )
        
        compilationQueue[index].checkpoint = checkpoint
        saveQueue()
        
        // Post progress notification
        notificationCenter.post(
            name: .compilationProgress,
            object: nil,
            userInfo: [
                "itemId": item.id,
                "stage": stage.rawValue,
                "progress": progress
            ]
        )
    }
    
    private func saveCheckpoint(for item: CompilationQueueItem) {
        guard let checkpoint = item.checkpoint else { return }
        checkpointManager.saveCompilationCheckpoint(checkpoint, for: item.id)
    }
    
    private func handleCompilationSuccess(item: CompilationQueueItem, node: WikiNode) {
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            
            if let index = self.compilationQueue.firstIndex(where: { $0.id == item.id }) {
                self.compilationQueue[index].status = .completed
                self.compilationQueue[index].completedAt = Date()
                self.compilationQueue[index].checkpoint = nil
            }
            
            self.saveQueue()
            self.isProcessing = false
            UIApplication.shared.endBackgroundTask(self.backgroundTask)
            
            // Post success notification
            self.notificationCenter.post(
                name: .compilationCompleted,
                object: nil,
                userInfo: ["itemId": item.id, "nodeId": node.id]
            )
            
            // Process next item
            self.processQueue()
        }
    }
    
    private func handleCompilationError(item: CompilationQueueItem, error: Error) {
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            
            let shouldRetry: Bool
            let newStatus: CompilationStatus
            
            if let compilationError = error as? CompilationError {
                switch compilationError {
                case .timeout:
                    // Save checkpoint and retry
                    self.saveCheckpoint(for: item)
                    shouldRetry = true
                    newStatus = .paused
                    
                case .maxRetriesExceeded:
                    shouldRetry = false
                    newStatus = .quarantined
                    
                default:
                    shouldRetry = item.retryCount < 3
                    newStatus = shouldRetry ? .pending : .failed
                }
            } else {
                shouldRetry = item.retryCount < 3
                newStatus = shouldRetry ? .pending : .failed
            }
            
            if let index = self.compilationQueue.firstIndex(where: { $0.id == item.id }) {
                self.compilationQueue[index].status = newStatus
                self.compilationQueue[index].errorMessage = error.localizedDescription
                
                if shouldRetry {
                    self.compilationQueue[index].retryCount += 1
                }
            }
            
            self.saveQueue()
            self.isProcessing = false
            UIApplication.shared.endBackgroundTask(self.backgroundTask)
            
            // Post error notification
            self.notificationCenter.post(
                name: .compilationFailed,
                object: nil,
                userInfo: [
                    "itemId": item.id,
                    "error": error.localizedDescription
                ]
            )
            
            // Process next item or retry
            if shouldRetry {
                DispatchQueue.main.asyncAfter(deadline: .now() + 5) { [weak self] in
                    self?.processQueue()
                }
            } else {
                self.processQueue()
            }
        }
    }
    
    private func handleBackgroundExpiration() {
        // Save checkpoint for current compilation
        if let item = currentCompilation {
            saveCheckpoint(for: item)
            
            if let index = compilationQueue.firstIndex(where: { $0.id == item.id }) {
                compilationQueue[index].status = .paused
            }
            
            saveQueue()
        }
        
        UIApplication.shared.endBackgroundTask(backgroundTask)
    }
    
    private func sortQueue() {
        compilationQueue.sort { a, b in
            // Sort by priority first, then by creation date
            if a.priority != b.priority {
                return a.priority < b.priority
            }
            return a.createdAt < b.createdAt
        }
    }
    
    private func saveQueue() {
        // Persist queue to database
        database.saveQueue(compilationQueue)
    }
    
    private func loadPendingCompilations() {
        // Load pending compilations from database
        compilationQueue = database.loadQueue()
        sortQueue()
    }
    
    private func registerBackgroundTasks() {
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: "com.truffle.compilation.background",
            using: nil
        ) { [weak self] task in
            self?.handleBackgroundCompilationTask(task as! BGProcessingTask)
        }
    }
    
    private func handleBackgroundCompilationTask(_ task: BGProcessingTask) {
        task.expirationHandler = { [weak self] in
            self?.handleBackgroundExpiration()
        }
        
        // Process queue
        processQueue()
        
        // Complete task when done
        DispatchQueue.main.asyncAfter(deadline: .now() + 25) { [weak self] in
            let status = self?.getQueueStatus()
            let success = status?.pending == 0 && status?.processing == 0
            task.setTaskCompleted(success: success)
        }
    }
}

// MARK: - Supporting Types

struct QueueStatus {
    let pending: Int
    let processing: Int
    let completed: Int
    let failed: Int
    let total: Int
}

struct WikiNode {
    let id: UUID
    let title: String
    let content: String
}

// MARK: - Notification Names

extension Notification.Name {
    static let compilationQueued = Notification.Name("compilationQueued")
    static let compilationStarted = Notification.Name("compilationStarted")
    static let compilationProgress = Notification.Name("compilationProgress")
    static let compilationCompleted = Notification.Name("compilationCompleted")
    static let compilationFailed = Notification.Name("compilationFailed")
}

// MARK: - Placeholder Classes

class CompilationDatabase {
    static let shared = CompilationDatabase()
    func saveQueue(_ queue: [CompilationManager.CompilationQueueItem]) {}
    func loadQueue() -> [CompilationManager.CompilationQueueItem] { return [] }
}

class GemmaBridge {
    static let shared = GemmaBridge()
    func compile(image: UIImage, ocrText: String?, timeout: TimeInterval) throws -> WikiNode {
        return WikiNode(id: UUID(), title: "", content: "")
    }
}

class CheckpointManager {
    static let shared = CheckpointManager()
    func saveCompilationCheckpoint(_ checkpoint: CompilationManager.CompilationCheckpoint, for itemId: UUID) {}
}
