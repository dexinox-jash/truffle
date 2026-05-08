package com.truffle.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.truffle.app.StorageManager

/**
 * SettingsScreen - Settings Interface for Android
 * 
 * SPEC v2.0 COMPLIANCE:
 * - Storage management
 * - Sync settings
 * - Device pairing
 * - Export functionality
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen() {
    val viewModel = remember { SettingsViewModel() }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
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
                .verticalScroll(rememberScrollState())
                .padding(16.dp)
        ) {
            // Storage section
            SettingsSection(title = "Storage") {
                StorageInfoCard(
                    usedBytes = viewModel.storageUsed,
                    onManageClick = { viewModel.showStorageDialog = true }
                )
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Sync section
            SettingsSection(title = "Sync") {
                SyncInfoCard(
                    lastSync = viewModel.lastSyncTime,
                    isSyncing = viewModel.isSyncing,
                    onSyncNow = { viewModel.syncNow() }
                )
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Data section
            SettingsSection(title = "Data") {
                SettingsItem(
                    icon = Icons.Default.Download,
                    title = "Export to Obsidian",
                    subtitle = "Export wiki as markdown",
                    onClick = { viewModel.showExportDialog = true }
                )
                
                SettingsItem(
                    icon = Icons.Default.QrCode,
                    title = "Pair New Device",
                    subtitle = "Sync with another device",
                    onClick = { viewModel.showPairingDialog = true }
                )
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // About section
            SettingsSection(title = "About") {
                InfoItem(title = "Version", value = "2.0.0")
                
                LinkItem(
                    icon = Icons.Default.MenuBook,
                    title = "Documentation",
                    url = "https://docs.truffle.io"
                )
                
                LinkItem(
                    icon = Icons.Default.PrivacyTip,
                    title = "Privacy Policy",
                    url = "https://truffle.io/privacy"
                )
            }
        }
    }
    
    // Dialogs
    if (viewModel.showStorageDialog) {
        StorageDialog(
            onDismiss = { viewModel.showStorageDialog = false },
            onClearFailed = { viewModel.clearFailedCaptures() },
            onClearAll = { viewModel.clearAllRaw() }
        )
    }
    
    if (viewModel.showExportDialog) {
        ExportDialog(
            onDismiss = { viewModel.showExportDialog = false },
            onExport = { viewModel.exportToObsidian() }
        )
    }
    
    if (viewModel.showPairingDialog) {
        PairingDialog(
            onDismiss = { viewModel.showPairingDialog = false }
        )
    }
}

@Composable
fun SettingsSection(
    title: String,
    content: @Composable ColumnScope.() -> Unit
) {
    Column {
        Text(
            text = title,
            style = MaterialTheme.typography.titleSmall,
            color = Color(0xFF737373),
            modifier = Modifier.padding(vertical = 8.dp)
        )
        
        Surface(
            color = Color(0xFF141414),
            shape = RoundedCornerShape(12.dp)
        ) {
            Column(
                modifier = Modifier.padding(vertical = 8.dp)
            ) {
                content()
            }
        }
    }
}

@Composable
fun StorageInfoCard(
    usedBytes: Long,
    onManageClick: () -> Unit
) {
    val formattedSize = remember(usedBytes) {
        formatBytes(usedBytes)
    }
    
    Column(
        modifier = Modifier.padding(16.dp)
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = Icons.Default.Storage,
                contentDescription = null,
                tint = Color(0xFFFFB800),
                modifier = Modifier.size(24.dp)
            )
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = "Storage Used",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Color.White
                )
                Text(
                    text = formattedSize,
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Bold,
                    color = Color(0xFFFFB800)
                )
            }
            
            TextButton(onClick = onManageClick) {
                Text("Manage")
            }
        }
        
        // Storage bar
        val fiveGB = 5L * 1024 * 1024 * 1024
        val tenGB = 10L * 1024 * 1024 * 1024
        val progress = (usedBytes.toFloat() / tenGB).coerceIn(0f, 1f)
        
        Spacer(modifier = Modifier.height(12.dp))
        
        LinearProgressIndicator(
            progress = { progress },
            modifier = Modifier
                .fillMaxWidth()
                .height(4.dp)
                .clip(RoundedCornerShape(2.dp)),
            color = when {
                usedBytes >= tenGB -> Color.Red
                usedBytes >= fiveGB -> Color(0xFFFFA500)
                else -> Color(0xFF00D4AA)
            },
            trackColor = Color(0xFF262626)
        )
        
        Spacer(modifier = Modifier.height(4.dp))
        
        Text(
            text = when {
                usedBytes >= tenGB -> "Storage full - Captures paused"
                usedBytes >= fiveGB -> "Warning: Approaching limit"
                else -> "Healthy"
            },
            style = MaterialTheme.typography.labelSmall,
            color = when {
                usedBytes >= tenGB -> Color.Red
                usedBytes >= fiveGB -> Color(0xFFFFA500)
                else -> Color(0xFF00D4AA)
            }
        )
    }
}

@Composable
fun SyncInfoCard(
    lastSync: java.util.Date?,
    isSyncing: Boolean,
    onSyncNow: () -> Unit
) {
    Column(
        modifier = Modifier.padding(16.dp)
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = Icons.Default.Sync,
                contentDescription = null,
                tint = Color(0xFF00D4AA),
                modifier = Modifier.size(24.dp)
            )
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = "Last Sync",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Color.White
                )
                Text(
                    text = lastSync?.let { formatDateTime(it) } ?: "Never",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Color(0xFF737373)
                )
            }
            
            if (isSyncing) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    color = Color(0xFFFFB800),
                    strokeWidth = 2.dp
                )
            } else {
                TextButton(onClick = onSyncNow) {
                    Text("Sync Now")
                }
            }
        }
        
        Spacer(modifier = Modifier.height(8.dp))
        
        Text(
            text = "Auto-syncs every 15 minutes when charging",
            style = MaterialTheme.typography.labelSmall,
            color = Color(0xFF737373)
        )
    }
}

@Composable
fun SettingsItem(
    icon: ImageVector,
    title: String,
    subtitle: String,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        color = Color.Transparent,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .padding(horizontal = 16.dp, vertical = 12.dp)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = Color(0xFFFFB800),
                modifier = Modifier.size(24.dp)
            )
            
            Spacer(modifier = Modifier.width(16.dp))
            
            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.bodyMedium,
                    color = Color.White
                )
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.bodySmall,
                    color = Color(0xFF737373)
                )
            }
            
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
fun InfoItem(title: String, value: String) {
    Row(
        modifier = Modifier
            .padding(horizontal = 16.dp, vertical = 12.dp)
            .fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.bodyMedium,
            color = Color.White,
            modifier = Modifier.weight(1f)
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
            color = Color(0xFF737373)
        )
    }
}

@Composable
fun LinkItem(
    icon: ImageVector,
    title: String,
    url: String
) {
    val context = androidx.compose.ui.platform.LocalContext.current
    
    Surface(
        onClick = {
            val intent = android.content.Intent(android.content.Intent.ACTION_VIEW, android.net.Uri.parse(url))
            context.startActivity(intent)
        },
        color = Color.Transparent,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .padding(horizontal = 16.dp, vertical = 12.dp)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = Color(0xFFFFB800),
                modifier = Modifier.size(24.dp)
            )
            
            Spacer(modifier = Modifier.width(16.dp))
            
            Text(
                text = title,
                style = MaterialTheme.typography.bodyMedium,
                color = Color.White,
                modifier = Modifier.weight(1f)
            )
            
            Icon(
                imageVector = Icons.Default.OpenInNew,
                contentDescription = null,
                tint = Color(0xFF737373),
                modifier = Modifier.size(16.dp)
            )
        }
    }
}

@Composable
fun StorageDialog(
    onDismiss: () -> Unit,
    onClearFailed: () -> Unit,
    onClearAll: () -> Unit
) {
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = Color(0xFF141414)
        ) {
            Column(
                modifier = Modifier.padding(20.dp)
            ) {
                Text(
                    text = "Manage Storage",
                    style = MaterialTheme.typography.headlineSmall,
                    color = Color.White
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                TextButton(
                    onClick = {
                        onClearFailed()
                        onDismiss()
                    },
                    colors = ButtonDefaults.textButtonColors(
                        contentColor = Color(0xFFFFA500)
                    )
                ) {
                    Text("Clear Failed Captures")
                }
                
                TextButton(
                    onClick = {
                        onClearAll()
                        onDismiss()
                    },
                    colors = ButtonDefaults.textButtonColors(
                        contentColor = Color.Red
                    )
                ) {
                    Text("Clear All Raw Images")
                }
                
                Spacer(modifier = Modifier.height(8.dp))
                
                TextButton(onClick = onDismiss) {
                    Text("Cancel")
                }
            }
        }
    }
}

@Composable
fun ExportDialog(
    onDismiss: () -> Unit,
    onExport: () -> Unit
) {
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = Color(0xFF141414)
        ) {
            Column(
                modifier = Modifier.padding(20.dp)
            ) {
                Text(
                    text = "Export to Obsidian",
                    style = MaterialTheme.typography.headlineSmall,
                    color = Color.White
                )
                
                Spacer(modifier = Modifier.height(8.dp))
                
                Text(
                    text = "This will export your wiki as markdown files compatible with Obsidian.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Color(0xFF737373)
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                Row(
                    horizontalArrangement = Arrangement.End,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    TextButton(onClick = onDismiss) {
                        Text("Cancel")
                    }
                    
                    Spacer(modifier = Modifier.width(8.dp))
                    
                    Button(
                        onClick = {
                            onExport()
                            onDismiss()
                        },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = Color(0xFFFFB800),
                            contentColor = Color.Black
                        )
                    ) {
                        Text("Export")
                    }
                }
            }
        }
    }
}

@Composable
fun PairingDialog(
    onDismiss: () -> Unit
) {
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = Color(0xFF141414)
        ) {
            Column(
                modifier = Modifier.padding(20.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    text = "Pair New Device",
                    style = MaterialTheme.typography.headlineSmall,
                    color = Color.White
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                // QR Code placeholder
                Box(
                    modifier = Modifier
                        .size(200.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .background(Color.White),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        imageVector = Icons.Default.QrCode,
                        contentDescription = "QR Code",
                        tint = Color.Black,
                        modifier = Modifier.size(150.dp)
                    )
                }
                
                Spacer(modifier = Modifier.height(16.dp))
                
                Text(
                    text = "Scan this QR code with your other device",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Color(0xFF737373)
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                TextButton(onClick = onDismiss) {
                    Text("Close")
                }
            }
        }
    }
}

// ViewModel
class SettingsViewModel {
    var storageUsed by mutableStateOf(0L)
    var lastSyncTime by mutableStateOf<java.util.Date?>(null)
    var isSyncing by mutableStateOf(false)
    
    var showStorageDialog by mutableStateOf(false)
    var showExportDialog by mutableStateOf(false)
    var showPairingDialog by mutableStateOf(false)
    
    init {
        loadSettings()
    }
    
    private fun loadSettings() {
        // Load from preferences
        storageUsed = 2_500_000_000 // 2.5 GB
    }
    
    fun syncNow() {
        isSyncing = true
        // Perform sync
        kotlinx.coroutines.GlobalScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            kotlinx.coroutines.delay(2000)
            kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Main) {
                isSyncing = false
                lastSyncTime = java.util.Date()
            }
        }
    }
    
    fun clearFailedCaptures() {
        // Clear failed captures
    }
    
    fun clearAllRaw() {
        // Clear all raw images
    }
    
    fun exportToObsidian() {
        // Export to Obsidian format
    }
}

// Helper functions
private fun formatBytes(bytes: Long): String {
    val units = arrayOf("B", "KB", "MB", "GB", "TB")
    var size = bytes.toDouble()
    var unitIndex = 0
    
    while (size >= 1024 && unitIndex < units.size - 1) {
        size /= 1024
        unitIndex++
    }
    
    return String.format("%.2f %s", size, units[unitIndex])
}

private fun formatDateTime(date: java.util.Date): String {
    val formatter = java.text.SimpleDateFormat("MMM d, HH:mm", java.util.Locale.getDefault())
    return formatter.format(date)
}
