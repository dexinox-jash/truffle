# Sync and Multi-Device Access

## Overview

Truffle Sync keeps your knowledge graph up-to-date across all your devices. Start work on your desktop, review on your tablet, and capture ideas on your phone - seamlessly.

[Screenshot: Sync status panel showing connected devices]

---

## How Sync Works

### Sync Architecture

Truffle uses a **secure cloud sync** with **local-first storage**:

`
+-------------+        +-------------+        +-------------+
|  Laptop     |<------>|  Truffle    |<------>|   Phone     |
|  (Local DB) |  Sync  |  Cloud      |  Sync  |  (Local DB) |
+-------------+        +-------------+        +-------------+
        |                      |                      |
        |                 Encrypted                |
        |                 Transmission             |
        +----------------------+----------------------+
`

### What Gets Synced

| Data | Synced | Notes |
|------|--------|-------|
| Entities | Yes | All content and metadata |
| Relationships | Yes | Complete graph structure |
| Attachments | Yes | Files are uploaded to cloud |
| Comments | Yes | Including resolved threads |
| Settings | Partial | Theme, preferences |
| Search History | No | Per-device only |
| View State | No | Each device remembers its own |

### Conflict Resolution

When the same entity is edited on multiple devices:

1. **Field-level merging** - Different fields merge automatically
2. **Text merging** - Smart diff for simultaneous text edits
3. **Timestamp priority** - If unresolvable, latest edit wins
4. **Conflict notifications** - You are notified of significant conflicts

---

## Setting Up Sync

### Step 1: Create a Truffle Account

If you do not have an account:

1. **Open Truffle** on your primary device
2. **Click Settings** -> **Sync**
3. **Click Enable Sync**
4. **Enter your email** and create a password
5. **Click Create Account**
6. **Verify your email** (check your inbox)

`
+-------------------------------------+
|  Enable Sync                        |
+-------------------------------------+
|                                     |
|  Email: [you@example.com        ]   |
|                                     |
|  Password: [****************    ]   |
|  (8+ characters, mixed case)        |
|                                     |
|  Confirm: [****************     ]   |
|                                     |
|  [x] I agree to the Terms of        |
|      Service and Privacy Policy     |
|                                     |
|  [Cancel]        [Create Account]   |
+-------------------------------------+
`

[Screenshot: Account creation form]

### Step 2: Sign In on Additional Devices

To add a new device:

1. **Install Truffle** on the new device
2. **Open Settings** -> **Sync**
3. **Click Sign In**
4. **Enter your email and password**
5. **Click Sign In**
6. Wait for initial sync to complete

### Step 3: Verify Sync Status

Check sync is working:

1. **Click the sync icon** in the toolbar
2. Verify status shows: **Synced** or **Syncing...**
3. See last sync time

`
+-------------------------------------+
|  Sync Status                        |
+-------------------------------------+
|                                     |
|  Status: Online and synced          |
|  Last sync: 2 minutes ago           |
|                                     |
|  Connected devices:                 |
|  - This Laptop (Active)             |
|  - iPhone 15 (Synced 5 min ago)     |
|  - iPad Pro (Synced 1 hour ago)     |
|                                     |
|  Storage used: 234 MB / 10 GB       |
|                                     |
|  [Sync Now]  [Settings]             |
+-------------------------------------+
`

---

## Device Pairing

### QR Code Pairing (Fastest)

Pair mobile devices quickly:

1. On desktop: **Settings** -> **Sync** -> **Add Device**
2. A QR code appears
3. On mobile: **Settings** -> **Sync** -> **Scan QR Code**
4. Point camera at the code
5. Devices are paired automatically

[Screenshot: QR code pairing screen]

### Manual Pairing

For devices without camera access:

1. On primary device: **Settings** -> **Sync** -> **Add Device** -> **Manual**
2. Copy the pairing code (e.g., TRF-ABC123-XYZ)
3. On new device: **Settings** -> **Sync** -> **Enter Pairing Code**
4. Type the code
5. **Click Pair**

### Managing Connected Devices

View and manage your devices:

**Settings** -> **Sync** -> **Devices**

`
+-------------------------------------+
|  Connected Devices                  |
+-------------------------------------+
|                                     |
|  Work Laptop                        |
|  Windows 11 - Active now            |
|  [Rename] [Remove]                  |
|                                     |
|  iPhone 15                          |
|  iOS 17 - Last seen: 10 min ago     |
|  [Rename] [Remove]                  |
|                                     |
|  iPad Pro                           |
|  iPadOS 17 - Last seen: 2 hours ago |
|  [Rename] [Remove]                  |
|                                     |
|  [Add New Device]                   |
+-------------------------------------+
`

### Removing a Device

To disconnect a device:

1. **Go to Devices list**
2. **Find the device** to remove
3. **Click Remove**
4. Confirm removal

The device will stop syncing and keep only its current local data.

---

## Offline Usage

### Working Offline

Truffle works fully offline with local data:

- **View all entities** - Everything stored locally
- **Create and edit** - Changes saved locally first
- **Add relationships** - New connections tracked
- **Search** - Local search works instantly

Changes sync automatically when you are back online.

### Offline Indicator

When offline, you will see:

`
+-------------------------------------+
|  OFFLINE MODE                       |
|  Changes will sync when reconnected |
+-------------------------------------+
`

### Preparing for Offline Work

To ensure content is available offline:

1. **Enable Offline Access** in entity settings
2. **Sync before traveling**
3. **Pin important entities** for guaranteed offline access

### Sync Queue

View pending changes:

**Settings** -> **Sync** -> **Queue**

`
+-------------------------------------+
|  Sync Queue                         |
+-------------------------------------+
|                                     |
|  Pending uploads: 12                |
|  - 3 entities modified              |
|  - 8 new entities                   |
|  - 1 attachment (2.3 MB)            |
|                                     |
|  Conflicts waiting: 0               |
|                                     |
|  Will sync when online              |
|                                     |
|  [Force Sync]  [Clear Queue]        |
+-------------------------------------+
`

---

## Conflict Resolution

### Automatic Resolution

Most conflicts resolve automatically:

| Scenario | Resolution |
|----------|------------|
| Different fields edited | Both changes kept |
| Same field, non-overlapping edits | Smart merge |
| One edit deletes, other modifies | Modified version wins |
| Simultaneous new entities | Both kept, may need deduplication |

### Manual Conflict Resolution

When automatic resolution fails:

`
+-------------------------------------+
|  Sync Conflict                      |
+-------------------------------------+
|                                     |
|  Entity: Project Alpha              |
|                                     |
|  Conflict: Description field        |
|                                     |
|  Your version (Laptop):             |
|  [Your text here...]                |
|                                     |
|  Server version (Phone):            |
|  [Phone edited text...]             |
|                                     |
|  [Keep Mine]  [Keep Server]         |
|  [Merge Both] [View Diff]           |
+-------------------------------------+
`

Choose:
- **Keep Mine** - Use your device is version
- **Keep Server** - Use the other version
- **Merge Both** - Manually combine
- **View Diff** - See exact differences

### Conflict Notifications

Get notified of conflicts:

- **In-app badge** on sync icon
- **Email digest** (configurable)
- **Notification center** on mobile

**Settings** -> **Notifications** -> **Sync Conflicts**

### Best Practices to Avoid Conflicts

1. **Sync before switching devices**
2. **Work on different entities** when possible
3. **Use entity locking** for major edits
4. **Enable auto-sync** to stay current

---

## Security and Encryption

### Encryption Standards

Your data is protected with:

| Layer | Encryption |
|-------|------------|
| **In transit** | TLS 1.3 |
| **At rest** | AES-256 |
| **Local database** | SQLCipher (AES-256) |
| **Backups** | AES-256 with unique keys |

### End-to-End Encryption (Optional)

For maximum security, enable E2EE:

1. **Settings** -> **Sync** -> **Security**
2. **Enable End-to-End Encryption**
3. **Create encryption password**
4. **Save recovery key** securely

`
+-------------------------------------+
|  End-to-End Encryption              |
+-------------------------------------+
|                                     |
|  Status: Enabled                    |
|                                     |
|  Recovery Key:                      |
|  TRF-RECOVER-XXXX-XXXX-XXXX-XXXX   |
|                                     |
|  WARNING: Store this key safely!    |
|  If lost, your data cannot be       |
|  recovered.                         |
|                                     |
|  [Download Recovery Key]            |
|  [Regenerate Key]                   |
+-------------------------------------+
`

### Zero-Knowledge Architecture

With E2EE enabled:

- Truffle servers **cannot read your data**
- Only your devices can decrypt content
- Passwords and keys **never leave your devices**
- Recovery requires your recovery key

### Two-Factor Authentication (2FA)

Add an extra security layer:

1. **Settings** -> **Account** -> **Security**
2. **Enable 2FA**
3. **Scan QR code** with authenticator app
4. **Enter verification code**
5. **Save backup codes**

### Security Best Practices

1. **Use a strong sync password**
2. **Enable 2FA** on your account
3. **Store recovery keys offline**
4. **Regularly review connected devices**
5. **Remove unused devices** promptly

---

## Sync Settings

### Auto-Sync Configuration

Configure automatic syncing:

**Settings** -> **Sync** -> **Preferences**

`
+-------------------------------------+
|  Sync Preferences                   |
+-------------------------------------+
|                                     |
|  Auto-sync:                         |
|  (*) When changes are made          |
|  ( ) Every 5 minutes                |
|  ( ) Every 15 minutes               |
|  ( ) Manual only                    |
|                                     |
|  Sync on metered connections:       |
|  [ ] Yes (may incur data charges)   |
|                                     |
|  Sync attachments:                  |
|  (*) Yes, all attachments           |
|  ( ) Yes, under 10 MB only          |
|  ( ) On WiFi only                   |
|                                     |
|  [Save Settings]                    |
+-------------------------------------+
`

### Bandwidth Management

Control data usage:

| Setting | Description |
|---------|-------------|
| **Smart sync** | Sync summaries first, details on demand |
| **Attachment limits** | Skip large files on mobile data |
| **Compression** | Reduce data usage by ~60% |
| **Delta sync** | Only transfer changed parts |

### Selective Sync

Choose which content syncs to each device:

1. **Settings** -> **Sync** -> **Selective Sync**
2. **Uncheck items** you do not want on this device
3. Changes apply on next sync

Useful for:
- Limited storage devices
- Work/personal separation
- Reducing mobile data usage

---

## Troubleshooting Sync Issues

### Sync Not Working

If sync fails:

1. **Check internet connection**
2. **Sign out and sign back in**
3. **Restart Truffle**
4. **Check sync status** for error messages

### Slow Sync Performance

If syncing is slow:

1. **Reduce attachment size** in settings
2. **Enable compression** in preferences
3. **Use selective sync** to reduce data
4. **Check for large files** in queue

### Storage Full

If you exceed your storage limit:

1. **Delete old attachments**
2. **Archive unused entities**
3. **Upgrade your plan** for more storage
4. **Use selective sync** on some devices

### Device Not Appearing

If a device does not show in the list:

1. **Verify both devices are signed in** to the same account
2. **Check sync is enabled** on both
3. **Force a manual sync** on both devices
4. **Restart Truffle** on both devices

### Data Discrepancies

If devices show different data:

1. **Force sync** on both devices
2. **Check for conflicts** in sync queue
3. **Verify selective sync** settings
4. **Check last sync times**

---

## Next Steps

Continue learning about Truffle:

- **[Building Your Knowledge Graph](./knowledge-graph.md)** - Create and organize content
- **[Collaboration Features](./collaboration.md)** - Work together with your team
- **[Frequently Asked Questions](./faq.md)** - Get answers to common questions
- **[Getting Started](./getting-started.md)** - Review the basics

---

*Last updated: 2026-04-10*
