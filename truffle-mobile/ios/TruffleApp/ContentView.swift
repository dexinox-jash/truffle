//
//  ContentView.swift
//  TruffleApp
//
//  Project Truffle - Main App Content View
//  SPEC v2.0 Section 5.3.1: Mobile Companion - Read-only wiki browser
//
//  Main entry point for the iOS app
//  - Read-only wiki browser (no editing on mobile)
//  - Search and navigation
//  - Sync status display
//

import SwiftUI

// MARK: - Main Content View
struct ContentView: View {
    
    // MARK: - State
    @StateObject private var viewModel = WikiViewModel()
    @StateObject private var syncViewModel = SyncStatusViewModel()
    @State private var selectedTab = 0
    @State private var showSearch = false
    
    // MARK: - Body
    var body: some View {
        TabView(selection: $selectedTab) {
            // Wiki Browser Tab
            NavigationView {
                WikiBrowserView(viewModel: viewModel)
                    .navigationTitle("Wiki")
                    .toolbar {
                        ToolbarItem(placement: .navigationBarTrailing) {
                            Button(action: { showSearch = true }) {
                                Image(systemName: "magnifyingglass")
                            }
                        }
                    }
            }
            .tabItem {
                Image(systemName: "doc.text")
                Text("Wiki")
            }
            .tag(0)
            
            // Raw Storage Tab
            NavigationView {
                RawStorageView()
                    .navigationTitle("Raw")
            }
            .tabItem {
                Image(systemName: "photo.on.rectangle")
                Text("Raw")
            }
            .tag(1)
            
            // Sync Status Tab
            NavigationView {
                SyncStatusView(viewModel: syncViewModel)
                    .navigationTitle("Sync")
            }
            .tabItem {
                Image(systemName: "arrow.triangle.2.circlepath")
                Text("Sync")
            }
            .tag(2)
            
            // Settings Tab
            NavigationView {
                SettingsView()
                    .navigationTitle("Settings")
            }
            .tabItem {
                Image(systemName: "gear")
                Text("Settings")
            }
            .tag(3)
        }
        .accentColor(Color(hex: "#FFB800")) // Amber primary color
        .sheet(isPresented: $showSearch) {
            SearchView(viewModel: SearchViewModel(wikiViewModel: viewModel))
        }
        .onAppear {
            // Apply Darkroom design system
            setupAppearance()
            
            // Initialize app
            viewModel.loadWikiNodes()
            syncViewModel.startMonitoring()
        }
    }
    
    // MARK: - Appearance Setup
    private func setupAppearance() {
        // Dark theme (Darkroom design system)
        UINavigationBar.appearance().barTintColor = UIColor(hex: "#0A0A0A")
        UINavigationBar.appearance().backgroundColor = UIColor(hex: "#0A0A0A")
        UINavigationBar.appearance().titleTextAttributes = [
            .foregroundColor: UIColor(hex: "#E5E5E5")
        ]
        UINavigationBar.appearance().tintColor = UIColor(hex: "#FFB800")
        
        UITabBar.appearance().barTintColor = UIColor(hex: "#141414")
        UITabBar.appearance().backgroundColor = UIColor(hex: "#141414")
        UITabBar.appearance().unselectedItemTintColor = UIColor(hex: "#737373")
        
        // Enable dark mode
        UIApplication.shared.windows.first?.overrideUserInterfaceStyle = .dark
    }
}

// MARK: - Supporting Views

// Raw Storage View
struct RawStorageView: View {
    @StateObject private var viewModel = RawStorageViewModel()
    
    var body: some View {
        List(viewModel.artifacts) { artifact in
            RawArtifactRow(artifact: artifact)
        }
        .listStyle(PlainListStyle())
        .background(Color(hex: "#0A0A0A"))
        .onAppear {
            viewModel.loadArtifacts()
        }
    }
}

struct RawArtifactRow: View {
    let artifact: RawArtifactDisplay
    
    var body: some View {
        HStack {
            // Thumbnail
            if let thumbnail = artifact.thumbnail {
                Image(uiImage: thumbnail)
                    .resizable()
                    .scaledToFill()
                    .frame(width: 60, height: 60)
                    .cornerRadius(8)
            } else {
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color(hex: "#262626"))
                    .frame(width: 60, height: 60)
                    .overlay(
                        Image(systemName: "photo")
                            .foregroundColor(Color(hex: "#737373"))
                    )
            }
            
            VStack(alignment: .leading, spacing: 4) {
                Text(artifact.filename)
                    .font(.system(size: 16, weight: .medium))
                    .foregroundColor(Color(hex: "#E5E5E5"))
                    .lineLimit(1)
                
                Text(artifact.capturedAt)
                    .font(.system(size: 12))
                    .foregroundColor(Color(hex: "#737373"))
                
                HStack {
                    StatusBadge(status: artifact.compilationStatus)
                    Spacer()
                    Text(artifact.sizeString)
                        .font(.system(size: 12))
                        .foregroundColor(Color(hex: "#737373"))
                }
            }
        }
        .padding(.vertical, 4)
        .background(Color(hex: "#141414"))
    }
}

struct StatusBadge: View {
    let status: CompilationStatusDisplay
    
    var body: some View {
        Text(status.label)
            .font(.system(size: 10, weight: .medium))
            .padding(.horizontal, 8)
            .padding(.vertical, 2)
            .background(status.color.opacity(0.2))
            .foregroundColor(status.color)
            .cornerRadius(4)
    }
}

// Sync Status View
struct SyncStatusView: View {
    @ObservedObject var viewModel: SyncStatusViewModel
    
    var body: some View {
        VStack(spacing: 20) {
            // Sync status card
            SyncStatusCard(viewModel: viewModel)
            
            // Device list
            DeviceListView(devices: viewModel.pairedDevices)
            
            // Sync history
            SyncHistoryView(history: viewModel.syncHistory)
            
            Spacer()
        }
        .padding()
        .background(Color(hex: "#0A0A0A"))
    }
}

struct SyncStatusCard: View {
    @ObservedObject var viewModel: SyncStatusViewModel
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: viewModel.statusIcon)
                    .foregroundColor(viewModel.statusColor)
                
                Text(viewModel.statusText)
                    .font(.system(size: 18, weight: .semibold))
                    .foregroundColor(Color(hex: "#E5E5E5"))
                
                Spacer()
                
                if viewModel.isSyncing {
                    ProgressView()
                        .progressViewStyle(CircularProgressViewStyle(tint: Color(hex: "#FFB800")))
                }
            }
            
            if let lastSync = viewModel.lastSyncTime {
                Text("Last sync: \(lastSync)")
                    .font(.system(size: 14))
                    .foregroundColor(Color(hex: "#737373"))
            }
            
            if let error = viewModel.lastError {
                Text("Error: \(error)")
                    .font(.system(size: 14))
                    .foregroundColor(.red)
            }
            
            // Manual sync button
            Button(action: { viewModel.forceSync() }) {
                HStack {
                    Image(systemName: "arrow.clockwise")
                    Text("Sync Now")
                }
                .frame(maxWidth: .infinity)
                .padding()
                .background(Color(hex: "#FFB800"))
                .foregroundColor(Color(hex: "#0A0A0A"))
                .cornerRadius(8)
            }
            .disabled(viewModel.isSyncing)
        }
        .padding()
        .background(Color(hex: "#141414"))
        .cornerRadius(12)
    }
}

struct DeviceListView: View {
    let devices: [PairedDevice]
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Paired Devices")
                .font(.system(size: 16, weight: .semibold))
                .foregroundColor(Color(hex: "#E5E5E5"))
            
            ForEach(devices) { device in
                HStack {
                    Image(systemName: device.icon)
                        .foregroundColor(Color(hex: "#00D4AA"))
                    
                    VStack(alignment: .leading) {
                        Text(device.name)
                            .font(.system(size: 14))
                            .foregroundColor(Color(hex: "#E5E5E5"))
                        Text(device.lastSeen)
                            .font(.system(size: 12))
                            .foregroundColor(Color(hex: "#737373"))
                    }
                    
                    Spacer()
                    
                    Circle()
                        .fill(device.isOnline ? Color(hex: "#00D4AA") : Color(hex: "#737373"))
                        .frame(width: 8, height: 8)
                }
                .padding(.vertical, 4)
            }
        }
        .padding()
        .background(Color(hex: "#141414"))
        .cornerRadius(12)
    }
}

struct SyncHistoryView: View {
    let history: [SyncHistoryItem]
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Recent Activity")
                .font(.system(size: 16, weight: .semibold))
                .foregroundColor(Color(hex: "#E5E5E5"))
            
            ForEach(history.prefix(5)) { item in
                HStack {
                    Image(systemName: item.success ? "checkmark.circle" : "xmark.circle")
                        .foregroundColor(item.success ? Color(hex: "#00D4AA") : .red)
                    
                    Text(item.description)
                        .font(.system(size: 14))
                        .foregroundColor(Color(hex: "#E5E5E5"))
                    
                    Spacer()
                    
                    Text(item.timeAgo)
                        .font(.system(size: 12))
                        .foregroundColor(Color(hex: "#737373"))
                }
            }
        }
        .padding()
        .background(Color(hex: "#141414"))
        .cornerRadius(12)
    }
}

// Settings View
struct SettingsView: View {
    @StateObject private var viewModel = SettingsViewModel()
    @State private var showExportSheet = false
    @State private var showClearConfirmation = false
    
    var body: some View {
        List {
            // Storage Section
            Section(header: Text("Storage").foregroundColor(Color(hex: "#737373"))) {
                StorageUsageView(
                    usedSpace: viewModel.usedStorage,
                    totalSpace: viewModel.totalStorage
                )
                
                Button(action: { showExportSheet = true }) {
                    Label("Export to Files", systemImage: "square.and.arrow.up")
                }
                .foregroundColor(Color(hex: "#FFB800"))
                
                Button(action: { showClearConfirmation = true }) {
                    Label("Clear All Data", systemImage: "trash")
                }
                .foregroundColor(.red)
            }
            
            // Sync Section
            Section(header: Text("Sync").foregroundColor(Color(hex: "#737373"))) {
                Toggle("Auto-sync when charging", isOn: $viewModel.autoSyncWhenCharging)
                Toggle("Sync only on WiFi", isOn: $viewModel.syncOnlyOnWiFi)
                
                NavigationLink(destination: DevicePairingView()) {
                    Label("Pair New Device", systemImage: "qrcode")
                }
            }
            
            // Privacy Section
            Section(header: Text("Privacy").foregroundColor(Color(hex: "#737373"))) {
                NavigationLink(destination: PrivacySettingsView()) {
                    Label("Privacy Settings", systemImage: "lock.shield")
                }
                
                NavigationLink(destination: ExportKeysView()) {
                    Label("Export Keys", systemImage: "key")
                }
            }
            
            // About Section
            Section(header: Text("About").foregroundColor(Color(hex: "#737373"))) {
                HStack {
                    Text("Version")
                    Spacer()
                    Text(viewModel.appVersion)
                        .foregroundColor(Color(hex: "#737373"))
                }
                
                Link(destination: URL(string: "https://truffle.io/privacy")!) {
                    Label("Privacy Policy", systemImage: "doc.text")
                }
                
                Link(destination: URL(string: "https://truffle.io/terms")!) {
                    Label("Terms of Service", systemImage: "doc.text")
                }
            }
        }
        .listStyle(InsetGroupedListStyle())
        .background(Color(hex: "#0A0A0A"))
        .sheet(isPresented: $showExportSheet) {
            ExportView()
        }
        .alert(isPresented: $showClearConfirmation) {
            Alert(
                title: Text("Clear All Data?"),
                message: Text("This will delete all screenshots and wiki data. This cannot be undone."),
                primaryButton: .destructive(Text("Clear")) {
                    viewModel.clearAllData()
                },
                secondaryButton: .cancel()
            )
        }
    }
}

struct StorageUsageView: View {
    let usedSpace: UInt64
    let totalSpace: UInt64
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Storage Used")
                Spacer()
                Text("\(formatBytes(usedSpace)) / \(formatBytes(totalSpace))")
                    .foregroundColor(Color(hex: "#737373"))
            }
            
            GeometryReader { geometry in
                ZStack(alignment: .leading) {
                    Rectangle()
                        .fill(Color(hex: "#262626"))
                        .frame(height: 8)
                        .cornerRadius(4)
                    
                    Rectangle()
                        .fill(usageColor)
                        .frame(width: geometry.size.width * usagePercentage, height: 8)
                        .cornerRadius(4)
                }
            }
            .frame(height: 8)
        }
        .padding(.vertical, 4)
    }
    
    private var usagePercentage: CGFloat {
        guard totalSpace > 0 else { return 0 }
        return CGFloat(min(Double(usedSpace) / Double(totalSpace), 1.0))
    }
    
    private var usageColor: Color {
        if usedSpace >= 10_000_000_000 { // 10GB
            return .red
        } else if usedSpace >= 5_000_000_000 { // 5GB
            return Color(hex: "#FFB800")
        } else {
            return Color(hex: "#00D4AA")
        }
    }
    
    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }
}

// MARK: - Color Extension
extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 3: // RGB (12-bit)
            (a, r, g, b) = (255, (int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
        case 6: // RGB (24-bit)
            (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8: // ARGB (32-bit)
            (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (1, 1, 1, 0)
        }
        self.init(
            .sRGB,
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue: Double(b) / 255,
            opacity: Double(a) / 255
        )
    }
}

extension UIColor {
    convenience init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 3:
            (a, r, g, b) = (255, (int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
        case 6:
            (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8:
            (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (1, 1, 1, 0)
        }
        self.init(red: CGFloat(r) / 255, green: CGFloat(g) / 255, blue: CGFloat(b) / 255, alpha: CGFloat(a) / 255)
    }
}

// MARK: - Preview
struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
            .preferredColorScheme(.dark)
    }
}
