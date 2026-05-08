# Frequently Asked Questions

## Overview

Find answers to common questions about Truffle, troubleshooting tips, and solutions to issues you might encounter.

---

## General Questions

### What is Truffle?

Truffle is a desktop application for building, visualizing, and sharing knowledge graphs. It helps you connect ideas, organize information, and collaborate with others through an interactive graph-based interface.

### Who is Truffle for?

Truffle is designed for:

- **Researchers** organizing sources and connections
- **Teams** documenting processes and projects
- **Students** connecting course materials
- **Writers** managing complex storylines
- **Knowledge workers** building personal wikis
- **Anyone** who wants to connect ideas visually

### Is Truffle free?

Truffle offers:

| Plan | Cost | Features |
|------|------|----------|
| **Free** |  | Local use, 1 workspace, basic features |
| **Pro** | /mo | Sync, unlimited workspaces, 10 GB storage |
| **Team** | /user/mo | Collaboration, admin controls, 100 GB storage |
| **Enterprise** | Custom | SSO, audit logs, dedicated support |

### What platforms does Truffle support?

Truffle is available for:

- Windows 10 and 11
- macOS 11 (Big Sur) and later
- Linux (Ubuntu 20.04+, Fedora 34+, Debian 11+)

Mobile apps for iOS and Android are in development.

### Do I need an internet connection?

No. Truffle works **completely offline** for local use. You only need internet for:

- Syncing across devices
- Collaboration features
- Software updates
- Cloud backups

---

## Getting Started

### How do I install Truffle?

1. Download from [truffle.dev/download](https://truffle.dev/download)
2. Run the installer for your platform
3. Launch Truffle and follow the setup wizard

See [Getting Started Guide](./getting-started.md) for detailed instructions.

### How do I create my first entity?

1. **Click the + button** or press **Ctrl/Cmd + N**
2. Select **New Entity**
3. Fill in the name, type, and description
4. **Click Save**

Check out the [Knowledge Graph Guide](./knowledge-graph.md) for more details.

### Can I import my existing notes?

Yes! Truffle supports importing from:

- **Markdown files** (.md)
- **CSV files** for bulk entity import
- **JSON** (Truffle format or custom mapping)
- **Evernote** (.enex)
- **Notion** (via export)
- **Roam Research** (via JSON)

**File** -> **Import** -> **Select format**

### What are the system requirements?

**Minimum:**
- 4 GB RAM
- 500 MB free storage
- Windows 10 / macOS 11 / Linux

**Recommended:**
- 8 GB RAM
- 2 GB free storage
- Dedicated graphics for large graphs

---

## Knowledge Graph Questions

### What is a knowledge graph?

A knowledge graph is a way of organizing information that focuses on **relationships** between items, not just the items themselves. Instead of folders, you have entities connected by meaningful links.

Learn more in the [Knowledge Graph Guide](./knowledge-graph.md).

### How many entities can I create?

| Plan | Entity Limit |
|------|--------------|
| Free | 1,000 entities |
| Pro | Unlimited |
| Team | Unlimited |

Performance may vary with very large graphs (10,000+ entities).

### Can I change an entity type after creation?

Yes:

1. **Open the entity**
2. **Click the type dropdown**
3. **Select new type**
4. **Map fields** if the types have different structures

### How do I find related entities?

Multiple ways:

1. **Graph View** - See visual connections
2. **Relationship panel** - List all connections
3. **Path finder** - Find routes between entities
4. **Search with linked:** filter - Find entities with specific relationships

### Can I merge duplicate entities?

Yes:

1. **Select both entities** (Ctrl/Cmd + Click)
2. **Right-click** -> **Merge Entities**
3. **Choose which properties to keep**
4. **Confirm merge**

All relationships are preserved and redirected to the merged entity.

### What file formats can I attach?

Truffle supports most file types:

- Documents: PDF, DOCX, TXT, MD
- Images: PNG, JPG, GIF, SVG
- Spreadsheets: CSV, XLSX
- Code files: Most programming languages
- Archives: ZIP (contents indexed)

Size limit: 100 MB per file (Pro), 10 MB (Free)

---

## Sync and Devices

### How do I sync across devices?

1. **Create a Truffle account** (Pro plan required)
2. **Sign in** on each device
3. Your data syncs automatically

See [Sync Guide](./sync.md) for detailed setup.

### Is my data secure during sync?

Yes. All data is encrypted:

- **In transit:** TLS 1.3
- **At rest:** AES-256
- **Local storage:** SQLCipher encryption

Optional end-to-end encryption available for maximum security.

### What happens if I edit offline?

Changes are saved locally and sync automatically when you reconnect. If conflicts occur, Truffle will notify you and help resolve them.

### Can I use Truffle on multiple computers?

Yes! Sign in with the same account on all your computers. Truffle Pro supports unlimited devices.

### How do I remove a device from my account?

**Settings** -> **Sync** -> **Devices** -> **Click Remove** next to the device.

### Will sync use a lot of data?

No. Truffle uses:

- Delta sync (only changes transferred)
- Compression (~60% reduction)
- Smart sync (summaries first)

Typical usage: 10-50 MB per month for regular use.

---

## Collaboration Questions

### How do I share an entity with my team?

1. **Open the entity**
2. **Click Share**
3. **Enter email addresses** or select team
4. **Set permission level** (View/Comment/Edit)
5. **Click Send Invite**

See [Collaboration Guide](./collaboration.md) for details.

### Can I work on the same entity simultaneously?

Yes! Truffle supports real-time collaboration. Changes appear instantly for all users. Presence indicators show who is online and what they are viewing.

### What permissions can I set?

| Permission | Can Do |
|------------|--------|
| **View** | Read content |
| **Comment** | View + add comments |
| **Edit** | Comment + modify content |
| **Admin** | Edit + share, delete |
| **Owner** | Full control |

### How do comments work?

- **Entity comments** - General discussion attached to an entity
- **Inline comments** - Comments on specific text selections
- **@mentions** - Notify specific users
- **Resolve** - Mark discussions as complete

### Can I prevent others from editing?

Yes, use **Entity Locking**:

1. **Open the entity**
2. **Click Lock** in toolbar
3. **Set duration**
4. Others see a lock icon while you work

---

## Troubleshooting

### Truffle won't start

**Solutions:**

1. **Check system requirements**
2. **Restart your computer**
3. **Reinstall Truffle** (data is preserved)
4. **Check for conflicting software** (antivirus, VPN)
5. **Run as administrator** (Windows)

### Database is locked error

This happens when multiple Truffle processes run:

1. **Close all Truffle windows**
2. **Check for zombie processes:**
   - Windows: Task Manager -> End Truffle tasks
   - macOS: Activity Monitor -> Quit Truffle
   - Linux: killall truffle
3. **Restart Truffle**

### Search is not finding results

**Try these steps:**

1. **Reindex search:** Settings -> Search -> Rebuild Index
2. **Check filters** - You may have active filters
3. **Try different keywords** - Use broader terms
4. **Verify content exists** - Check the entity manually

### Graph view is slow or laggy

For large graphs:

1. **Reduce graph depth** - Show fewer connection levels
2. **Apply filters** - Limit visible entity types
3. **Enable performance mode** - Settings -> Graph -> Performance
4. **Upgrade hardware** - Dedicated GPU helps significantly

### Sync is not working

**Check these items:**

1. **Internet connection** - Verify you are online
2. **Sign out and back in** - Refresh authentication
3. **Check sync status** - Look for error messages
4. **Firewall settings** - Allow Truffle network access
5. **VPN interference** - Try without VPN

### Forgot my password

1. **Click Forgot Password** on sign-in screen
2. **Enter your email**
3. **Check inbox** for reset link
4. **Create new password**

### How do I back up my data?

**Automatic backups:**
- Cloud sync (Pro plans)
- Scheduled exports

**Manual backup:**
1. **File** -> **Export** -> **All Data**
2. Choose format (JSON recommended)
3. Save to secure location

---

## Error Messages Explained

### Entity not found

**Cause:** The entity ID does not exist or you do not have access.

**Solution:**
- Check if the entity was deleted
- Verify you have permission to view it
- Try searching by name instead

### Conflict detected

**Cause:** Simultaneous edits from multiple users/devices.

**Solution:**
- Review both versions
- Choose which to keep or merge
- See [Conflict Resolution](./sync.md#conflict-resolution)

### Merge failed

**Cause:** Entities have incompatible data structures.

**Solution:**
- Manually copy important fields
- Delete one entity
- Recreate relationships as needed

### Sync quota exceeded

**Cause:** You have reached your plan's storage limit.

**Solution:**
- Delete old attachments
- Archive unused entities
- Upgrade to a higher plan

### Network timeout

**Cause:** Slow or interrupted internet connection.

**Solution:**
- Check internet connection
- Retry the operation
- Work offline until connection improves

---

## Performance Tips

### Speed up Truffle

1. **Limit active graphs** - Close unused graph views
2. **Use filters** - Show fewer entities at once
3. **Disable animations** - Settings -> Appearance -> Animations
4. **Clear cache** - Settings -> Advanced -> Clear Cache
5. **Upgrade hardware** - SSD and more RAM help significantly

### Optimize large knowledge graphs

| Graph Size | Recommendation |
|------------|----------------|
| 100-500 entities | Standard settings work fine |
| 500-2,000 entities | Use filters regularly |
| 2,000-5,000 entities | Enable performance mode |
| 5,000+ entities | Consider splitting into workspaces |

### Reduce memory usage

1. **Close unused entity tabs**
2. **Switch to List view** instead of Graph
3. **Limit search history** - Settings -> Search -> History Size
4. **Disable real-time sync** - Sync manually instead

---

## Data Backup and Recovery

### Automatic Backups

With Pro plans, Truffle automatically backs up:

- Daily cloud backups (30-day retention)
- Version history for entities (90 days)
- Deleted item recovery (30 days in Trash)

### Manual Export

Create your own backups:

1. **File** -> **Export** -> **All Data**
2. **Choose format:**
   - JSON (complete with relationships)
   - CSV (for spreadsheet analysis)
   - Markdown (for reading)
3. **Save** to external drive or cloud storage

### Restore from Backup

1. **File** -> **Import**
2. **Select backup file**
3. **Choose restore options:**
   - Merge with existing data
   - Replace all data
   - Import specific entities only

### Recover Deleted Items

1. **Open Trash** from sidebar
2. **Find the deleted entity**
3. **Click Restore**
4. Item returns to original location

**Note:** Items in Trash for 30+ days are permanently deleted.

### Export Specific Entities

1. **Select entities** (Ctrl/Cmd + Click)
2. **Right-click** -> **Export Selected**
3. **Choose format**
4. **Save**

---

## Account and Billing

### How do I change my plan?

**Settings** -> **Account** -> **Subscription** -> **Change Plan**

Changes take effect immediately. Prorated charges or credits applied.

### How do I cancel my subscription?

**Settings** -> **Account** -> **Subscription** -> **Cancel**

Your data remains accessible in read-only mode. Reactivate anytime.

### Can I get a refund?

Refunds are available within 14 days of purchase. Contact support@truffle.dev.

### How do I update payment information?

**Settings** -> **Account** -> **Billing** -> **Update Payment Method**

### Do you offer educational discounts?

Yes! Students and educators get 50% off Pro plans. Email edu@truffle.dev with your .edu address.

---

## Getting Help

### Where can I find more documentation?

- **User Guides:** docs.truffle.dev/guides
- **API Reference:** docs.truffle.dev/api
- **Video Tutorials:** youtube.com/trufflekg

### How do I report a bug?

1. **Help** -> **Report Issue**
2. **Describe the problem**
3. **Include steps to reproduce**
4. **Attach logs** if possible

Or email: bugs@truffle.dev

### How do I request a feature?

Submit ideas at:

- **In-app:** Help -> **Request Feature**
- **Online:** [feedback.truffle.dev](https://feedback.truffle.dev)
- **Email:** features@truffle.dev

### Community support

- **Community Forum:** [community.truffle.dev](https://community.truffle.dev)
- **Discord:** [discord.gg/truffle](https://discord.gg/truffle)
- **Reddit:** r/trufflekg

### Contact support

- **Email:** support@truffle.dev
- **Response time:** Within 24 hours (Pro), 72 hours (Free)
- **Live chat:** Available for Enterprise plans

---

## Quick Reference

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl/Cmd + N | New entity |
| Ctrl/Cmd + K | Quick search |
| Ctrl/Cmd + F | Find in view |
| Ctrl/Cmd + S | Save |
| Ctrl/Cmd + Shift + S | Sync now |
| Esc | Close modal |
| ? | Show shortcuts |

### Common Operations

| Task | How To |
|------|--------|
| Create entity | + button or Ctrl+N |
| Add relationship | Drag in graph or Add Relationship button |
| Search | Ctrl+K or click search bar |
| Share | Share button in entity view |
| Export | File -> Export |
| Import | File -> Import |
| Settings | Gear icon or Ctrl+, |

---

*Last updated: 2026-04-10*

*Did not find your question? Contact us at support@truffle.dev*
