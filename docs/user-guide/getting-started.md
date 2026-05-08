# Getting Started with Truffle

## Overview

Welcome to Truffle! This guide will help you set up the Truffle knowledge graph platform and create your first knowledge connections in under 5 minutes.

**Truffle** is a powerful desktop application for building, visualizing, and sharing knowledge graphs. Connect ideas, collaborate with your team, and discover insights through interactive visualizations.

[Screenshot: Truffle main dashboard interface]

---

## What is Truffle?

Truffle is a **knowledge graph platform** that helps you:

- **Capture knowledge** - Store entities, concepts, and information in a structured way
- **Connect ideas** - Link related items to build a web of knowledge
- **Visualize relationships** - See connections through interactive graph views
- **Collaborate in real-time** - Work together with your team seamlessly
- **Access anywhere** - Sync across all your devices securely

Whether you are a researcher organizing sources, a team documenting processes, or a student connecting course materials, Truffle adapts to your workflow.

---

## System Requirements

### Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| **OS** | Windows 10, macOS 11+, or Linux (Ubuntu 20.04+) |
| **RAM** | 4 GB |
| **Storage** | 500 MB free space |
| **Display** | 1280x720 resolution |
| **Internet** | Required for sync and collaboration |

### Recommended Specifications

| Component | Recommendation |
|-----------|----------------|
| **OS** | Windows 11, macOS 14+, or Linux (Ubuntu 22.04+) |
| **RAM** | 8 GB |
| **Storage** | 2 GB free space (for large knowledge graphs) |
| **Display** | 1920x1080 resolution |
| **GPU** | Dedicated graphics for large graphs |

---

## Installation

### Step 1: Download Truffle

1. Visit **[truffle.dev/download](https://truffle.dev/download)**
2. Select your operating system
3. **Click Download** to save the installer

[Screenshot: Download page showing OS selection]

### Step 2: Install the Application

#### Windows
1. Run Truffle-Setup.exe
2. Follow the installation wizard
3. **Click Finish** to launch Truffle

#### macOS
1. Open Truffle.dmg
2. Drag Truffle to your **Applications** folder
3. Open Truffle from Launchpad or Applications

#### Linux
`ash
# For .AppImage
chmod +x Truffle.AppImage
./Truffle.AppImage

# For .deb (Ubuntu/Debian)
sudo dpkg -i truffle.deb
sudo apt-get install -f  # Fix dependencies

# For .rpm (Fedora/RHEL)
sudo rpm -i truffle.rpm
`

### Step 3: First Launch

On first launch, Truffle will:
1. Create a local database in your user folder
2. Set up default configuration
3. Display the welcome screen

---

## First-Time Setup

### Create Your Account

When you first open Truffle, you will see the setup wizard:

[Screenshot: Account creation screen]

1. **Enter your name** - This will be displayed to collaborators
2. **Add your email** - Used for sync and notifications
3. **Choose a profile picture** (optional)
4. **Click Continue**

### Set Up Your Workspace

Your workspace is where all your knowledge lives:

1. **Choose a workspace name** (e.g., Personal, Work, Research)
2. **Select a theme** - Light, Dark, or Auto
3. **Click Create Workspace**

[Screenshot: Workspace creation with theme selection]

### Configure Sync (Optional)

To access your knowledge across devices:

1. **Click Set Up Sync** on the welcome screen
2. **Sign in** or **create an account** on truffle.cloud
3. **Click Enable Sync**

> **Tip:** You can skip this step and enable sync later from Settings.

---

## Quick Start Tutorial (5 Minutes)

Let us create your first knowledge graph!

### Step 1: Create Your First Entity (1 minute)

Entities are the building blocks of your knowledge graph.

1. **Click the + button** in the top toolbar
2. **Select New Entity** from the dropdown
3. Fill in the details:
   - **Name:** Project Truffle
   - **Type:** Project
   - **Description:** A knowledge graph platform for organizing ideas
4. **Click Save**

[Screenshot: Entity creation form filled out]

**You have created your first entity!**

### Step 2: Add Another Entity (1 minute)

Let us add something related:

1. **Click +** then **New Entity** again
2. Enter:
   - **Name:** Knowledge Graph
   - **Type:** Concept
   - **Description:** A network of connected information nodes
3. **Click Save**

### Step 3: Create a Relationship (1 minute)

Now let us connect these two entities:

1. **Click on Project Truffle** to open it
2. **Click Add Relationship**
3. Configure the relationship:
   - **To:** Knowledge Graph
   - **Type:** implements
   - **Description:** Truffle is built on knowledge graph technology
4. **Click Save**

[Screenshot: Relationship creation showing connection between entities]

### Step 4: Explore the Graph View (1 minute)

See your knowledge come to life:

1. **Click the Graph tab** in the top navigation
2. **Zoom in/out** using your mouse wheel or pinch gesture
3. **Drag nodes** to rearrange the view
4. **Click a node** to see its details

[Screenshot: Graph view showing connected nodes]

### Step 5: Search Your Knowledge (1 minute)

Find anything instantly:

1. **Click the search bar** at the top
2. **Type knowledge**
3. **See results** from both entity names and content
4. **Click on a result** to navigate to it

[Screenshot: Search results showing matching entities]

---

## Basic Navigation

### Main Interface

`
+-------------------------------------------------------------+
|  Search...              [+]  [Bell]  [Person]  [Gear]      |
+----------+--------------------------------------------------+
|          |                                                  |
| ENTITIES |         MAIN CONTENT AREA                        |
|          |                                                  |
|  All     |    [List] [Table] [Graph] [Board]                |
|  Fav     |                                                  |
|  Rec     |    - Project Truffle                             |
|  Tags    |    - Knowledge Graph                             |
|          |                                                  |
|          |                                                  |
+----------+--------------------------------------------------+
|  Quick actions | Sync status: OK | 2 entities              |
+-------------------------------------------------------------+
`

### Sidebar Navigation

| Icon | Section | Description |
|------|---------|-------------|
| Folder | **All Entities** | Browse everything in your workspace |
| Star | **Favorites** | Quickly access starred items |
| Clock | **Recent** | Recently viewed or edited entities |
| Tag | **Tags** | Filter by category labels |
| Link | **Relationships** | View all connections |
| Trash | **Trash** | Deleted items (recoverable for 30 days) |

### View Modes

Switch between different ways to see your knowledge:

- **List View** - Simple, scannable list of entities
- **Table View** - Sortable columns with details
- **Graph View** - Interactive visualization of connections
- **Board View** - Kanban-style organization

**Click the view icons** in the toolbar to switch modes.

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl/Cmd + N | Create new entity |
| Ctrl/Cmd + K | Open quick search |
| Ctrl/Cmd + F | Find in current view |
| Ctrl/Cmd + S | Save current changes |
| Esc | Close modal/dialog |
| ? | Show all shortcuts |

---

## Tips for New Users

### Getting Started Tips

- **Start small** - Begin with 5-10 entities and grow organically
- **Use descriptive names** - Make entities easy to find later
- **Link liberally** - Connections are what make knowledge graphs powerful
- **Favorite often** - Star important entities for quick access
- **Tag consistently** - Use tags to categorize and filter

### Common First Actions

1. **Import existing notes** - Use File -> Import to bring in Markdown, JSON, or CSV
2. **Set up templates** - Create templates for common entity types
3. **Invite collaborators** - Share your workspace for team collaboration
4. **Enable sync** - Access your knowledge across devices

---

## Next Steps

Now that you have got the basics, explore more features:

### Learn More

- **[Building Your Knowledge Graph](./knowledge-graph.md)** - Deep dive into entities, relationships, and visualization
- **[Collaboration Features](./collaboration.md)** - Work together with your team in real-time
- **[Sync and Multi-Device](./sync.md)** - Access your knowledge anywhere
- **[Frequently Asked Questions](./faq.md)** - Find answers to common questions

### Quick Actions

- **Create 5 more entities** - Practice building your graph
- **Try all view modes** - Find what works best for you
- **Invite a friend** - Collaboration is better together
- **Check the FAQ** - Get answers to common questions

---

## Getting Help

Need assistance? We are here to help:

- **Documentation:** [docs.truffle.dev](https://docs.truffle.dev)
- **Community:** [community.truffle.dev](https://community.truffle.dev)
- **Email:** support@truffle.dev
- **Twitter:** [@trufflekg](https://twitter.com/trufflekg)

---

*Last updated: 2026-04-10*
