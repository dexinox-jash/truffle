# Collaboration Features

## Overview

Truffle makes teamwork seamless with real-time collaboration features. Share knowledge with your team, discuss ideas, and work together on building comprehensive knowledge graphs.

[Screenshot: Multi-user collaboration view showing presence indicators and live cursors]

---

## Real-Time Collaboration Features

### What is Real-Time Collaboration?

When multiple team members work in the same workspace, changes appear **instantly** for everyone. No refreshing, no version conflicts - just smooth, synchronized work.

### Features at a Glance

| Feature | Description |
|---------|-------------|
| **Live editing** | See changes as they happen |
| **Presence indicators** | Know who is online and viewing |
| **Live cursors** | See where others are working |
| **Comments** | Discuss without disrupting flow |
| **Activity feed** | Track what has changed |
| **Conflict resolution** | Automatic handling of simultaneous edits |

---

## Sharing Entities

### Share with Individuals

To share a specific entity with someone:

1. **Open the entity** you want to share
2. **Click the Share button** in the top right
3. Enter the person is email address
4. **Set permissions** (View, Comment, or Edit)
5. **Click Send Invite**

`
+-------------------------------------+
|  Share Project Alpha                |
+-------------------------------------+
|                                     |
|  Share with:                        |
|  [alice@example.com           ]     |
|                                     |
|  Permission level:                  |
|  ( ) Can view                       |
|  (*) Can comment                    |
|  ( ) Can edit                       |
|                                     |
|  Message (optional):                |
|  [Please review the project        |
|   timeline and add your feedback]   |
|                                     |
|  [Cancel]          [Send Invite]    |
+-------------------------------------+
`

[Screenshot: Share dialog with email input and permission options]

### Share with Teams

To share with an entire team:

1. **Click Share**
2. **Select the Team tab**
3. Choose from your existing teams
4. **Select permission level**
5. **Click Share with Team**

### Create Shareable Links

Generate a link for easy sharing:

1. **Click Share** -> **Create Link**
2. **Choose access level:**
   - Anyone with the link
   - Organization members only
   - Specific people only
3. **Set expiration** (optional)
4. **Click Generate Link**
5. **Copy and share** the URL

`
Link created: https://truffle.dev/s/abc123xyz
Expires: 7 days
Permissions: Can view
`

### Managing Access

Review and modify who has access:

1. **Open the entity**
2. **Click Share** -> **Manage Access**
3. See everyone with access
4. **Change permissions** or **Revoke access** as needed

---

## Comments and Discussions

### Adding Comments

Comment on any entity or specific text:

#### Entity-Level Comments

1. **Open the entity**
2. **Click the Comments icon** in the sidebar
3. **Type your comment**
4. **Click Post**

`
+-------------------------------------+
|  Comments                           |
+-------------------------------------+
|                                     |
|  Alice Chen - 2 hours ago           |
|  Should we update the timeline      |
|  to reflect the new deadline?       |
|                                     |
|  Reply...                           |
|                                     |
|  Bob Smith - 5 hours ago            |
|  Great work on the research!        |
|                                     |
|  3 replies                          |
|                                     |
|  [Add a comment...        ] [Post]  |
+-------------------------------------+
`

#### Inline Comments

Comment on specific text:

1. **Select text** in the entity description
2. **Right-click** -> **Add Comment**
3. **Type your comment**
4. **Click Post**

[Screenshot: Inline comment highlight on selected text]

### Resolving Comments

When a discussion is complete:

1. **Hover over the comment**
2. **Click Resolve**
3. The comment is archived but remains visible in history

### Mentioning Users

Get someone is attention by mentioning them:

- **Type @** followed by their name
- They will receive a notification
- Example: @Alice Chen can you review this section?

### Comment Notifications

Choose when to be notified:

- **All comments** - Every comment on entities you can access
- **Mentions only** - Only when someone mentions you
- **None** - No comment notifications

**Settings** -> **Notifications** -> **Comments**

---

## Presence Indicators

### User Presence

See who is active in real-time:

`
+-------------------------------------+
|  Project Alpha              Online  |
|                              -----  |
|  Currently viewing:                 |
|  Alice Chen (editing)               |
|  Bob Smith (viewing)                |
|  You                                |
+-------------------------------------+
`

### Status Indicators

| Indicator | Meaning |
|-----------|---------|
| Green dot | Online and active |
| Yellow dot | Online but idle |
| Gray dot | Offline |
| Pencil icon | Currently editing |
| Eye icon | Currently viewing |

### Live Cursors

See exactly where others are working:

- **Colored cursors** show each user is position
- **Name labels** identify who is who
- **Selection highlights** show what text others have selected

[Screenshot: Multiple colored cursors in an entity editor]

---

## Conflict Resolution

### Automatic Merging

Truffle automatically handles most simultaneous edits:

- **Different fields edited** - Both changes are saved
- **Same field, different parts** - Changes merged intelligently
- **Appending text** - Both additions preserved

### Conflict Detection

When automatic merging is not possible:

`
+-------------------------------------+
|  Conflict Detected                  |
+-------------------------------------+
|                                     |
|  Alice edited the description       |
|  while you were making changes.     |
|                                     |
|  Your version:                      |
|  [Your text here...]                |
|                                     |
|  Alice is version:                  |
|  [Alice is text here...]           |
|                                     |
|  [Keep Mine]  [Keep Alice is]      |
|  [Merge Both] [View Diff]           |
+-------------------------------------+
`

### Resolving Conflicts

Choose how to handle conflicts:

- **Keep Mine** - Discard other changes
- **Keep Theirs** - Discard your changes
- **Merge Both** - Combine both versions manually
- **View Diff** - See exactly what changed

### Best Practices to Avoid Conflicts

1. **Work on different entities** when possible
2. **Use comments** to coordinate on shared items
3. **Lock entities** for major edits (see below)
4. **Save frequently** to reduce conflict windows

### Entity Locking

Temporarily prevent others from editing:

1. **Open the entity**
2. **Click Lock** in the toolbar
3. **Set duration** (15 min, 1 hour, or until unlocked)
4. Others see a lock icon and your name
5. **Automatically unlocks** when time expires or you finish

`
+-------------------------------------+
|  Locked by You (12 min remaining)   |
|                                     |
|  [Unlock Early]                     |
+-------------------------------------+
`

---

## Permissions and Access Control

### Permission Levels

| Level | Can Do |
|-------|--------|
| **View** | Read content, see relationships |
| **Comment** | View + add comments |
| **Edit** | Comment + modify content |
| **Admin** | Edit + share, delete, change permissions |
| **Owner** | Full control including transfer ownership |

### Workspace Permissions

Control access at the workspace level:

**Settings** -> **Workspace** -> **Access Control**

`
+-------------------------------------+
|  Workspace Access                   |
+-------------------------------------+
|                                     |
|  Who can join this workspace?       |
|  ( ) Anyone with the link           |
|  (*) Organization members           |
|  ( ) Invite only                    |
|                                     |
|  Default permission for new members |
|  [Can edit v]                       |
|                                     |
|  Members:                           |
|  Alice Chen - Owner                 |
|  Bob Smith - Admin                  |
|  Carol Jones - Edit                 |
+-------------------------------------+
`

### Role-Based Access

Assign roles to team members:

| Role | Permissions |
|------|-------------|
| **Owner** | Full control |
| **Admin** | Manage members, settings |
| **Editor** | Create and edit content |
| **Viewer** | Read-only access |
| **Guest** | Limited access to specific items |

### Entity-Level Permissions

Override workspace defaults for specific entities:

1. **Open entity** -> **Share** -> **Advanced**
2. **Set specific permissions**
3. **Add exceptions** for specific users

Example: A project visible to all workspace members, but only editable by the project team.

### Audit Log

Track all permission changes:

**Settings** -> **Security** -> **Audit Log**

`
2026-04-10 14:32 - Alice Chen granted Bob Smith edit access to Project Alpha
2026-04-10 11:15 - Carol Jones is access revoked from Customer Research
2026-04-09 16:45 - Workspace default changed from Edit to View
`

---

## Collaboration Best Practices

### Communication Guidelines

1. **Use comments for questions** - Keep discussions attached to content
2. **@mention relevant people** - Ensure the right people see comments
3. **Resolve old comments** - Keep comment sections clean
4. **Be specific in mentions** - @someone about a specific section

### Workflow Tips

1. **Coordinate on major changes** - Use comments to plan big edits
2. **Lock for significant rewrites** - Prevent conflicts during major work
3. **Share early and often** - Get feedback sooner rather than later
4. **Use tags for handoffs** - Tag items as Ready for Review

### Notification Management

Avoid notification overload:

- **Customize per workspace** - Different settings for different projects
- **Use Do Not Disturb** - Pause notifications during focus time
- **Batch email notifications** - Get summaries instead of instant emails
- **Mute specific threads** - Stop following comments on resolved items

---

## Troubleshooting Collaboration Issues

### Changes Not Appearing

If you do not see others is changes:

1. **Check your internet connection**
2. **Refresh the page** (rarely needed)
3. **Verify you have the right permissions**
4. **Check if the entity is locked**

### Cannot Share

If sharing fails:

1. **Verify you have Admin or Owner permissions**
2. **Check if the user already has access**
3. **Ensure email address is correct**
4. **Check workspace sharing settings**

### Comment Notifications Not Working

1. **Check notification settings** in your profile
2. **Verify email is confirmed**
3. **Check spam/junk folders**
4. **Test with a self-mention**

---

## Next Steps

Learn more about working with Truffle:

- **[Building Your Knowledge Graph](./knowledge-graph.md)** - Create and organize content
- **[Sync and Multi-Device](./sync.md)** - Access from anywhere
- **[Frequently Asked Questions](./faq.md)** - Get answers to common questions
- **[Getting Started](./getting-started.md)** - Review the basics

---

*Last updated: 2026-04-10*
