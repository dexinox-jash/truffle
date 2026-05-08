//
//  ShareViewController.swift
//  TruffleShareExtension
//
//  Project Truffle - iOS Share Extension
//  SPEC v2.0 Section 5.3: Mobile Companion
//
//  Handles screenshot capture from iOS Share Sheet
//  - Saves to raw storage
//  - Triggers background compilation
//  - Respects 30s background execution limit with checkpoint/resume
//

import UIKit
import Social
import MobileCoreServices
import UniformTypeIdentifiers

// MARK: - Share View Controller
class ShareViewController: SLComposeServiceViewController {
    
    // MARK: - Dependencies
    private let storageManager = RawStorageManager.shared
    private let compilationManager = CompilationManager.shared
    private let checkpointManager = CheckpointManager.shared
    
    // MARK: - State
    private var capturedImage: UIImage?
    private var processingUUID: UUID?
    private var isProcessing = false
    
    // MARK: - Constants
    private let maxBackgroundExecutionTime: TimeInterval = 25.0 // 30s limit - 5s buffer
    private let storageWarningThreshold: UInt64 = 5 * 1024 * 1024 * 1024 // 5GB
    private let storagePauseThreshold: UInt64 = 10 * 1024 * 1024 * 1024 // 10GB
    
    // MARK: - Lifecycle
    override func viewDidLoad() {
        super.viewDidLoad()
        setupUI()
        checkStorageLimits()
    }
    
    // MARK: - UI Configuration
    private func setupUI() {
        placeholder = "Add a note (optional)..."
        
        // Darkroom design system colors
        view.backgroundColor = UIColor(red: 0.039, green: 0.039, blue: 0.039, alpha: 1.0) // #0A0A0A
        
        // Set navigation title
        navigationController?.navigationBar.titleTextAttributes = [
            .foregroundColor: UIColor(red: 0.898, green: 0.898, blue: 0.898, alpha: 1.0) // #E5E5E5
        ]
    }
    
    // MARK: - Content Validation
    override func isContentValid() -> Bool {
        // Allow post even without text - the image is what matters
        return capturedImage != nil || isLoadingImage
    }
    
    private var isLoadingImage = true
    
    // MARK: - Post Action
    override func didSelectPost() {
        guard let image = capturedImage else {
            showError(message: "No image captured")
            return
        }
        
        // Check storage limits before processing
        let currentUsage = storageManager.getStorageUsage()
        if currentUsage >= storagePauseThreshold {
            showStorageExceededAlert()
            return
        }
        
        // Process the screenshot
        processScreenshot(image)
    }
    
    // MARK: - Configuration Items
    override func configurationItems() -> [Any]! {
        // No additional configuration needed for basic capture
        // Future: Could add tags, privacy classification
        return []
    }
    
    // MARK: - Image Extraction
    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        extractImageFromExtensionContext()
    }
    
    private func extractImageFromExtensionContext() {
        guard let extensionItems = extensionContext?.inputItems as? [NSExtensionItem] else {
            isLoadingImage = false
            return
        }
        
        for item in extensionItems {
            guard let attachments = item.attachments else { continue }
            
            for attachment in attachments {
                // Check if attachment is an image
                if attachment.hasItemConformingToTypeIdentifier(UTType.image.identifier) {
                    loadImage(from: attachment)
                    return
                }
            }
        }
        
        isLoadingImage = false
    }
    
    private func loadImage(from attachment: NSItemProvider) {
        attachment.loadItem(forTypeIdentifier: UTType.image.identifier, options: nil) { [weak self] (item, error) in
            DispatchQueue.main.async {
                self?.isLoadingImage = false
                
                if let error = error {
                    self?.showError(message: "Failed to load image: \(error.localizedDescription)")
                    return
                }
                
                var image: UIImage?
                
                if let imageURL = item as? URL {
                    image = UIImage(contentsOfFile: imageURL.path)
                } else if let imageData = item as? Data {
                    image = UIImage(data: imageData)
                } else if let uiImage = item as? UIImage {
                    image = uiImage
                }
                
                guard let finalImage = image else {
                    self?.showError(message: "Could not process image")
                    return
                }
                
                self?.capturedImage = finalImage
                self?.validateContent()
            }
        }
    }
    
    // MARK: - Screenshot Processing
    func processScreenshot(_ image: UIImage) {
        isProcessing = true
        
        // Start background task for processing
        var backgroundTask: UIBackgroundTaskIdentifier = .invalid
        backgroundTask = UIApplication.shared.beginBackgroundTask { [weak self] in
            // Cleanup on expiration
            self?.checkpointManager.saveCheckpoint()
            UIApplication.shared.endBackgroundTask(backgroundTask)
        }
        
        // Set up timeout handler
        let timeoutWorkItem = DispatchWorkItem { [weak self] in
            self?.handleProcessingTimeout()
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + maxBackgroundExecutionTime, execute: timeoutWorkItem)
        
        // Process on background queue
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }
            
            do {
                // Step 1: Convert to optimized data
                guard let imageData = self.optimizeImage(image) else {
                    throw ProcessingError.imageOptimizationFailed
                }
                
                // Check for checkpoint (resume scenario)
                if let checkpoint = self.checkpointManager.loadCheckpoint() {
                    self.resumeFromCheckpoint(checkpoint)
                    return
                }
                
                // Step 2: Save to raw storage
                let uuid = self.saveToRawStorage(imageData, originalImage: image)
                self.processingUUID = uuid
                
                // Step 3: Trigger background compilation
                self.triggerCompilation(uuid)
                
                // Success - complete the extension
                DispatchQueue.main.async {
                    timeoutWorkItem.cancel()
                    self.extensionContext?.completeRequest(returningItems: nil)
                }
                
            } catch {
                DispatchQueue.main.async {
                    timeoutWorkItem.cancel()
                    self.handleProcessingError(error)
                }
            }
            
            UIApplication.shared.endBackgroundTask(backgroundTask)
        }
    }
    
    // MARK: - Image Optimization
    private func optimizeImage(_ image: UIImage) -> Data? {
        // Resize to max dimension 896px (as per spec 2.2.2)
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
        
        // Render resized image
        UIGraphicsBeginImageContextWithOptions(newSize, false, 1.0)
        image.draw(in: CGRect(origin: .zero, size: newSize))
        let resizedImage = UIGraphicsGetImageFromCurrentImageContext()
        UIGraphicsEndImageContext()
        
        // Compress as high-quality JPEG
        return resizedImage?.jpegData(compressionQuality: 0.9)
    }
    
    // MARK: - Raw Storage
    func saveToRawStorage(_ imageData: Data, originalImage: UIImage) -> UUID {
        // Generate content-addressable UUID (SHA256 of image data)
        let uuid = generateContentUUID(for: imageData)
        
        // Create raw artifact metadata
        let artifact = RawArtifact(
            uuid: uuid,
            filename: "screenshot_\(Date().timeIntervalSince1970).jpg",
            binary: imageData,
            capturedAt: ISO8601DateFormatter().string(from: Date()),
            deviceId: getDeviceFingerprint(),
            appContext: extractAppContext(),
            geohash: nil, // Privacy: only 4-char geohash if location enabled
            metadata: RawMetadata(
                sizeBytes: UInt64(imageData.count),
                width: Int(originalImage.size.width),
                height: Int(originalImage.size.height),
                ocrText: nil,
                embeddingVector: nil
            )
        )
        
        // Save to raw storage
        storageManager.saveArtifact(artifact)
        
        return uuid
    }
    
    private func generateContentUUID(for data: Data) -> UUID {
        // Use first 16 bytes of SHA256 as UUID
        var digest = [UInt8](repeating: 0, count: Int(CC_SHA256_DIGEST_LENGTH))
        data.withUnsafeBytes { bytes in
            _ = CC_SHA256(bytes.baseAddress, CC_LONG(data.count), &digest)
        }
        
        let uuidBytes = Array(digest[0..<16])
        return UUID(uuid: (uuidBytes[0], uuidBytes[1], uuidBytes[2], uuidBytes[3],
                          uuidBytes[4], uuidBytes[5], uuidBytes[6], uuidBytes[7],
                          uuidBytes[8], uuidBytes[9], uuidBytes[10], uuidBytes[11],
                          uuidBytes[12], uuidBytes[13], uuidBytes[14], uuidBytes[15]))
    }
    
    private func getDeviceFingerprint() -> String {
        // Privacy-preserving device hash
        let deviceID = UIDevice.current.identifierForVendor?.uuidString ?? "unknown"
        let salt = "truffle_device_salt_v1" // Constant salt for consistency
        let combined = "\(deviceID)\(salt)"
        
        var digest = [UInt8](repeating: 0, count: Int(CC_SHA256_DIGEST_LENGTH))
        combined.data(using: .utf8)?.withUnsafeBytes { bytes in
            _ = CC_SHA256(bytes.baseAddress, CC_LONG(combined.utf8.count), &digest)
        }
        
        return Data(digest).base64EncodedString()
    }
    
    private func extractAppContext() -> AppContext? {
        // Try to get source app info from extension context
        guard let extensionContext = extensionContext else { return nil }
        
        // The source app bundle ID is available in iOS 15+
        if #available(iOS 15.0, *) {
            // Note: iOS doesn't always expose the source app name to extensions
            return AppContext(
                bundleId: extensionContext.hostBundleIdentifier ?? "unknown",
                appName: "Shared from App",
                windowTitle: nil
            )
        }
        
        return nil
    }
    
    // MARK: - Compilation Trigger
    func triggerCompilation(_ uuid: UUID) {
        // Add to compilation queue with P1 priority (immediate)
        let queueItem = CompilationQueueItem(
            id: UUID(),
            rawUUID: uuid,
            status: .pending,
            retryCount: 0,
            priority: .p1, // User-initiated
            createdAt: Date()
        )
        
        compilationManager.enqueue(item: queueItem)
        
        // Request background processing time
        scheduleBackgroundCompilation()
    }
    
    private func scheduleBackgroundCompilation() {
        // Use BGTaskScheduler for background compilation
        // This will run when device is charging and on WiFi
        compilationManager.scheduleBackgroundProcessing()
    }
    
    // MARK: - Checkpoint/Resume (30s limit handling)
    private func handleProcessingTimeout() {
        // Save checkpoint for resume
        if let uuid = processingUUID {
            checkpointManager.createCheckpoint(
                uuid: uuid,
                stage: .saving,
                progress: 0.5
            )
        }
        
        // Complete the extension - processing will resume in background
        extensionContext?.completeRequest(returningItems: nil)
    }
    
    private func resumeFromCheckpoint(_ checkpoint: ProcessingCheckpoint) {
        // Resume processing from saved checkpoint
        switch checkpoint.stage {
        case .saving:
            // Re-save to raw storage
            if let image = capturedImage,
               let imageData = optimizeImage(image) {
                let uuid = saveToRawStorage(imageData, originalImage: image)
                triggerCompilation(uuid)
            }
        case .compiling:
            // Just trigger compilation
            triggerCompilation(checkpoint.uuid)
        default:
            break
        }
        
        // Clear checkpoint after resume
        checkpointManager.clearCheckpoint()
    }
    
    // MARK: - Storage Management
    private func checkStorageLimits() {
        let usage = storageManager.getStorageUsage()
        
        if usage >= storagePauseThreshold {
            showStorageWarning(isCritical: true)
        } else if usage >= storageWarningThreshold {
            showStorageWarning(isCritical: false)
        }
    }
    
    private func showStorageWarning(isCritical: Bool) {
        let title = isCritical ? "Storage Full" : "Storage Warning"
        let message = isCritical
            ? "You've reached 10GB of storage. Please export or delete screenshots before capturing more."
            : "You've used 5GB of storage. Consider exporting screenshots to free up space."
        
        let alert = UIAlertController(title: title, message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "OK", style: .default))
        
        if !isCritical {
            alert.addAction(UIAlertAction(title: "Export Now", style: .default) { [weak self] _ in
                self?.openMainAppForExport()
            })
        }
        
        present(alert, animated: true)
    }
    
    private func showStorageExceededAlert() {
        let alert = UIAlertController(
            title: "Storage Limit Reached",
            message: "Please open the Truffle app to export or delete screenshots before capturing more.",
            preferredStyle: .alert
        )
        alert.addAction(UIAlertAction(title: "Open Truffle", style: .default) { [weak self] _ in
            self?.openMainApp()
        })
        alert.addAction(UIAlertAction(title: "Cancel", style: .cancel) { [weak self] _ in
            self?.extensionContext?.cancelRequest(withError: NSError(domain: "Truffle", code: -1))
        })
        present(alert, animated: true)
    }
    
    // MARK: - Error Handling
    private func handleProcessingError(_ error: Error) {
        let message: String
        if let processingError = error as? ProcessingError {
            message = processingError.localizedDescription
        } else {
            message = "An unexpected error occurred"
        }
        
        showError(message: message)
    }
    
    private func showError(message: String) {
        let alert = UIAlertController(title: "Error", message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "OK", style: .default) { [weak self] _ in
            self?.extensionContext?.cancelRequest(withError: NSError(domain: "Truffle", code: -1))
        })
        present(alert, animated: true)
    }
    
    // MARK: - Deep Linking to Main App
    private func openMainApp() {
        guard let url = URL(string: "truffle://open") else { return }
        
        // Open the main app
        _ = extensionContext?.open(url, completionHandler: nil)
    }
    
    private func openMainAppForExport() {
        guard let url = URL(string: "truffle://export") else { return }
        _ = extensionContext?.open(url, completionHandler: nil)
    }
}

// MARK: - Supporting Types

enum ProcessingError: Error {
    case imageOptimizationFailed
    case storageFailed
    case compilationFailed
    
    var localizedDescription: String {
        switch self {
        case .imageOptimizationFailed:
            return "Failed to process the image"
        case .storageFailed:
            return "Failed to save the screenshot"
        case .compilationFailed:
            return "Failed to start compilation"
        }
    }
}

struct RawArtifact {
    let uuid: UUID
    let filename: String
    let binary: Data
    let capturedAt: String
    let deviceId: String
    let appContext: AppContext?
    let geohash: String?
    let metadata: RawMetadata
}

struct RawMetadata {
    let sizeBytes: UInt64
    let width: Int
    let height: Int
    let ocrText: String?
    let embeddingVector: [Float]?
}

struct AppContext {
    let bundleId: String
    let appName: String
    let windowTitle: String?
}

struct CompilationQueueItem {
    let id: UUID
    let rawUUID: UUID
    var status: CompilationStatus
    var retryCount: Int
    let priority: CompilationPriority
    let createdAt: Date
}

enum CompilationStatus {
    case pending
    case processing
    case completed
    case failed
    case quarantined
}

enum CompilationPriority: Int {
    case p0 = 0 // User-initiated "Compile Now"
    case p1 = 1 // Share extension capture
    case p2 = 2 // Background batch
}

struct ProcessingCheckpoint {
    let uuid: UUID
    let stage: ProcessingStage
    let progress: Double
}

enum ProcessingStage {
    case extracting
    case saving
    case compiling
}

// MARK: - SHA256 Helper
import CommonCrypto

// MARK: - Manager Protocols (Implemented in separate files)

protocol RawStorageManagerProtocol {
    func saveArtifact(_ artifact: RawArtifact)
    func getStorageUsage() -> UInt64
}

protocol CompilationManagerProtocol {
    func enqueue(item: CompilationQueueItem)
    func scheduleBackgroundProcessing()
}

protocol CheckpointManagerProtocol {
    func createCheckpoint(uuid: UUID, stage: ProcessingStage, progress: Double)
    func loadCheckpoint() -> ProcessingCheckpoint?
    func saveCheckpoint()
    func clearCheckpoint()
}

// Placeholder implementations - full implementations in separate files
class RawStorageManager: RawStorageManagerProtocol {
    static let shared = RawStorageManager()
    func saveArtifact(_ artifact: RawArtifact) {}
    func getStorageUsage() -> UInt64 { return 0 }
}

class CompilationManager: CompilationManagerProtocol {
    static let shared = CompilationManager()
    func enqueue(item: CompilationQueueItem) {}
    func scheduleBackgroundProcessing() {}
}

class CheckpointManager: CheckpointManagerProtocol {
    static let shared = CheckpointManager()
    func createCheckpoint(uuid: UUID, stage: ProcessingStage, progress: Double) {}
    func loadCheckpoint() -> ProcessingCheckpoint? { return nil }
    func saveCheckpoint() {}
    func clearCheckpoint() {}
}
