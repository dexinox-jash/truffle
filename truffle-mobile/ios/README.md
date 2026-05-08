# Truffle iOS - Mobile Companion

**SPEC v2.0 Section 5.3: Mobile Companion Implementation**

This directory contains the iOS implementation for Project Truffle, a local-first knowledge compiler that transforms screenshots into structured wiki content.

## Architecture Overview

```
Truffle iOS
├── TruffleApp/              # Main iOS Application
│   ├── TruffleApp.swift     # App entry point
│   ├── ContentView.swift    # Main content view with tabs
│   ├── Views/               # UI Views
│   │   ├── WikiBrowserView.swift   # Read-only wiki browser
│   │   └── SearchView.swift        # Search interface
│   ├── ViewModels/          # View models
│   └── Info.plist           # App configuration
│
├── TruffleShareExtension/   # iOS Share Extension
│   ├── ShareViewController.swift   # Screenshot capture handler
│   ├── Info.plist                  # Extension configuration
│   └── TruffleShareExtension.entitlements
│
├── TruffleCore/             # Shared Framework
│   ├── SyncManager.swift           # Background sync (15min when charging)
│   ├── KeychainManager.swift       # Secure key storage (Secure Enclave)
│   ├── CompilationManager.swift    # Screenshot compilation
│   ├── RawStorageManager.swift     # Raw image storage
│   └── CheckpointManager.swift     # 30s limit checkpoint/resume
│
├── Podfile                  # CocoaPods dependencies
└── project.pbxproj          # Xcode project
```

## Key Features

### 1. Share Extension (Screenshot Capture)
- Activated from iOS Share Sheet for single images
- Saves to `raw/` storage with content-addressable UUID
- Triggers background compilation
- Respects 30s background execution limit with checkpoint/resume

### 2. Read-Only Wiki Browser
- Tree view navigation (Obsidian-style)
- WikiLink support `[[Target]]`
- Backlinks display
- Original screenshot viewing with zoom
- Markdown rendering with custom theme

### 3. Background Sync
- BGTaskScheduler for background fetch
- Runs every 15 minutes when charging + WiFi
- CRDT delta sync via Zero-Knowledge protocol
- X3DH key exchange with Kyber-768 hybrid

### 4. Secure Storage
- iOS Secure Enclave for private keys
- Keychain with `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`
- AES-256-GCM encryption
- Biometric authentication option

### 5. Storage Management
- Warning at 5GB usage
- Auto-pause captures at 10GB
- Thumbnail generation
- Export to Files app

## Technical Constraints (Section 5.3.2)

| Constraint | Implementation |
|------------|----------------|
| 30s background limit | Checkpoint/resume with BGTaskScheduler |
| Storage warning 5GB | Notification + UI indicator |
| Storage pause 10GB | Block new captures, prompt export |
| Read-only wiki | No editing UI, view-only mode |
| Secure Enclave | KeychainManager with biometric auth |
| Background fetch 15min | BGAppRefreshTask when charging |

## Dependencies

```ruby
# Database
pod 'SQLite.swift', '~> 0.14.0'

# YAML parsing
pod 'Yams', '~> 5.0.0'

# Markdown rendering
pod 'MarkdownUI', '~> 2.0.0'

# Image loading
pod 'SDWebImageSwiftUI', '~> 2.0.0'

# Network
pod 'Alamofire', '~> 5.8.0'
```

## Build Instructions

1. Install dependencies:
```bash
cd /mnt/okcomputer/output/truffle/truffle-mobile/ios
pod install
```

2. Open workspace in Xcode:
```bash
open Truffle.xcworkspace
```

3. Configure signing:
   - Select your team
   - Update bundle identifiers if needed
   - Enable App Groups capability

4. Build and run on device or simulator

## File Structure

### ShareViewController.swift
Main share extension entry point:
- `isContentValid()` - Validate share content
- `didSelectPost()` - Process screenshot
- `processScreenshot()` - Main processing pipeline
- `saveToRawStorage()` - Persist to filesystem
- `triggerCompilation()` - Queue for AI processing

### SyncManager.swift
Background synchronization:
- `scheduleBackgroundSync()` - Register BGTaskScheduler
- `syncWhenCharging()` - Conditional sync
- `handleSyncConflict()` - CRDT merge resolution
- Checkpoint/resume for 30s limit

### KeychainManager.swift
Secure key storage:
- `storeKey()` - Store in Secure Enclave/Keychain
- `retrieveKey()` - Retrieve key
- `deleteKey()` - Remove key
- `generateAndStoreKey()` - Generate new key
- `storeKeyWithBiometric()` - Biometric-protected keys

### WikiBrowserView.swift
Read-only wiki browser:
- Tree view navigation
- WikiLink tap handling
- Backlinks section
- Source screenshot viewing
- Markdown rendering

### SearchView.swift
Search interface:
- Full-text search
- Filter by type
- Recent searches
- Search suggestions
- Highlighted results

## Design System: Darkroom

| Element | Value |
|---------|-------|
| Background | `#0A0A0A` (OLED black) |
| Surface | `#141414` |
| Primary | `#FFB800` (Amber) |
| Secondary | `#00D4AA` (Success) |
| Text Primary | `#E5E5E5` |
| Text Secondary | `#737373` |
| Border | `#262626` |

## Mobile-Only Limitations

Per SPEC v2.0, mobile is **companion-only**:
- ✅ Screenshot capture via Share Extension
- ✅ Read-only wiki browsing
- ✅ Search and navigation
- ✅ Background sync
- ❌ Wiki editing (desktop only)
- ❌ Schema editing (desktop only)
- ❌ Full compilation (background only)

## Testing

Run unit tests:
```bash
xcodebuild test -workspace Truffle.xcworkspace -scheme TruffleApp -destination 'platform=iOS Simulator,name=iPhone 15'
```

## License

AGPL v3 - See LICENSE file
