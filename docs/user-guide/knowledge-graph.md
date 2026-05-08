# Building Your Knowledge Graph

## Overview

This guide covers everything you need to know about creating, organizing, and visualizing knowledge in Truffle. Learn how to turn scattered information into a connected web of insights.

[Screenshot: Full knowledge graph visualization showing hundreds of connected nodes]

---

## What is a Knowledge Graph?

A **knowledge graph** is a way of organizing information that focuses on **relationships** rather than just individual items.

### Traditional Approach vs. Knowledge Graph

**Traditional (Folder-based):**
`
Documents/
├── Projects/
│   └── Project A.docx
├── Notes/
│   └── Meeting Notes.txt
└── Research/
    └── Article.pdf
`

**Knowledge Graph (Truffle):**
`
Project A --implements--> Knowledge Graph Technology
    │                           │
    └──discussed in------> Meeting Notes
                                │
                            references
                                │
                                ↓
                           Article.pdf
`

### Why Knowledge Graphs?

| Benefit | Description |
|---------|-------------|
| **Discover connections** | Find relationships you did not know existed |
| **Navigate intuitively** | Follow links rather than digging through folders |
| **Capture context** | See why things are related, not just that they are |
| **Scale gracefully** | Add complexity without losing organization |
| **Enable serendipity** | Stumble across relevant information naturally |

---

## Creating Your First Entity

### What is an Entity?

An **entity** is any distinct item in your knowledge graph - a person, project, concept, document, or anything you want to track.

### Creating an Entity

#### Method 1: Quick Create

1. **Press Ctrl/Cmd + N** or **Click the + button**
2. **Select New Entity**
3. Fill in the basic information:
4. **Click Save**

[Screenshot: Entity creation form with fields]

#### Method 2: From Selection

1. **Highlight text** in any entity description
2. **Right-click** → **Create Entity from Selection**
3. The new entity is created and automatically linked

[Screenshot: Context menu showing Create Entity from Selection]

#### Method 3: Bulk Import

For multiple entities at once:

1. **File** → **Import** → **CSV/JSON**
2. Map your columns to entity fields
3. **Click Import**

### Entity Types

Truffle comes with built-in types:

| Type | Use For | Example Fields |
|------|---------|----------------|
| **Person** | Team members, contacts | Email, Role, Organization |
| **Project** | Work initiatives | Status, Deadline, Priority |
| **Concept** | Ideas, topics | Definition, Category |
| **Document** | Files, notes | URL, Format, Size |
| **Event** | Meetings, milestones | Date, Location, Attendees |
| **Task** | Action items | Due date, Assignee, Status |
| **Custom** | Anything else | You define the fields |

### Entity Fields

Each entity can have multiple fields with Markdown support.

---

## Linking Entities with Relationships

### What are Relationships?

**Relationships** are the connections between entities. They transform a collection of items into a knowledge graph.

### Creating Relationships

#### Method 1: From Entity View

1. **Open an entity** (e.g., Project Alpha)
2. **Click Add Relationship**
3. Configure the connection:
   - From: Project Alpha
   - Relationship Type: requires
   - To: Customer Research Survey
4. **Click Add Link**

#### Method 2: Drag and Drop (Graph View)

1. **Switch to Graph view**
2. **Drag** from one entity node to another
3. **Select relationship type** from the popup

[Screenshot: Graph view showing drag-to-connect action]

#### Method 3: Using @ Mentions

In any text field, **type @** to reference another entity:

`markdown
This project is based on the @Customer Research Survey 
conducted last quarter.
`

Truffle automatically creates mentions relationships.

### Relationship Types

Common relationship types:

| Type | Direction | Use When |
|------|-----------|----------|
| **related to** | ↔ | General connection |
| **part of** | → | Component relationship |
| **requires** | → | Dependency |
| **depends on** | → | Strong dependency |
| **created by** | → | Authorship |
| **references** | → | Citation/link |
| **implements** | → | Realization |
| **blocks** | → | Obstruction |
| **supersedes** | → | Replacement |

### Bidirectional Relationships

Some relationships work both ways:

- **Alice** ←colleagues→ **Bob**
- **Feature A** ←related to→ **Feature B**

---

## Searching the Graph

### Quick Search

**Press Ctrl/Cmd + K** or **Click the search bar**:

Type to search across:
- Entity names
- Descriptions
- Tags
- Relationship notes
- File contents (if indexed)

### Advanced Search

Click the **filter icon** for advanced options with filters for:
- Entity types
- Tags
- Creation date
- Relationship status

### Search Operators

Use special syntax for precise results:

| Operator | Example | Finds |
|----------|---------|-------|
| quotes | customer feedback | Exact phrase |
| type: | type:project | Specific entity type |
| tag: | tag:urgent | Entities with tag |
| created: | created:2026-03 | Creation date |
| linked: | linked:Project Alpha | Entities with links |
| AND | customer AND feedback | Both terms |
| OR | project OR initiative | Either term |
| - | meeting -recurring | Exclude term |

### Saved Searches

Save frequently used searches:

1. **Build your search** with filters
2. **Click Save Search**
3. **Name it** (e.g., Weekly Review Items)
4. Access from the sidebar

---

## Visualizing Connections

### Graph View

The **Graph View** shows your knowledge as an interactive network:

[Screenshot: Graph view with 20+ nodes and various connection types]

#### Navigating the Graph

| Action | How To |
|--------|--------|
| **Pan** | Click and drag empty space |
| **Zoom** | Mouse wheel or pinch gesture |
| **Focus** | Double-click a node |
| **Select** | Click a node |
| **Expand** | Click the + on a node to show connections |
| **Rearrange** | Drag nodes to new positions |

#### Graph Layouts

Choose different layouts for different perspectives:

- **Force-directed** - Organic clustering based on connections
- **Hierarchical** - Top-down tree structure
- **Circular** - Radial arrangement
- **Grid** - Ordered, grid-based layout
- **Timeline** - Time-based horizontal layout

#### Filtering the Graph

Reduce complexity with filters:

1. **Entity Type Filter** - Show only Projects, hide Concepts
2. **Relationship Filter** - Show only requires relationships
3. **Time Filter** - Show entities created this month
4. **Tag Filter** - Show only entities tagged priority

### Path Finding

Find connections between two entities:

1. **Select first entity**
2. **Hold Shift** and **click second entity**
3. **Click Find Path**
4. Truffle highlights the connection path

[Screenshot: Graph showing highlighted path between two distant nodes]

### Neighborhood View

See everything connected to an entity:

1. **Right-click an entity**
2. **Select Show Neighborhood**
3. View the 1-hop or 2-hop network around that entity

---

## Exporting Data

### Export Formats

Truffle supports multiple export formats:

| Format | Extension | Best For |
|--------|-----------|----------|
| **JSON** | .json | Full data with relationships |
| **CSV** | .csv | Spreadsheet analysis |
| **GraphML** | .graphml | Import to other graph tools |
| **Markdown** | .md | Documentation, notes |
| **PDF** | .pdf | Reports, sharing |
| **PNG/SVG** | .png/.svg | Graph visualizations |

### Exporting Entities

1. **Select entities** (checkboxes or Ctrl+Click)
2. **File** → **Export** → **Selected Entities**
3. **Choose format**
4. **Click Export**

### Exporting the Graph

To export the visualization:

1. **Switch to Graph view**
2. **Click the camera icon** in the toolbar
3. Choose:
   - **PNG** - For presentations
   - **SVG** - For editing in design tools
   - **Interactive HTML** - For web sharing

### Scheduled Exports

Set up automatic backups:

1. **Settings** → **Export**
2. **Enable scheduled exports**
3. Choose frequency: Daily, Weekly, or Monthly
4. Select destination folder

---

## Tips for Effective Knowledge Graphs

### Design Principles

1. **Start with questions** - What do you want to know?
2. **Link liberally** - When in doubt, create a relationship
3. **Use consistent naming** - Customer Feedback not feedback from customers
4. **Review and refine** - Periodically clean up orphaned entities
5. **Tag strategically** - Tags should group related items

### Anti-Patterns to Avoid

| Do Not | Do Instead |
|----------|---------------|
| Create entities for everything | Group related items |
| Use generic relationship types | Be specific about connections |
| Let orphans accumulate | Connect or delete unused entities |
| Duplicate information | Link to the source |
| Ignore the graph view | Use it to discover connections |

### Scaling Your Graph

As your graph grows:

- Use **filters** to focus on relevant subsets
- Create **saved views** for different contexts
- Use **hierarchical relationships** (part of) for organization
- Regularly **archive** outdated information

---

## Next Steps

Now that you understand knowledge graphs:

- **[Collaboration Guide](./collaboration.md)** - Share and collaborate on your graph
- **[Sync Guide](./sync.md)** - Access your knowledge across devices
- **[FAQ](./faq.md)** - Get answers to common questions
- **[Getting Started](./getting-started.md)** - Review the basics

---

*Last updated: 2026-04-10*
