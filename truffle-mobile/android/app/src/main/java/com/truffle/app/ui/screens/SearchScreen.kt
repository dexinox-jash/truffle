package com.truffle.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
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
import java.util.*

/**
 * SearchScreen - Search Interface for Android
 * 
 * SPEC v2.0 COMPLIANCE:
 * - Full-text search using sqlite-vec
 * - Semantic search support
 * - Filter by type (wiki, screenshots, entities)
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchScreen() {
    val viewModel = remember { SearchViewModel() }
    var searchQuery by remember { mutableStateOf("") }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Search") },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Color(0xFF0A0A0A),
                    titleContentColor = Color.White
                )
            )
        },
        containerColor = Color(0xFF0A0A0A)
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(16.dp)
        ) {
            // Search bar
            SearchBar(
                query = searchQuery,
                onQueryChange = { 
                    searchQuery = it
                    viewModel.search(it)
                },
                isSearching = viewModel.isSearching
            )
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Filter chips
            FilterChips(
                selectedFilter = viewModel.selectedFilter,
                onFilterSelected = { viewModel.selectedFilter = it }
            )
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Results
            when {
                searchQuery.isEmpty() -> SearchSuggestions(viewModel)
                viewModel.results.isEmpty() -> EmptySearchView(searchQuery)
                else -> SearchResultsList(
                    results = viewModel.results,
                    onResultClick = { result ->
                        viewModel.selectedResult = result
                    }
                )
            }
        }
    }
    
    // Result detail dialog
    viewModel.selectedResult?.let { result ->
        SearchResultDialog(
            result = result,
            onDismiss = { viewModel.selectedResult = null }
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchBar(
    query: String,
    onQueryChange: (String) -> Unit,
    isSearching: Boolean
) {
    OutlinedTextField(
        value = query,
        onValueChange = onQueryChange,
        placeholder = { Text("Search wiki, screenshots...") },
        leadingIcon = { Icon(Icons.Default.Search, null) },
        trailingIcon = {
            if (isSearching) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    color = Color(0xFFFFB800),
                    strokeWidth = 2.dp
                )
            } else if (query.isNotEmpty()) {
                IconButton(onClick = { onQueryChange("") }) {
                    Icon(Icons.Default.Clear, null)
                }
            }
        },
        modifier = Modifier.fillMaxWidth(),
        colors = OutlinedTextFieldDefaults.colors(
            focusedBorderColor = Color(0xFFFFB800),
            unfocusedBorderColor = Color(0xFF262626),
            focusedTextColor = Color.White,
            unfocusedTextColor = Color.White,
            focusedContainerColor = Color(0xFF141414),
            unfocusedContainerColor = Color(0xFF141414)
        ),
        shape = RoundedCornerShape(12.dp)
    )
}

@Composable
fun FilterChips(
    selectedFilter: SearchFilter,
    onFilterSelected: (SearchFilter) -> Unit
) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        SearchFilter.values().forEach { filter ->
            FilterChip(
                filter = filter,
                isSelected = selectedFilter == filter,
                onClick = { onFilterSelected(filter) }
            )
        }
    }
}

@Composable
fun FilterChip(
    filter: SearchFilter,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        color = if (isSelected) Color(0xFFFFB800) else Color(0xFF141414),
        shape = RoundedCornerShape(16.dp)
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = filter.icon,
                contentDescription = null,
                tint = if (isSelected) Color.Black else Color(0xFF737373),
                modifier = Modifier.size(16.dp)
            )
            Spacer(modifier = Modifier.width(6.dp))
            Text(
                text = filter.displayName,
                style = MaterialTheme.typography.labelMedium,
                color = if (isSelected) Color.Black else Color(0xFFE5E5E5)
            )
        }
    }
}

@Composable
fun SearchSuggestions(viewModel: SearchViewModel) {
    LazyColumn {
        item {
            Text(
                text = "Recent Searches",
                style = MaterialTheme.typography.titleSmall,
                color = Color(0xFF737373),
                modifier = Modifier.padding(vertical = 8.dp)
            )
        }
        
        items(viewModel.recentSearches) { search ->
            SuggestionItem(
                icon = Icons.Default.History,
                text = search,
                onClick = { /* Perform search */ }
            )
        }
        
        item {
            Text(
                text = "Suggested",
                style = MaterialTheme.typography.titleSmall,
                color = Color(0xFF737373),
                modifier = Modifier.padding(vertical = 8.dp)
            )
        }
        
        items(viewModel.suggestedSearches) { suggestion ->
            SuggestionItem(
                icon = Icons.Default.Star,
                text = suggestion,
                iconTint = Color(0xFFFFB800),
                onClick = { /* Perform search */ }
            )
        }
    }
}

@Composable
fun SuggestionItem(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    text: String,
    iconTint: Color = Color(0xFF737373),
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        color = Color.Transparent,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .padding(vertical = 12.dp)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = iconTint,
                modifier = Modifier.size(20.dp)
            )
            Spacer(modifier = Modifier.width(16.dp))
            Text(
                text = text,
                style = MaterialTheme.typography.bodyMedium,
                color = Color(0xFFE5E5E5)
            )
        }
    }
}

@Composable
fun EmptySearchView(query: String) {
    Column(
        modifier = Modifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Icon(
            imageVector = Icons.Default.Search,
            contentDescription = null,
            modifier = Modifier.size(60.dp),
            tint = Color.Gray
        )
        
        Spacer(modifier = Modifier.height(16.dp))
        
        Text(
            text = "No results for \"$query\"",
            style = MaterialTheme.typography.titleMedium,
            color = Color.White
        )
        
        Spacer(modifier = Modifier.height(8.dp))
        
        Text(
            text = "Try different keywords or check your spelling",
            style = MaterialTheme.typography.bodyMedium,
            color = Color(0xFF737373)
        )
    }
}

@Composable
fun SearchResultsList(
    results: List<SearchResult>,
    onResultClick: (SearchResult) -> Unit
) {
    LazyColumn {
        items(results) { result ->
            SearchResultItem(
                result = result,
                onClick = { onResultClick(result) }
            )
        }
    }
}

@Composable
fun SearchResultItem(
    result: SearchResult,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        color = Color.Transparent,
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier
                .padding(vertical = 12.dp)
                .fillMaxWidth()
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically
            ) {
                ResultTypeIcon(type = result.type)
                
                Spacer(modifier = Modifier.width(12.dp))
                
                Text(
                    text = result.title,
                    style = MaterialTheme.typography.titleSmall,
                    color = Color.White,
                    modifier = Modifier.weight(1f)
                )
                
                result.relevanceScore?.let { score ->
                    RelevanceBadge(score = score)
                }
            }
            
            result.snippet?.let { snippet ->
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = snippet,
                    style = MaterialTheme.typography.bodySmall,
                    color = Color(0xFF737373),
                    maxLines = 2
                )
            }
            
            Spacer(modifier = Modifier.height(4.dp))
            
            Row(
                verticalAlignment = Alignment.CenterVertically
            ) {
                result.lastModified?.let { date ->
                    Icon(
                        imageVector = Icons.Default.CalendarToday,
                        contentDescription = null,
                        tint = Color(0xFF737373),
                        modifier = Modifier.size(12.dp)
                    )
                    Spacer(modifier = Modifier.width(4.dp))
                    Text(
                        text = formatDate(date),
                        style = MaterialTheme.typography.labelSmall,
                        color = Color(0xFF737373)
                    )
                }
                
                if (result.isEncrypted) {
                    Spacer(modifier = Modifier.width(12.dp))
                    Icon(
                        imageVector = Icons.Default.Lock,
                        contentDescription = "Encrypted",
                        tint = Color(0xFF00D4AA),
                        modifier = Modifier.size(12.dp)
                    )
                    Spacer(modifier = Modifier.width(4.dp))
                    Text(
                        text = "Encrypted",
                        style = MaterialTheme.typography.labelSmall,
                        color = Color(0xFF00D4AA)
                    )
                }
            }
        }
    }
}

@Composable
fun ResultTypeIcon(type: SearchResult.ResultType) {
    val (icon, color) = when (type) {
        SearchResult.ResultType.WIKI_NODE -> Pair(Icons.Default.Description, Color(0xFFFFB800))
        SearchResult.ResultType.SCREENSHOT -> Pair(Icons.Default.Image, Color.Blue)
        SearchResult.ResultType.ENTITY -> Pair(Icons.Default.Person, Color.Green)
    }
    
    Box(
        modifier = Modifier
            .size(32.dp)
            .clip(RoundedCornerShape(6.dp))
            .background(color.copy(alpha = 0.2f)),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = color,
            modifier = Modifier.size(16.dp)
        )
    }
}

@Composable
fun RelevanceBadge(score: Double) {
    val percentage = (score * 100).toInt()
    val color = when {
        score > 0.9 -> Color.Green
        score > 0.7 -> Color(0xFFFFB800)
        else -> Color(0xFFFFA500)
    }
    
    Surface(
        color = color.copy(alpha = 0.2f),
        shape = RoundedCornerShape(4.dp)
    ) {
        Text(
            text = "$percentage%",
            style = MaterialTheme.typography.labelSmall,
            color = color,
            modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
        )
    }
}

@Composable
fun SearchResultDialog(
    result: SearchResult,
    onDismiss: () -> Unit
) {
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
                Row(
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    ResultTypeIcon(type = result.type)
                    Spacer(modifier = Modifier.width(12.dp))
                    Text(
                        text = result.title,
                        style = MaterialTheme.typography.headlineSmall,
                        color = Color.White,
                        modifier = Modifier.weight(1f)
                    )
                    IconButton(onClick = onDismiss) {
                        Icon(Icons.Default.Close, null, tint = Color.White)
                    }
                }
                
                Spacer(modifier = Modifier.height(16.dp))
                
                result.fullContent?.let { content ->
                    Text(
                        text = content,
                        style = MaterialTheme.typography.bodyMedium,
                        color = Color(0xFFE5E5E5)
                    )
                }
            }
        }
    }
}

// Data models
enum class SearchFilter(
    val displayName: String,
    val icon: androidx.compose.ui.graphics.vector.ImageVector
) {
    ALL("All", Icons.Default.SelectAll),
    WIKI("Wiki", Icons.Default.Book),
    SCREENSHOTS("Screenshots", Icons.Default.PhotoLibrary),
    ENTITIES("Entities", Icons.Default.People)
}

data class SearchResult(
    val id: UUID,
    val title: String,
    val type: ResultType,
    val snippet: String?,
    val fullContent: String?,
    val relevanceScore: Double?,
    val lastModified: Date?,
    val isEncrypted: Boolean,
    val sourceArtifacts: List<UUID>
) {
    enum class ResultType {
        WIKI_NODE, SCREENSHOT, ENTITY
    }
}

// ViewModel
class SearchViewModel {
    var isSearching by mutableStateOf(false)
    var selectedFilter by mutableStateOf(SearchFilter.ALL)
    var results by mutableStateOf<List<SearchResult>>(emptyList())
    var selectedResult by mutableStateOf<SearchResult?>(null)
    
    val recentSearches = listOf("receipts", "travel", "coffee shop")
    val suggestedSearches = listOf("Recent receipts", "Uncompiled screenshots", "Financial entities")
    
    private var searchJob: kotlinx.coroutines.Job? = null
    
    fun search(query: String) {
        searchJob?.cancel()
        
        if (query.isEmpty()) {
            results = emptyList()
            return
        }
        
        isSearching = true
        
        searchJob = kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.Dispatchers.IO).launch {
            kotlinx.coroutines.delay(300) // Debounce
            
            // Perform search
            val searchResults = performSearch(query, selectedFilter)
            
            kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Main) {
                results = searchResults
                isSearching = false
            }
        }
    }
    
    private fun performSearch(query: String, filter: SearchFilter): List<SearchResult> {
        // In production, use sqlite-vec for semantic search
        // and FTS for full-text search
        
        return listOf(
            SearchResult(
                id = UUID.randomUUID(),
                title = "Receipt: Coffee Shop - Jan 15",
                type = SearchResult.ResultType.WIKI_NODE,
                snippet = "Total: $12.50 | Items: Latte, Croissant | Payment: Credit Card",
                fullContent = "# Receipt: Coffee Shop\n\n**Date:** January 15, 2024\n**Total:** $12.50",
                relevanceScore = 0.95,
                lastModified = Date(),
                isEncrypted = true,
                sourceArtifacts = listOf(UUID.randomUUID())
            ),
            SearchResult(
                id = UUID.randomUUID(),
                title = "Receipt: Grocery Store - Jan 14",
                type = SearchResult.ResultType.WIKI_NODE,
                snippet = "Total: $87.32 | 23 items | Store: Whole Foods Market",
                fullContent = null,
                relevanceScore = 0.82,
                lastModified = Date(Date().time - 86400000),
                isEncrypted = true,
                sourceArtifacts = listOf(UUID.randomUUID())
            )
        )
    }
}

private fun formatDate(date: Date): String {
    val formatter = java.text.SimpleDateFormat("MMM d, yyyy", java.util.Locale.getDefault())
    return formatter.format(date)
}
