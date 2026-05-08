//
//  SyncManager.swift
//  TruffleCore
//
//  Project Truffle - Background Sync Manager
//  SPEC v2.0 Section 5.3.2: Technical Constraints
//
//  Handles background sync with:
//  - BGTaskScheduler for background fetch (every 15 min when charging)
//  - Checkpoint/resume for 30s execution limit
//  - CRDT delta sync via Zero-Knowledge protocol
//

import Foundation
import BackgroundTasks

// MARK: - Sync Manager
class SyncManager {
    
    // MARK: - Singleton
    static let shared = SyncManager()
    
    // MARK: - Dependencies
    private let keychainManager = KeychainManager.shared
    private let checkpointManager = SyncCheckpointManager.shared
    private let networkManager = NetworkManager.shared
    private let crdtManager = CRDTManager.shared
    
    // MARK: - Constants
    private let backgroundFetchInterval: TimeInterval = 15 * 60 // 15 minutes
    private let maxExecutionTime: TimeInterval = 25.0 // 30s limit - 5s buffer
    private let syncTaskIdentifier = "com.truffle.sync.background"
    private let chargingCheckInterval: TimeInterval = 60.0 // Check charging status every minute
    
    // MARK: - State
    private var isSyncing = false
    private var lastSyncTime: Date?
    private var syncQueue: [SyncOperation] = []
    private var backgroundTask: UIBackgroundTaskIdentifier = .invalid
    
    // MARK: - Types
    struct SyncOperation {
        let id: UUID
        let type: OperationType
        let priority: OperationPriority
        let data: Data
        let timestamp: Date
        
        enum OperationType {
            case crdtDelta
            case blobUpload
            case blobDownload
            case metadataSync
        }
        
        enum OperationPriority: Int {
            case critical = 0
            case high = 1
            case normal = 2
            case low = 3
        }
    }
    
    struct SyncCheckpoint {
        let operationId: UUID
        let bytesTransferred: Int64
        let totalBytes: Int64
        let timestamp: Date
    }
    
    enum SyncError: Error {
        case networkUnavailable
        case authenticationFailed
        case quotaExceeded
        case executionTimeout
        case crdtConflict
        case decryptionFailed
        
        var localizedDescription: String {
            switch self {
            case .networkUnavailable:
                return "Network unavailable. Will retry when connection restored."
            case .authenticationFailed:
                return "Authentication failed. Please re-pair your devices."
            case .quotaExceeded:
                return "Sync quota exceeded. Upgrade your plan or wait for reset."
            case .executionTimeout:
                return "Sync timed out. Will resume in next background fetch."
            case .crdtConflict:
                return "Sync conflict detected. Resolving automatically."
            case .decryptionFailed:
                return "Failed to decrypt sync data. Keys may be corrupted."
            }
        }
    }
    
    // MARK: - Initialization
    private init() {
        registerBackgroundTasks()
        setupNotifications()
    }
    
    // MARK: - Background Task Registration
    private func registerBackgroundTasks() {
        // Register background fetch task
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: syncTaskIdentifier,
            using: nil
        ) { [weak self] task in
            self?.handleBackgroundSync(task: task as! BGAppRefreshTask)
        }
    }
    
    private func setupNotifications() {
        // Listen for charging state changes
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handlePowerStateChange),
            name: NSNotification.Name.NSProcessInfoPowerStateDidChange,
            object: nil
        )
    }
    
    // MARK: - Public API
    
    /// Schedule background sync to run periodically
    func scheduleBackgroundSync() {
        let request = BGAppRefreshTaskRequest(identifier: syncTaskIdentifier)
        request.earliestBeginDate = Date(timeIntervalSinceNow: backgroundFetchInterval)
        
        do {
            try BGTaskScheduler.shared.submit(request)
            print("[SyncManager] Background sync scheduled")
        } catch {
            print("[SyncManager] Failed to schedule background sync: \(error)")
        }
    }
    
    /// Sync specifically when device is charging (optimal for large transfers)
    func syncWhenCharging() {
        guard isDeviceCharging() else {
            print("[SyncManager] Device not charging, deferring sync")
            return
        }
        
        guard isWiFiConnected() else {
            print("[SyncManager] WiFi not connected, deferring sync")
            return
        }
        
        // Start sync with extended background time
        beginBackgroundSync()
    }
    
    /// Handle sync conflicts using CRDT merge
    func handleSyncConflict(_ conflict: SyncConflict) {
        print("[SyncManager] Handling sync conflict: \(conflict)")
        
        // CRDT automatic merge - Yjs handles this
        let mergedState = crdtManager.mergeConflicts(conflict.localState, conflict.remoteState)
        
        // Save merged state
        crdtManager.applyMergedState(mergedState)
        
        // Log conflict resolution for debugging
        logConflictResolution(conflict, mergedState: mergedState)
    }
    
    /// Force immediate sync (user-initiated)
    func forceSync(completion: @escaping (Result<SyncResult, SyncError>) -> Void) {
        guard !isSyncing else {
            completion(.failure(.executionTimeout))
            return
        }
        
        isSyncing = true
        
        // Check for checkpoint resume
        if let checkpoint = checkpointManager.loadCheckpoint() {
            resumeSyncFromCheckpoint(checkpoint, completion: completion)
        } else {
            performFullSync(completion: completion)
        }
    }
    
    // MARK: - Private Implementation
    
    private func handleBackgroundSync(task: BGAppRefreshTask) {
        // Schedule next background task
        scheduleBackgroundSync()
        
        // Set expiration handler
        task.expirationHandler = { [weak self] in
            self?.saveSyncCheckpoint()
            task.setTaskCompleted(success: false)
        }
        
        // Only sync if charging and on WiFi
        guard isDeviceCharging(), isWiFiConnected() else {
            task.setTaskCompleted(success: true)
            return
        }
        
        // Perform sync
        performFullSync { result in
            switch result {
            case .success:
                task.setTaskCompleted(success: true)
            case .failure:
                task.setTaskCompleted(success: false)
            }
        }
    }
    
    private func performFullSync(completion: @escaping (Result<SyncResult, SyncError>) -> Void) {
        isSyncing = true
        
        // Begin background task for extended execution time
        backgroundTask = UIApplication.shared.beginBackgroundTask { [weak self] in
            self?.saveSyncCheckpoint()
            UIApplication.shared.endBackgroundTask(self?.backgroundTask ?? .invalid)
        }
        
        // Set timeout handler
        let timeoutWorkItem = DispatchWorkItem { [weak self] in
            self?.handleSyncTimeout(completion: completion)
        }
        DispatchQueue.global(qos: .utility).asyncAfter(
            deadline: .now() + maxExecutionTime,
            execute: timeoutWorkItem
        )
        
        // Perform sync operations
        DispatchQueue.global(qos: .utility).async { [weak self] in
            guard let self = self else { return }
            
            do {
                // Step 1: Generate CRDT delta
                let delta = try self.crdtManager.generateDelta()
                
                // Step 2: Encrypt delta
                let encryptedDelta = try self.encryptDelta(delta)
                
                // Step 3: Upload to relay
                try self.uploadToRelay(encryptedDelta)
                
                // Step 4: Download remote changes
                let remoteDelta = try self.downloadFromRelay()
                
                // Step 5: Decrypt and apply
                let decryptedDelta = try self.decryptDelta(remoteDelta)
                try self.crdtManager.applyDelta(decryptedDelta)
                
                // Success
                self.lastSyncTime = Date()
                timeoutWorkItem.cancel()
                
                DispatchQueue.main.async {
                    self.isSyncing = false
                    UIApplication.shared.endBackgroundTask(self.backgroundTask)
                    completion(.success(SyncResult(
                        bytesUploaded: encryptedDelta.count,
                        bytesDownloaded: remoteDelta.count,
                        timestamp: Date()
                    )))
                }
                
            } catch let error as SyncError {
                timeoutWorkItem.cancel()
                DispatchQueue.main.async {
                    self.isSyncing = false
                    UIApplication.shared.endBackgroundTask(self.backgroundTask)
                    completion(.failure(error))
                }
            } catch {
                timeoutWorkItem.cancel()
                DispatchQueue.main.async {
                    self.isSyncing = false
                    UIApplication.shared.endBackgroundTask(self.backgroundTask)
                    completion(.failure(.networkUnavailable))
                }
            }
        }
    }
    
    private func resumeSyncFromCheckpoint(
        _ checkpoint: SyncCheckpoint,
        completion: @escaping (Result<SyncResult, SyncError>) -> Void
    ) {
        print("[SyncManager] Resuming sync from checkpoint: \(checkpoint.bytesTransferred)/\(checkpoint.totalBytes) bytes")
        
        // Resume blob upload/download from checkpoint
        // Implementation depends on specific operation type
        performFullSync(completion: completion)
    }
    
    private func handleSyncTimeout(completion: @escaping (Result<SyncResult, SyncError>) -> Void) {
        saveSyncCheckpoint()
        isSyncing = false
        UIApplication.shared.endBackgroundTask(backgroundTask)
        completion(.failure(.executionTimeout))
    }
    
    private func saveSyncCheckpoint() {
        // Save current sync progress for resume
        let checkpoint = SyncCheckpoint(
            operationId: UUID(),
            bytesTransferred: 0,
            totalBytes: 0,
            timestamp: Date()
        )
        checkpointManager.saveCheckpoint(checkpoint)
    }
    
    // MARK: - Encryption
    
    private func encryptDelta(_ delta: Data) throws -> Data {
        // Retrieve sync key from Secure Enclave
        guard let syncKey = keychainManager.retrieveKey(identifier: "sync_key") else {
            throw SyncError.authenticationFailed
        }
        
        // Encrypt using AES-256-GCM
        return try CryptoUtils.encryptAESGCM(data: delta, key: syncKey)
    }
    
    private func decryptDelta(_ encryptedData: Data) throws -> Data {
        guard let syncKey = keychainManager.retrieveKey(identifier: "sync_key") else {
            throw SyncError.authenticationFailed
        }
        
        return try CryptoUtils.decryptAESGCM(data: encryptedData, key: syncKey)
    }
    
    // MARK: - Network Operations
    
    private func uploadToRelay(_ data: Data) throws {
        // Upload encrypted delta to Cloudflare relay
        // Implementation uses URLSession with background configuration
        try networkManager.upload(data: data, endpoint: "/sync/upload")
    }
    
    private func downloadFromRelay() throws -> Data {
        // Download encrypted delta from Cloudflare relay
        return try networkManager.download(endpoint: "/sync/download")
    }
    
    // MARK: - Device State
    
    private func isDeviceCharging() -> Bool {
        let powerState = NSProcessInfo.processInfo.isLowPowerModeEnabled
        // Note: iOS doesn't directly expose charging state to apps
        // We use battery monitoring and heuristics
        UIDevice.current.isBatteryMonitoringEnabled = true
        let batteryState = UIDevice.current.batteryState
        return batteryState == .charging || batteryState == .full
    }
    
    private func isWiFiConnected() -> Bool {
        // Check WiFi connectivity using NWPathMonitor
        return networkManager.isWiFiConnected
    }
    
    @objc private func handlePowerStateChange() {
        if isDeviceCharging() && isWiFiConnected() {
            syncWhenCharging()
        }
    }
    
    private func beginBackgroundSync() {
        // Request extended background execution
        backgroundTask = UIApplication.shared.beginBackgroundTask { [weak self] in
            self?.saveSyncCheckpoint()
            UIApplication.shared.endBackgroundTask(self?.backgroundTask ?? .invalid)
        }
        
        performFullSync { [weak self] _ in
            UIApplication.shared.endBackgroundTask(self?.backgroundTask ?? .invalid)
        }
    }
    
    // MARK: - Logging
    
    private func logConflictResolution(_ conflict: SyncConflict, mergedState: CRDTState) {
        // Log conflict resolution for debugging (no PII)
        print("[SyncManager] Conflict resolved - Local version: \(conflict.localVersion), Remote version: \(conflict.remoteVersion)")
    }
}

// MARK: - Supporting Types

struct SyncResult {
    let bytesUploaded: Int
    let bytesDownloaded: Int
    let timestamp: Date
}

struct SyncConflict {
    let localState: CRDTState
    let remoteState: CRDTState
    let localVersion: Int
    let remoteVersion: Int
    let timestamp: Date
}

struct CRDTState {
    let vectorClock: [String: Int]
    let data: Data
}

// MARK: - Manager Placeholders (Full implementations in separate files)

class SyncCheckpointManager {
    static let shared = SyncCheckpointManager()
    func saveCheckpoint(_ checkpoint: SyncManager.SyncCheckpoint) {}
    func loadCheckpoint() -> SyncManager.SyncCheckpoint? { return nil }
}

class NetworkManager {
    static let shared = NetworkManager()
    var isWiFiConnected: Bool = false
    func upload(data: Data, endpoint: String) throws {}
    func download(endpoint: String) throws -> Data { return Data() }
}

class CRDTManager {
    static let shared = CRDTManager()
    func generateDelta() throws -> Data { return Data() }
    func applyDelta(_ delta: Data) throws {}
    func mergeConflicts(_ local: CRDTState, _ remote: CRDTState) -> CRDTState { return local }
    func applyMergedState(_ state: CRDTState) {}
}

class CryptoUtils {
    static func encryptAESGCM(data: Data, key: Data) throws -> Data { return Data() }
    static func decryptAESGCM(data: Data, key: Data) throws -> Data { return Data() }
}
