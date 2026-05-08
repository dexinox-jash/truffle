# Truffle Mobile - iOS & Android Share Extensions

**Project Truffle - Mobile Companion (Section 5.3)**

> **CRITICAL**: Mobile is COMPANION-ONLY. Full functionality on desktop. This is intentional design.

## Overview

Truffle Mobile provides companion apps for iOS and Android that enable:

1. **Capture**: Share Extension → Save to raw/ → Trigger background compilation
2. **View**: Read-only wiki browser (search, navigate links, view original screenshots)
3. **Sync**: Background sync every 15 minutes when charging

## Technical Constraints (Section 5.3.2)

| Constraint | iOS | Android |
|------------|-----|---------|
| Background Execution | 30s limit (checkpoint/resume) | WorkManager with constraints |
| Storage Warning | 5GB | 5GB |
| Storage Pause | 10GB (auto-pause captures) | 10GB (auto-pause captures) |
| Secure Storage | Secure Enclave / Keychain | Android Keystore |
| Background Sync | BGTaskScheduler (15 min) | WorkManager (15 min) |

## Project Structure

```
truffle-mobile/
├── ios/                          # iOS App & Share Extension
│   ├── TruffleApp/              # Main iOS App
│   │   ├── TruffleApp.swift     # App entry point
│   │   └── Views/
│   │       ├── ContentView.swift      # Main container
│   │       ├── WikiBrowserView.swift  # Read-only wiki browser
│   │       └── SearchView.swift       # Search interface
│   ├── TruffleShareExtension/   # iOS Share Extension
│   │   ├── ShareViewController.swift  # Share sheet UI
│   │   └── CaptureManager.swift       # Capture handling
│   ├── TruffleCore/             # Shared iOS framework
│   │   ├── KeychainManager.swift      # Secure Enclave storage
│   │   ├── BackgroundTaskManager.swift # BGTaskScheduler
│   │   └── SyncManager.swift          # ZKS-1 sync protocol
│   └── Podfile                  # CocoaPods dependencies
│
├── android/                      # Android App & Share Extension
│   └── app/src/main/java/com/truffle/
│       ├── app/                 # Main Android App
│       │   ├── MainActivity.kt          # Entry point
│       │   └── ui/screens/
│       │       ├── WikiBrowserScreen.kt # Read-only wiki browser
│       │       ├── SearchScreen.kt      # Search interface
│       │       ├── CapturesScreen.kt    # Capture list
│       │       └── SettingsScreen.kt    # Settings
│       ├── shareextension/      # Android Share Extension
│       │   ├── ShareActivity.kt         # Share receiver
│       │   └── CaptureReceiver.kt       # Broadcast receiver
│       ├── workmanager/         # Background processing
│       │   └── CompilationWorker.kt     # Checkpoint/resume compilation
│       └── keystorage/          # Secure key storage
│           └── KeystoreManager.kt       # Android Keystore
│
└── shared/                       # Shared logic (optional)
    └── (Rust or Kotlin Multiplatform)
```

## iOS Implementation

### Share Extension

The iOS Share Extension (`ShareViewController`) handles:

1. Receiving images via iOS Share Sheet
2. Saving to app group container (`raw/` directory)
3. Triggering background compilation via `BGTaskScheduler`
4. 30-second execution limit with checkpoint/resume

```swift
// Key components:
- ShareViewController.swift    # UI and processing
- CaptureManager.swift         # Save to raw/, trigger compilation
- CheckpointManager          # Resume interrupted operations
```

### Secure Storage

Uses iOS Secure Enclave via Keychain:

```swift
// KeychainManager.swift
- Identity keys (Ed25519) for device pairing
- Sync key (AES-256-GCM) for encrypted sync
- Auth key (HMAC-SHA256) for message authentication
- Device fingerprint (SHA-256 hashed)
```

### Background Tasks

```swift
// BackgroundTaskManager.swift
- BGProcessingTask: Compilation (30s budget)
- BGAppRefreshTask: Periodic sync (15 min when charging)
- Checkpoint/resume for long operations
```

## Android Implementation

### Share Extension

The Android Share Extension (`ShareActivity`) handles:

1. Receiving images via `Intent.ACTION_SEND`
2. Saving to app's private storage (`raw/` directory)
3. Enqueuing WorkManager compilation task
4. Battery-aware scheduling

```kotlin
// Key components:
- ShareActivity.kt           # Share receiver UI
- CaptureReceiver.kt         # System event handling
- ShareActivity.xml          # Overlay UI layout
```

### Secure Storage

Uses Android Keystore (hardware-backed when available):

```kotlin
// KeystoreManager.kt
- Identity key pair (EC P-256) for device pairing
- Sync key (AES-256-GCM) for encrypted sync
- X3DH key exchange support
- HKDF-SHA256 key derivation
```

### WorkManager

```kotlin
// CompilationWorker.kt
- PeriodicWorkRequest: Every 15 min when charging
- OneTimeWorkRequest: Immediate compilation
- Constraints: Battery not low, storage not low
- Checkpoint/resume for 5-stage compilation pipeline
```

## Compilation Pipeline (Checkpoint/Resume)

Both platforms implement a 5-stage compilation pipeline with checkpoint/resume:

```
Stage 1: OCR (Optical Character Recognition)
    ↓ [checkpoint]
Stage 2: Classification (receipt, travel, contact, etc.)
    ↓ [checkpoint]
Stage 3: Entity Extraction (merchant, amount, dates, etc.)
    ↓ [checkpoint]
Stage 4: Linking (connect to existing wiki nodes)
    ↓ [checkpoint]
Stage 5: Persistence (save to wiki/ directory)
    ↓ [clear checkpoint]
```

If interrupted (iOS 30s limit, app termination), the pipeline resumes from the last checkpoint.

## Zero-Knowledge Sync Protocol (ZKS-1)

Mobile implements the ZKS-1 protocol for device sync:

### Cryptographic Primitives

| Algorithm | Purpose | iOS | Android |
|-----------|---------|-----|---------|
| X3DH | Key exchange | P256.KeyAgreement | ECDH |
| AES-256-GCM | Symmetric encryption | CryptoKit | Cipher |
| ChaCha20-Poly1305 | Mobile encryption | CryptoKit | Cipher |
| HMAC-SHA256 | Message authentication | CryptoKit | Mac |
| HKDF-SHA256 | Key derivation | CryptoKit | KeystoreManager |

### Sync Message Format

```swift/kotlin
struct SyncMessage {
    header: {
        protocol_version: 1
        device_id: String (hashed fingerprint)
        timestamp: UnixMs
        nonce: 12 bytes
    }
    payload: ChaCha20-Poly1305 encrypted {
        crdt_update: Yjs binary diff
        schema_version: String
        deleted_artifacts: UUID[] (tombstones)
    }
    mac: HMAC-SHA256
}
```

## Darkroom Design System

Mobile apps follow the Darkroom design system:

| Element | Color | Usage |
|---------|-------|-------|
| Background | #0A0A0A | Pure black (OLED optimization) |
| Surface | #141414 | Cards, elevated elements |
| Border | #262626 | Dividers, outlines |
| Primary | #FFB800 | Amber (actions, links) |
| Secondary | #00D4AA | Success, encrypted indicator |
| Text Primary | #E5E5E5 | Main text |
| Text Secondary | #737373 | Captions, hints |

## Building

### iOS

```bash
cd ios
pod install
open Truffle.xcworkspace
# Build in Xcode (requires iOS 16.0+, Xcode 15+)
```

### Android

```bash
cd android
./gradlew assembleDebug
# Or open in Android Studio
```

## Testing

### iOS Tests

```bash
cd ios
xcodebuild test -workspace Truffle.xcworkspace -scheme TruffleApp -destination 'platform=iOS Simulator,name=iPhone 15'
```

### Android Tests

```bash
cd android
./gradlew test
./gradlew connectedAndroidTest  # Device tests
```

## Security Considerations

1. **No data leaves device unencrypted**
2. **Keys stored in hardware-backed keystore when available**
3. **Sync uses zero-knowledge protocol**
4. **No analytics or telemetry**
5. **Local-first architecture**

## License

AGPL-3.0 for local components (see main project LICENSE)
