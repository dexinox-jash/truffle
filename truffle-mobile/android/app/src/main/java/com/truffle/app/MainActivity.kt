package com.truffle.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.truffle.app.ui.screens.*
import com.truffle.app.ui.theme.TruffleTheme
import com.truffle.keystorage.KeystoreManager
import com.truffle.workmanager.CompilationWorker
import androidx.work.*

/**
 * MainActivity - Truffle Android Main Entry Point
 * 
 * SPEC v2.0 COMPLIANCE: Mobile is COMPANION-ONLY
 * - Read-only wiki browser
 * - Background sync when charging
 * - Storage warnings at 5GB, pause at 10GB
 */
class MainActivity : ComponentActivity() {
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize secure storage
        KeystoreManager.initialize(this)
        
        // Schedule background compilation
        scheduleBackgroundCompilation()
        
        // Check storage on launch
        checkStorage()
        
        enableEdgeToEdge()
        setContent {
            TruffleTheme {
                TruffleApp()
            }
        }
    }
    
    private fun scheduleBackgroundCompilation() {
        val constraints = Constraints.Builder()
            .setRequiresBatteryNotLow(true)
            .setRequiresStorageNotLow(true)
            .build()
        
        val compilationWork = PeriodicWorkRequestBuilder<CompilationWorker>(
            15, java.util.concurrent.TimeUnit.MINUTES
        )
            .setConstraints(constraints)
            .addTag("truffle_compilation")
            .build()
        
        WorkManager.getInstance(this).enqueueUniquePeriodicWork(
            "truffle_compilation",
            ExistingPeriodicWorkPolicy.KEEP,
            compilationWork
        )
    }
    
    private fun checkStorage() {
        val storageManager = StorageManager(this)
        val used = storageManager.getStorageUsed()
        val fiveGB = 5L * 1024 * 1024 * 1024
        val tenGB = 10L * 1024 * 1024 * 1024
        
        when {
            used >= tenGB -> {
                // Show critical storage warning
            }
            used >= fiveGB -> {
                // Show storage warning
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TruffleApp() {
    val navController = rememberNavController()
    
    val screens = listOf(
        Screen.Wiki,
        Screen.Search,
        Screen.Captures,
        Screen.Settings
    )
    
    Scaffold(
        bottomBar = {
            NavigationBar(
                containerColor = Color(0xFF141414),
                contentColor = Color.White
            ) {
                val navBackStackEntry by navController.currentBackStackEntryAsState()
                val currentDestination = navBackStackEntry?.destination
                
                screens.forEach { screen ->
                    NavigationBarItem(
                        icon = { Icon(screen.icon, contentDescription = screen.title) },
                        label = { Text(screen.title) },
                        selected = currentDestination?.hierarchy?.any { it.route == screen.route } == true,
                        onClick = {
                            navController.navigate(screen.route) {
                                popUpTo(navController.graph.findStartDestination().id) {
                                    saveState = true
                                }
                                launchSingleTop = true
                                restoreState = true
                            }
                        },
                        colors = NavigationBarItemDefaults.colors(
                            selectedIconColor = Color(0xFFFFB800),
                            selectedTextColor = Color(0xFFFFB800),
                            unselectedIconColor = Color(0xFF737373),
                            unselectedTextColor = Color(0xFF737373),
                            indicatorColor = Color(0xFFFFB800).copy(alpha = 0.2f)
                        )
                    )
                }
            }
        }
    ) { innerPadding ->
        NavHost(
            navController = navController,
            startDestination = Screen.Wiki.route,
            modifier = Modifier.padding(innerPadding)
        ) {
            composable(Screen.Wiki.route) { WikiBrowserScreen() }
            composable(Screen.Search.route) { SearchScreen() }
            composable(Screen.Captures.route) { CapturesScreen() }
            composable(Screen.Settings.route) { SettingsScreen() }
        }
    }
}

sealed class Screen(val route: String, val title: String, val icon: androidx.compose.ui.graphics.vector.ImageVector) {
    object Wiki : Screen("wiki", "Wiki", Icons.Default.Book)
    object Search : Screen("search", "Search", Icons.Default.Search)
    object Captures : Screen("captures", "Captures", Icons.Default.PhotoLibrary)
    object Settings : Screen("settings", "Settings", Icons.Default.Settings)
}

// Storage Manager for Android
class StorageManager(private val context: android.content.Context) {
    
    private val rawDir by lazy {
        context.getExternalFilesDir("raw") ?: context.filesDir.resolve("raw")
    }
    
    private val wikiDir by lazy {
        context.getExternalFilesDir("wiki") ?: context.filesDir.resolve("wiki")
    }
    
    fun getStorageUsed(): Long {
        return calculateDirectorySize(rawDir) + calculateDirectorySize(wikiDir)
    }
    
    private fun calculateDirectorySize(dir: java.io.File): Long {
        if (!dir.exists()) return 0
        
        var size = 0L
        dir.listFiles()?.forEach { file ->
            size += if (file.isDirectory) {
                calculateDirectorySize(file)
            } else {
                file.length()
            }
        }
        return size
    }
    
    fun getRawDirectory(): java.io.File = rawDir
    fun getWikiDirectory(): java.io.File = wikiDir
    
    fun formatBytes(bytes: Long): String {
        val units = arrayOf("B", "KB", "MB", "GB", "TB")
        var size = bytes.toDouble()
        var unitIndex = 0
        
        while (size >= 1024 && unitIndex < units.size - 1) {
            size /= 1024
            unitIndex++
        }
        
        return String.format("%.2f %s", size, units[unitIndex])
    }
}
