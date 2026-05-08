package com.truffle.app.ui.screens

import androidx.compose.foundation.Image
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
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import java.util.*

/**
 * CapturesScreen - Raw Captures View for Android
 * 
 * SPEC v2.0 COMPLIANCE:
 * - View all captured screenshots
 * - Status tracking (pending, processing, completed, failed)
 * - View original screenshots
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CapturesScreen() {
    val viewModel = remember { CapturesViewModel() }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Captures") },
                actions = {
                    IconButton(onClick = { viewModel.refresh() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Color(0xFF0A0A0A),
                    titleContentColor = Color.White,
                    actionIconContentColor = Color.White
                )
            )
        },
        containerColor = Color(0xFF0A0A0A)
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 16.dp)
        ) {
            // Status summary
            item {
                StatusSummary(
                    pending = viewModel.pendingCount,
                    processing = viewModel.processingCount,
                    completed = viewModel.completedCount,
                    failed = viewModel.failedCount
                )
                
                Spacer(modifier = Modifier.height(16.dp))
            }
            
            // Capture list
            items(viewModel.captures) { capture ->
                CaptureItem(
                    capture = capture,
                    onClick = { viewModel.selectedCapture = capture }
                )
                
                Spacer(modifier = Modifier.height(8.dp))
            }
        }
    }
    
    // Capture detail dialog
    viewModel.selectedCapture?.let { capture ->
        CaptureDetailDialog(
            capture = capture,
            onDismiss = { viewModel.selectedCapture = null }
        )
    }
}

@Composable
fun StatusSummary(
    pending: Int,
    processing: Int,
    completed: Int,
    failed: Int
) {
    Surface(
        color = Color(0xFF141414),
        shape = RoundedCornerShape(12.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            StatusCount(count = pending, label = "Pending", color = Color(0xFFFFA500))
            StatusCount(count = processing, label = "Processing", color = Color.Blue)
            StatusCount(count = completed, label = "Done", color = Color(0xFF00D4AA))
            StatusCount(count = failed, label = "Failed", color = Color.Red)
        }
    }
}

@Composable
fun StatusCount(count: Int, label: String, color: Color) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = count.toString(),
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.Bold,
            color = color
        )
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall,
            color = Color(0xFF737373)
        )
    }
}

@Composable
fun CaptureItem(
    capture: Capture,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        color = Color(0xFF141414),
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Thumbnail
            Box(
                modifier = Modifier
                    .size(60.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(Color(0xFF0A0A0A)),
                contentAlignment = Alignment.Center
            ) {
                capture.thumbnail?.let { bitmap ->
                    Image(
                        bitmap = bitmap.asImageBitmap(),
                        contentDescription = null,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier.fillMaxSize()
                    )
                } ?: run {
                    Icon(
                        imageVector = Icons.Default.Image,
                        contentDescription = null,
                        tint = Color.Gray
                    )
                }
            }
            
            Spacer(modifier = Modifier.width(12.dp))
            
            // Info
            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = capture.filename,
                    style = MaterialTheme.typography.bodyMedium,
                    color = Color.White,
                    maxLines = 1
                )
                
                Spacer(modifier = Modifier.height(4.dp))
                
                Row(
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    StatusBadge(status = capture.status)
                    
                    Spacer(modifier = Modifier.width(8.dp))
                    
                    Text(
                        text = formatDate(capture.capturedAt),
                        style = MaterialTheme.typography.labelSmall,
                        color = Color(0xFF737373)
                    )
                }
            }
            
            // Arrow
            Icon(
                imageVector = Icons.Default.ChevronRight,
                contentDescription = null,
                tint = Color(0xFF737373),
                modifier = Modifier.size(20.dp)
            )
        }
    }
}

@Composable
fun StatusBadge(status: CaptureStatus) {
    val (bgColor, textColor, text) = when (status) {
        CaptureStatus.PENDING -> Triple(
            Color(0xFFFFA500).copy(alpha = 0.2f),
            Color(0xFFFFA500),
            "Pending"
        )
        CaptureStatus.PROCESSING -> Triple(
            Color.Blue.copy(alpha = 0.2f),
            Color.Blue,
            "Processing"
        )
        CaptureStatus.COMPLETED -> Triple(
            Color(0xFF00D4AA).copy(alpha = 0.2f),
            Color(0xFF00D4AA),
            "Done"
        )
        CaptureStatus.FAILED -> Triple(
            Color.Red.copy(alpha = 0.2f),
            Color.Red,
            "Failed"
        )
        CaptureStatus.QUARANTINED -> Triple(
            Color.Magenta.copy(alpha = 0.2f),
            Color.Magenta,
            "Quarantined"
        )
    }
    
    Surface(
        color = bgColor,
        shape = RoundedCornerShape(4.dp)
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(6.dp)
                    .background(textColor, RoundedCornerShape(3.dp))
            )
            Spacer(modifier = Modifier.width(4.dp))
            Text(
                text = text,
                style = MaterialTheme.typography.labelSmall,
                color = textColor
            )
        }
    }
}

@Composable
fun CaptureDetailDialog(
    capture: Capture,
    onDismiss: () -> Unit
) {
    Dialog(onDismissRequest = onDismiss) {
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
                // Header
                Row(
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "Capture Details",
                        style = MaterialTheme.typography.headlineSmall,
                        color = Color.White,
                        modifier = Modifier.weight(1f)
                    )
                    IconButton(onClick = onDismiss) {
                        Icon(Icons.Default.Close, null, tint = Color.White)
                    }
                }
                
                Spacer(modifier = Modifier.height(16.dp))
                
                // Full image
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(9f / 16f)
                        .clip(RoundedCornerShape(12.dp))
                        .background(Color(0xFF0A0A0A)),
                    contentAlignment = Alignment.Center
                ) {
                    capture.fullImage?.let { bitmap ->
                        Image(
                            bitmap = bitmap.asImageBitmap(),
                            contentDescription = null,
                            contentScale = ContentScale.Fit,
                            modifier = Modifier.fillMaxSize()
                        )
                    } ?: run {
                        Icon(
                            imageVector = Icons.Default.Image,
                            contentDescription = null,
                            tint = Color.Gray,
                            modifier = Modifier.size(60.dp)
                        )
                    }
                }
                
                Spacer(modifier = Modifier.height(16.dp))
                
                // Details
                DetailRow(label = "Filename", value = capture.filename)
                DetailRow(label = "Captured", value = formatDateTime(capture.capturedAt))
                DetailRow(label = "Size", value = formatBytes(capture.sizeBytes))
                DetailRow(label = "Status", value = capture.status.name)
                
                capture.ocrText?.let { ocr ->
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Extracted Text",
                        style = MaterialTheme.typography.labelSmall,
                        color = Color(0xFF737373)
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Surface(
                        color = Color(0xFF0A0A0A),
                        shape = RoundedCornerShape(8.dp)
                    ) {
                        Text(
                            text = ocr,
                            style = MaterialTheme.typography.bodySmall,
                            color = Color(0xFFE5E5E5),
                            modifier = Modifier.padding(12.dp)
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun DetailRow(label: String, value: String) {
    Row(
        modifier = Modifier.padding(vertical = 4.dp)
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall,
            color = Color(0xFF737373),
            modifier = Modifier.width(80.dp)
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
            color = Color(0xFFE5E5E5)
        )
    }
}

// Data models
enum class CaptureStatus {
    PENDING, PROCESSING, COMPLETED, FAILED, QUARANTINED
}

data class Capture(
    val id: UUID,
    val filename: String,
    val capturedAt: Date,
    val sizeBytes: Long,
    val status: CaptureStatus,
    val thumbnail: android.graphics.Bitmap?,
    val fullImage: android.graphics.Bitmap?,
    val ocrText: String?
)

// ViewModel
class CapturesViewModel {
    var captures by mutableStateOf<List<Capture>>(emptyList())
    var selectedCapture by mutableStateOf<Capture?>(null)
    
    val pendingCount: Int get() = captures.count { it.status == CaptureStatus.PENDING }
    val processingCount: Int get() = captures.count { it.status == CaptureStatus.PROCESSING }
    val completedCount: Int get() = captures.count { it.status == CaptureStatus.COMPLETED }
    val failedCount: Int get() = captures.count { it.status == CaptureStatus.FAILED }
    
    init {
        loadCaptures()
    }
    
    private fun loadCaptures() {
        // Load from database
        captures = listOf(
            Capture(
                id = UUID.randomUUID(),
                filename = "capture_20240115_083012.jpg",
                capturedAt = Date(),
                sizeBytes = 2_500_000,
                status = CaptureStatus.COMPLETED,
                thumbnail = null,
                fullImage = null,
                ocrText = "Coffee Shop Receipt\nTotal: $12.50"
            ),
            Capture(
                id = UUID.randomUUID(),
                filename = "capture_20240114_192345.jpg",
                capturedAt = Date(Date().time - 86400000),
                sizeBytes = 3_200_000,
                status = CaptureStatus.PENDING,
                thumbnail = null,
                fullImage = null,
                ocrText = null
            )
        )
    }
    
    fun refresh() {
        loadCaptures()
    }
}

// Helper functions
private fun formatDate(date: Date): String {
    val formatter = java.text.SimpleDateFormat("MMM d", java.util.Locale.getDefault())
    return formatter.format(date)
}

private fun formatDateTime(date: Date): String {
    val formatter = java.text.SimpleDateFormat("MMM d, yyyy HH:mm", java.util.Locale.getDefault())
    return formatter.format(date)
}

private fun formatBytes(bytes: Long): String {
    val units = arrayOf("B", "KB", "MB", "GB")
    var size = bytes.toDouble()
    var unitIndex = 0
    
    while (size >= 1024 && unitIndex < units.size - 1) {
        size /= 1024
        unitIndex++
    }
    
    return String.format("%.2f %s", size, units[unitIndex])
}
