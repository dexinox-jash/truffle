//
//  TruffleApp.swift
//  TruffleApp
//
//  Project Truffle - Main App Entry Point
//  SPEC v2.0 Section 5.3: Mobile Companion
//
//  iOS App entry point with:
//  - Deep link handling
//  - Background task registration
//  - App lifecycle management
//

import SwiftUI
import BackgroundTasks

@main
struct TruffleApp: App {
    
    @UIApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    @Environment(\.scenePhase) private var scenePhase
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .onOpenURL { url in
                    handleDeepLink(url)
                }
        }
        .onChange(of: scenePhase) { newPhase in
            switch newPhase {
            case .background:
                scheduleBackgroundTasks()
            case .inactive:
                break
            case .active:
                cancelBackgroundTasks()
            @unknown default:
                break
            }
        }
    }
    
    private func handleDeepLink(_ url: URL) {
        guard url.scheme == "truffle" else { return }
        
        switch url.host {
        case "open":
            // Open main app
            break
        case "export":
            // Navigate to export view
            NotificationCenter.default.post(name: .showExportView, object: nil)
        case "search":
            // Show search with query
            if let query = URLComponents(url: url, resolvingAgainstBaseURL: false)?
                .queryItems?.first(where: { $0.name == "q" })?.value {
                NotificationCenter.default.post(
                    name: .showSearch,
                    object: nil,
                    userInfo: ["query": query]
                )
            }
        default:
            break
        }
    }
    
    private func scheduleBackgroundTasks() {
        // Schedule sync
        SyncManager.shared.scheduleBackgroundSync()
        
        // Schedule compilation
        CompilationManager.shared.scheduleBackgroundProcessing()
    }
    
    private func cancelBackgroundTasks() {
        // Cancel any pending background tasks if needed
    }
}

// MARK: - App Delegate

class AppDelegate: NSObject, UIApplicationDelegate {
    
    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil
    ) -> Bool {
        // Register background tasks
        registerBackgroundTasks()
        
        // Setup notifications
        setupNotifications()
        
        // Initialize core systems
        initializeCoreSystems()
        
        return true
    }
    
    func application(
        _ application: UIApplication,
        configurationForConnecting connectingSceneSession: UISceneSession,
        options: UIScene.ConnectionOptions
    ) -> UISceneConfiguration {
        return UISceneConfiguration(
            name: "Default Configuration",
            sessionRole: connectingSceneSession.role
        )
    }
    
    func application(
        _ application: UIApplication,
        didDiscardSceneSessions sceneSessions: Set<UISceneSession>
    ) {
        // Handle discarded scenes
    }
    
    func applicationDidEnterBackground(_ application: UIApplication) {
        // Schedule background tasks
        SyncManager.shared.scheduleBackgroundSync()
        CompilationManager.shared.scheduleBackgroundProcessing()
    }
    
    // MARK: - Private Methods
    
    private func registerBackgroundTasks() {
        // Register sync task
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: "com.truffle.sync.background",
            using: nil
        ) { task in
            self.handleBackgroundSync(task: task as! BGAppRefreshTask)
        }
        
        // Register compilation task
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: "com.truffle.compilation.background",
            using: nil
        ) { task in
            self.handleBackgroundCompilation(task: task as! BGProcessingTask)
        }
    }
    
    private func setupNotifications() {
        // Request notification permissions
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .badge, .sound]) { granted, _ in
            print("[AppDelegate] Notification permission: \(granted)")
        }
    }
    
    private func initializeCoreSystems() {
        // Initialize database
        _ = CompilationDatabase.shared
        
        // Initialize storage
        _ = RawStorageManager.shared
        
        // Initialize keychain
        _ = KeychainManager.shared
        
        print("[AppDelegate] Core systems initialized")
    }
    
    private func handleBackgroundSync(task: BGAppRefreshTask) {
        let queue = OperationQueue()
        queue.maxConcurrentOperationCount = 1
        
        let syncOperation = BlockOperation {
            let semaphore = DispatchSemaphore(value: 0)
            
            SyncManager.shared.forceSync { result in
                switch result {
                case .success:
                    task.setTaskCompleted(success: true)
                case .failure:
                    task.setTaskCompleted(success: false)
                }
                semaphore.signal()
            }
            
            semaphore.wait()
        }
        
        task.expirationHandler = {
            queue.cancelAllOperations()
        }
        
        queue.addOperations([syncOperation], waitUntilFinished: false)
    }
    
    private func handleBackgroundCompilation(task: BGProcessingTask) {
        // Process compilation queue
        CompilationManager.shared.processQueue()
        
        // Complete after processing
        DispatchQueue.main.asyncAfter(deadline: .now() + 25) {
            let status = CompilationManager.shared.getQueueStatus()
            task.setTaskCompleted(success: status.pending == 0)
        }
    }
}

// MARK: - Notification Names

extension Notification.Name {
    static let showExportView = Notification.Name("showExportView")
    static let showSearch = Notification.Name("showSearch")
}
