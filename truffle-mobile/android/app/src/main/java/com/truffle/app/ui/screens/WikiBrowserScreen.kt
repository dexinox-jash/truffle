package com.truffle.app.ui.screens

import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import com.truffle.app.ui.components.*
import java.util.*

/**
 * WikiBrowserScreen - Read-Only Wiki Browser for Android
 * 
 * SPEC v2.0 COMPLIANCE:
 * - Read-only wiki browser (no editing on mobile)
 * - Full editing functionality on desktop only
 * - Navigate, search, view screenshots
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun WikiBrowserScreen() {
    val viewModel = remember { WikiBrowserViewModel() }
    var showNodeList by remember { mutableStateOf(false) }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(viewModel.currentNode?.title ?: "Wiki") },
                navigationIcon = {
                    IconButton(onClick = { showNodeList = true }) {
                        Icon(Icons.Default.List, contentDescription = "All Pages")
                    }
                },
                actions = {
                    IconButton(
                        onClick = { viewModel.navigateBack() },
                        enabled = viewModel.canGoBack
                    ) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                    IconButton(
                        onClick = { viewModel.navigateForward() },
                        enabled = viewModel.canGoForward
                    ) {
                        Icon(Icons.Default.ArrowForward, contentDescription = "Forward")
                    }
                    IconButton(onClick = { viewModel.refresh() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Color(0xFF0A0A0A),
                    titleContentColor = Color.White,
                    navigationIconContentColor = Color.White,
                    actionIconContentColor = Color.White
                )
            )
        },
        containerColor = Color(0xFF0A0A0A)
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .background(Color(0xFF0A0A0A))
        ) {
            if (viewModel.currentNode == null) {
                EmptyWikiView()
            } else {
                WikiNodeContent(
                    node = viewModel.currentNode!!,
                    onLinkClick = { link -> viewModel.navigateToNode(link) }
                )
            }
        }
        
        if (showNodeList) {
            NodeListDialog(
                nodes = viewModel.allNodes,
                onNodeSelected = { node ->
                    viewModel.navigateToNode(node)
                    showNodeList = false
                },
                onDismiss = { showNodeList = false }
            )
        }
    }
}

@Composable
fun EmptyWikiView() {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Default.Book,
            contentDescription = null,
            modifier = Modifier.size(80.dp),
            tint = Color.Gray
        )
        
        Spacer(modifier = Modifier.height(20.dp))
        
        Text(
            text = "No Wiki Content Yet",
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.SemiBold,
            color = Color.White
        )
        
        Spacer(modifier = Modifier.height(8.dp))
        
        Text(
            text = "Your compiled knowledge will appear here.\nUse the desktop app for full editing.",
            style = MaterialTheme.typography.bodyMedium,
            color = Color(0xFF737373),
            modifier = Modifier.padding(horizontal = 16.dp)
        )
        
        Spacer(modifier = Modifier.height(24.dp))
        
        Column(
            modifier = Modifier
                .clip(RoundedCornerShape(12.dp))
                .background(Color(0xFF141414))
                .padding(16.dp)
        ) {
            FeatureRow(icon = Icons.Default.PhotoCamera, text = "Capture screenshots via Share")
            FeatureRow(icon = Icons.Default.Psychology, text = "AI compiles them into knowledge")
            FeatureRow(icon = Icons.Default.Link, text = "Auto-linked wiki pages")
        }
    }
}

@Composable
fun FeatureRow(icon: androidx.compose.ui.graphics.vector.ImageVector, text: String) {
    Row(
        modifier = Modifier.padding(vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = Color(0xFFFFB800),
            modifier = Modifier.size(20.dp)
        )
        Spacer(modifier = Modifier.width(12.dp))
        Text(
            text = text,
            style = MaterialTheme.typography.bodyMedium,
            color = Color(0xFFE5E5E5)
        )
    }
}

@Composable
fun WikiNodeContent(
    node: WikiNode,
    onLinkClick: (String) -> Unit
) {
    val scrollState = rememberScrollState()
    
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(scrollState)
            .padding(16.dp)
    ) {
        // Node header
        NodeHeader(node = node)
        
        Spacer(modifier = Modifier.height(16.dp))
        
        // Content
        MarkdownContent(
            content = node.content,
            onLinkClick = onLinkClick
        )
        
        Spacer(modifier = Modifier.height(24.dp))
        
        // Backlinks
        if (node.backlinks.isNotEmpty()) {
            BacklinksSection(
                backlinks = node.backlinks,
                onLinkClick = onLinkClick
            )
            Spacer(modifier = Modifier.height(24.dp))
        }
        
        // Source artifacts
        if (node.sourceArtifacts.isNotEmpty()) {
            SourceArtifactsSection(artifacts = node.sourceArtifacts)
            Spacer(modifier = Modifier.height(24.dp))
        }
        
        // Metadata footer
        NodeMetadataFooter(node = node)
    }
}

@Composable
fun NodeHeader(node: WikiNode) {
    Column {
        Row(
            verticalAlignment = Alignment.CenterVertically
        ) {
            NodeTypeBadge(type = node.nodeType)
            Spacer(modifier = Modifier.weight(1f))
            if (node.isEncrypted) {
                Icon(
                    imageVector = Icons.Default.Lock,
                    contentDescription = "Encrypted",
                    tint = Color(0xFF00D4AA),
                    modifier = Modifier.size(16.dp)
                )
            }
        }
        
        Spacer(modifier = Modifier.height(8.dp))
        
        Text(
            text = node.title,
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            color = Color.White
        )
        
        node.subtitle?.let {
            Text(
                text = it,
                style = MaterialTheme.typography.bodyMedium,
                color = Color(0xFF737373)
            )
        }
    }
}

@Composable
fun NodeTypeBadge(type: WikiNode.NodeType) {
    val (bgColor, textColor) = when (type) {
        WikiNode.NodeType.ENTITY -> Pair(Color.Blue.copy(alpha = 0.2f), Color.Blue)
        WikiNode.NodeType.CONCEPT -> Pair(Color(0xFFFFB800).copy(alpha = 0.2f), Color(0xFFFFB800))
        WikiNode.NodeType.CHRONOLOGY -> Pair(Color.Magenta.copy(alpha = 0.2f), Color.Magenta)
        WikiNode.NodeType.INDEX -> Pair(Color.Green.copy(alpha = 0.2f), Color.Green)
    }
    
    Surface(
        color = bgColor,
        shape = RoundedCornerShape(4.dp)
    ) {
        Text(
            text = type.displayName,
            style = MaterialTheme.typography.labelSmall,
            color = textColor,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp)
        )
    }
}

@Composable
fun MarkdownContent(
    content: String,
    onLinkClick: (String) -> Unit
) {
    // Convert markdown to HTML with wiki link support
    val html = remember(content) {
        renderMarkdownToHtml(content)
    }
    
    AndroidView(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(Color(0xFF141414))
            .padding(16.dp),
        factory = { context ->
            WebView(context).apply {
                setBackgroundColor(android.graphics.Color.TRANSPARENT)
                webViewClient = object : WebViewClient() {
                    override fun shouldOverrideUrlLoading(
                        view: WebView?,
                        url: String?
                    ): Boolean {
                        url?.let {
                            if (it.startsWith("wiki://")) {
                                val nodeName = it.removePrefix("wiki://")
                                onLinkClick(nodeName)
                                return true
                            }
                        }
                        return false
                    }
                }
                loadDataWithBaseURL(null, html, "text/html", "UTF-8", null)
            }
        },
        update = { webView ->
            webView.loadDataWithBaseURL(null, html, "text/html", "UTF-8", null)
        }
    )
}

@Composable
fun BacklinksSection(
    backlinks: List<WikiLink>,
    onLinkClick: (String) -> Unit
) {
    Column {
        Text(
            text = "Backlinks",
            style = MaterialTheme.typography.titleSmall,
            color = Color(0xFF737373)
        )
        
        Spacer(modifier = Modifier.height(8.dp))
        
        backlinks.forEach { link ->
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { onLinkClick(link.target) }
                    .padding(vertical = 4.dp),
                color = Color(0xFF0A0A0A),
                shape = RoundedCornerShape(8.dp)
            ) {
                Row(
                    modifier = Modifier.padding(12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = Icons.Default.Link,
                        contentDescription = null,
                        tint = Color(0xFFFFB800),
                        modifier = Modifier.size(16.dp)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = link.target,
                        style = MaterialTheme.typography.bodyMedium,
                        color = Color(0xFFE5E5E5)
                    )
                    Spacer(modifier = Modifier.weight(1f))
                    Icon(
                        imageVector = Icons.Default.ChevronRight,
                        contentDescription = null,
                        tint = Color(0xFF737373),
                        modifier = Modifier.size(16.dp)
                    )
                }
            }
        }
    }
}

@Composable
fun SourceArtifactsSection(artifacts: List<UUID>) {
    Column {
        Text(
            text = "Source Screenshots",
            style = MaterialTheme.typography.titleSmall,
            color = Color(0xFF737373)
        )
        
        Spacer(modifier = Modifier.height(8.dp))
        
        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            artifacts.forEach { _ ->
                Surface(
                    modifier = Modifier.size(80.dp),
                    color = Color(0xFF0A0A0A),
                    shape = RoundedCornerShape(8.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            imageVector = Icons.Default.Image,
                            contentDescription = null,
                            tint = Color.Gray
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun NodeMetadataFooter(node: WikiNode) {
    Column {
        Divider(color = Color(0xFF262626))
        
        Spacer(modifier = Modifier.height(8.dp))
        
        Row {
            Text(
                text = "Compiled: ${formatDate(node.compiledAt)}",
                style = MaterialTheme.typography.labelSmall,
                color = Color(0xFF737373)
            )
            
            Spacer(modifier = Modifier.weight(1f))
            
            Text(
                text = "v${node.modelVersion}",
                style = MaterialTheme.typography.labelSmall,
                color = Color(0xFF737373)
            )
        }
        
        if (node.confidenceScore < 0.95) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.padding(top = 4.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.Warning,
                    contentDescription = null,
                    tint = Color(0xFFFFA500),
                    modifier = Modifier.size(12.dp)
                )
                Spacer(modifier = Modifier.width(4.dp))
                Text(
                    text = "Low confidence: ${(node.confidenceScore * 100).toInt()}%",
                    style = MaterialTheme.typography.labelSmall,
                    color = Color(0xFFFFA500)
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NodeListDialog(
    nodes: List<WikiNode>,
    onNodeSelected: (WikiNode) -> Unit,
    onDismiss: () -> Unit
) {
    var searchQuery by remember { mutableStateOf("") }
    
    val filteredNodes = remember(searchQuery, nodes) {
        if (searchQuery.isEmpty()) nodes
        else nodes.filter { it.title.contains(searchQuery, ignoreCase = true) }
    }
    
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        Surface(
            modifier = Modifier
                .fillMaxWidth(0.95f)
                .fillMaxHeight(0.8f),
            shape = RoundedCornerShape(16.dp),
            color = Color(0xFF141414)
        ) {
            Column(
                modifier = Modifier.padding(16.dp)
            ) {
                Text(
                    text = "All Pages",
                    style = MaterialTheme.typography.headlineSmall,
                    color = Color.White
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                OutlinedTextField(
                    value = searchQuery,
                    onValueChange = { searchQuery = it },
                    placeholder = { Text("Search pages...") },
                    leadingIcon = { Icon(Icons.Default.Search, null) },
                    modifier = Modifier.fillMaxWidth(),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedBorderColor = Color(0xFFFFB800),
                        unfocusedBorderColor = Color(0xFF262626),
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White
                    )
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                LazyColumn {
                    items(filteredNodes) { node ->
                        Surface(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable { onNodeSelected(node) }
                                .padding(vertical = 4.dp),
                            color = Color.Transparent
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                NodeTypeBadge(type = node.nodeType)
                                Spacer(modifier = Modifier.width(12.dp))
                                Text(
                                    text = node.title,
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = Color(0xFFE5E5E5)
                                )
                                Spacer(modifier = Modifier.weight(1f))
                                Icon(
                                    imageVector = Icons.Default.ChevronRight,
                                    contentDescription = null,
                                    tint = Color(0xFF737373),
                                    modifier = Modifier.size(16.dp)
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

// Helper functions
private fun renderMarkdownToHtml(markdown: String): String {
    // Convert wiki links [[Page Name]] to HTML links
    var html = markdown
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
    
    // Wiki links
    val wikiLinkRegex = "\\[\\[([^\\]]+)\\]\\]".toRegex()
    html = wikiLinkRegex.replace(html) { match ->
        val linkText = match.groupValues[1]
        "<a href=\"wiki://$linkText\" style=\"color: #FFB800; text-decoration: none; background: rgba(255,184,0,0.1); padding: 2px 6px; border-radius: 4px;\">$linkText</a>"
    }
    
    // Headers
    html = html.replace("^# (.+)$".toRegex(RegexOption.MULTILINE), "<h1 style=\"color: white; border-bottom: 1px solid #262626; padding-bottom: 8px;\">$1</h1>")
    html = html.replace("^## (.+)$".toRegex(RegexOption.MULTILINE), "<h2 style=\"color: white;\">$1</h2>")
    html = html.replace("^### (.+)$".toRegex(RegexOption.MULTILINE), "<h3 style=\"color: white;\">$1</h3>")
    
    // Bold and italic
    html = html.replace("\\*\\*(.+?)\\*\\*".toRegex(), "<strong style=\"color: white;\">$1</strong>")
    html = html.replace("\\*(.+?)\\*".toRegex(), "<em>$1</em>")
    
    // Code
    html = html.replace("`(.+?)`".toRegex(), "<code style=\"background: #0A0A0A; padding: 2px 6px; border-radius: 4px; font-family: monospace;\">$1</code>")
    
    // Lists
    html = html.replace("^- (.+)$".toRegex(RegexOption.MULTILINE), "<li style=\"color: #E5E5E5;\">$1</li>")
    
    // Paragraphs
    html = html.replace("\n\n".toRegex(), "</p><p style=\"color: #E5E5E5; margin-bottom: 16px;\">")
    
    return """
        <!DOCTYPE html>
        <html>
        <head>
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <style>
                body { 
                    font-family: -apple-system, BlinkMacSystemFont, sans-serif; 
                    font-size: 16px; 
                    line-height: 1.6; 
                    color: #E5E5E5; 
                    background: transparent;
                    margin: 0;
                    padding: 0;
                }
                a { color: #FFB800; }
            </style>
        </head>
        <body>
            <p style="color: #E5E5E5; margin-bottom: 16px;">$html</p>
        </body>
        </html>
    """.trimIndent()
}

private fun formatDate(date: Date): String {
    val formatter = java.text.SimpleDateFormat("MMM d, yyyy", java.util.Locale.getDefault())
    return formatter.format(date)
}

// Data models
data class WikiNode(
    val id: UUID,
    val nodeType: NodeType,
    val title: String,
    val subtitle: String?,
    val content: String,
    val backlinks: List<WikiLink>,
    val sourceArtifacts: List<UUID>,
    val compiledAt: Date,
    val modelVersion: String,
    val confidenceScore: Double,
    val isEncrypted: Boolean
) {
    enum class NodeType(val displayName: String) {
        ENTITY("Entity"),
        CONCEPT("Concept"),
        CHRONOLOGY("Chronology"),
        INDEX("Index")
    }
}

data class WikiLink(
    val target: String,
    val context: String?
)

// ViewModel
class WikiBrowserViewModel {
    var currentNode by mutableStateOf<WikiNode?>(null)
    var allNodes by mutableStateOf<List<WikiNode>>(emptyList())
    var canGoBack by mutableStateOf(false)
    var canGoForward by mutableStateOf(false)
    
    private val navigationHistory = mutableListOf<WikiNode>()
    private var currentIndex = -1
    
    init {
        loadNodes()
    }
    
    private fun loadNodes() {
        // Load from storage
        allNodes = listOf(
            WikiNode(
                id = UUID.randomUUID(),
                nodeType = WikiNode.NodeType.INDEX,
                title = "Home",
                subtitle = "Your knowledge base index",
                content = """
                    # Welcome to Truffle
                    
                    This is your compiled knowledge base.
                    
                    ## Recent Entities
                    - [[Receipts]]
                    - [[Travel]]
                    - [[Contacts]]
                """.trimIndent(),
                backlinks = emptyList(),
                sourceArtifacts = emptyList(),
                compiledAt = Date(),
                modelVersion = "2.0.0",
                confidenceScore = 1.0,
                isEncrypted = false
            ),
            WikiNode(
                id = UUID.randomUUID(),
                nodeType = WikiNode.NodeType.CONCEPT,
                title = "Receipts",
                subtitle = "Financial records",
                content = "All receipt screenshots are compiled here.",
                backlinks = listOf(WikiLink("Home", null)),
                sourceArtifacts = listOf(UUID.randomUUID()),
                compiledAt = Date(),
                modelVersion = "2.0.0",
                confidenceScore = 0.98,
                isEncrypted = true
            )
        )
        currentNode = allNodes.firstOrNull()
    }
    
    fun navigateToNode(node: WikiNode) {
        if (currentIndex < navigationHistory.size - 1) {
            navigationHistory.subList(currentIndex + 1, navigationHistory.size).clear()
        }
        navigationHistory.add(node)
        currentIndex = navigationHistory.size - 1
        currentNode = node
        updateNavigationState()
    }
    
    fun navigateToNode(name: String) {
        allNodes.find { it.title == name }?.let { navigateToNode(it) }
    }
    
    fun navigateBack() {
        if (currentIndex > 0) {
            currentIndex--
            currentNode = navigationHistory[currentIndex]
            updateNavigationState()
        }
    }
    
    fun navigateForward() {
        if (currentIndex < navigationHistory.size - 1) {
            currentIndex++
            currentNode = navigationHistory[currentIndex]
            updateNavigationState()
        }
    }
    
    fun refresh() {
        loadNodes()
    }
    
    private fun updateNavigationState() {
        canGoBack = currentIndex > 0
        canGoForward = currentIndex < navigationHistory.size - 1
    }
}
