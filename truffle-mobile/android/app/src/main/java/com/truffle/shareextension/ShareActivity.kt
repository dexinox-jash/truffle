package com.truffle.shareextension

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.view.View
import android.widget.Button
import android.widget.ProgressBar
import android.widget.TextView
import androidx.core.content.FileProvider
import com.truffle.app.R
import kotlinx.coroutines.*
import java.io.File
import java.io.FileOutputStream
import java.util.*

/**
 * ShareActivity - Android Share Extension Activity
 * 
 * SPEC v2.0 COMPLIANCE:
 * - Receive screenshots via Android Share Sheet
 * - Save to raw/ directory
 * - Trigger background compilation
 * - Storage warnings at 5GB, pause at 10GB
 */
class ShareActivity : Activity() {
    
    private lateinit var titleText: TextView
    private lateinit var progressBar: ProgressBar
    private lateinit var statusText: TextView
    private lateinit var cancelButton: Button
    
    private val coroutineScope = CoroutineScope(Dispatchers.Main + Job())
    private var isCancelled = false
    
    companion object {
        private const val TAG = "TruffleShare"
        private const val FIVE_GB = 5L * 1024 * 1024 * 1024
        private const val TEN_GB = 10L * 1024 * 1024 * 1024
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Set transparent background for overlay effect
        window.setBackgroundDrawableResource(android.R.color.transparent)
        
        setContentView(R.layout.activity_share)
        
        initializeViews()
        
        // Process shared items
        processSharedItems()
    }
    
    private fun initializeViews() {
        titleText = findViewById(R.id.titleText)
        progressBar = findViewById(R.id.progressBar)
        statusText = findViewById(R.id.statusText)
        cancelButton = findViewById(R.id.cancelButton)
        
        cancelButton.setOnClickListener {
            isCancelled = true
            finish()
        }
    }
    
    private fun processSharedItems() {
        coroutineScope.launch {
            try {
                // Check storage first
                when (val storageStatus = checkStorageStatus()) {
                    is StorageStatus.Critical -> {
                        showError("Storage full. Please free up space.")
                        return@launch
                    }
                    is StorageStatus.Warning -> {
                        updateStatus("Warning: Storage at ${formatBytes(storageStatus.bytes)}")
                    }
                    else -> {}
                }
                
                // Get shared items
                val intent = intent
                val action = intent.action
                val type = intent.type
                
                when {
                    Intent.ACTION_SEND == action && type != null -> {
                        if (type.startsWith("image/")) {
                            handleSingleImage(intent)
                        }
                    }
                    Intent.ACTION_SEND_MULTIPLE == action && type != null -> {
                        if (type.startsWith("image/")) {
                            handleMultipleImages(intent)
                        }
                    }
                    else -> {
                        showError("Unsupported content type")
                    }
                }
            } catch (e: Exception) {
                showError("Error: ${e.message}")
            }
        }
    }
    
    private suspend fun handleSingleImage(intent: Intent) {
        val imageUri = intent.getParcelableExtra<Uri>(Intent.EXTRA_STREAM)
        
        if (imageUri == null) {
            showError("No image received")
            return
        }
        
        updateProgress(0)
        updateStatus("Processing image...")
        
        val success = saveImage(imageUri)
        
        if (success) {
            updateProgress(100)
            updateStatus("Saved successfully")
            
            // Trigger haptic feedback
            triggerSuccessFeedback()
            
            // Delay then close
            delay(500)
            finish()
        } else {
            showError("Failed to save image")
        }
    }
    
    private suspend fun handleMultipleImages(intent: Intent) {
        val imageUris = intent.getParcelableArrayListExtra<Uri>(Intent.EXTRA_STREAM)
        
        if (imageUris.isNullOrEmpty()) {
            showError("No images received")
            return
        }
        
        val total = imageUris.size
        var processed = 0
        
        for (uri in imageUris) {
            if (isCancelled) break
            
            val progress = (processed * 100) / total
            updateProgress(progress)
            updateStatus("Processing ${processed + 1} of $total...")
            
            saveImage(uri)
            processed++
        }
        
        updateProgress(100)
        updateStatus("Saved $processed image(s)")
        
        triggerSuccessFeedback()
        delay(500)
        finish()
    }
    
    private suspend fun saveImage(uri: Uri): Boolean = withContext(Dispatchers.IO) {
        try {
            // Create raw directory if needed
            val rawDir = File(filesDir, "raw").apply { mkdirs() }
            
            // Generate filename
            val timestamp = java.text.SimpleDateFormat("yyyyMMdd_HHmmss", Locale.getDefault()).format(Date())
            val filename = "capture_${timestamp}_${UUID.randomUUID().toString().take(8)}.jpg"
            val outputFile = File(rawDir, filename)
            
            // Copy image data
            contentResolver.openInputStream(uri)?.use { input ->
                FileOutputStream(outputFile).use { output ->
                    input.copyTo(output)
                }
            }
            
            // Add to queue database
            addToQueue(outputFile, filename)
            
            // Trigger background compilation
            triggerBackgroundCompilation()
            
            true
        } catch (e: Exception) {
            e.printStackTrace()
            false
        }
    }
    
    private fun addToQueue(file: File, filename: String) {
        // Add to SQLite queue database
        val db = CaptureDatabase(this)
        db.addCapture(
            CaptureRecord(
                id = UUID.randomUUID(),
                filename = filename,
                path = file.absolutePath,
                capturedAt = Date(),
                sizeBytes = file.length(),
                status = CaptureStatus.PENDING,
                appContext = getAppContext()
            )
        )
        db.close()
    }
    
    private fun getAppContext(): AppContext? {
        // Get the source app info if available
        val referrer = referrer
        return referrer?.let {
            AppContext(
                packageName = it.host ?: "unknown",
                appName = "Shared from Android",
                windowTitle = null
            )
        }
    }
    
    private fun triggerBackgroundCompilation() {
        // Schedule WorkManager compilation task
        val workRequest = androidx.work.OneTimeWorkRequestBuilder<com.truffle.workmanager.CompilationWorker>()
            .setConstraints(
                androidx.work.Constraints.Builder()
                    .setRequiresBatteryNotLow(true)
                    .build()
            )
            .addTag("truffle_compilation")
            .build()
        
        androidx.work.WorkManager.getInstance(this).enqueue(workRequest)
    }
    
    private suspend fun checkStorageStatus(): StorageStatus = withContext(Dispatchers.IO) {
        val rawDir = File(filesDir, "raw")
        val used = calculateDirectorySize(rawDir)
        
        when {
            used >= TEN_GB -> StorageStatus.Critical
            used >= FIVE_GB -> StorageStatus.Warning(used)
            else -> StorageStatus.Ok
        }
    }
    
    private fun calculateDirectorySize(dir: File): Long {
        if (!dir.exists()) return 0
        
        return dir.listFiles()?.sumOf { file ->
            if (file.isDirectory) {
                calculateDirectorySize(file)
            } else {
                file.length()
            }
        } ?: 0
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
    
    private suspend fun updateProgress(progress: Int) = withContext(Dispatchers.Main) {
        progressBar.progress = progress
    }
    
    private suspend fun updateStatus(message: String) = withContext(Dispatchers.Main) {
        statusText.text = message
    }
    
    private suspend fun showError(message: String) = withContext(Dispatchers.Main) {
        statusText.text = "Error: $message"
        statusText.setTextColor(android.graphics.Color.RED)
        progressBar.progressDrawable.setTint(android.graphics.Color.RED)
        
        delay(2000)
        finish()
    }
    
    private fun triggerSuccessFeedback() {
        // Haptic feedback
        val vibrator = getSystemService(VIBRATOR_SERVICE) as android.os.Vibrator
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
            vibrator.vibrate(android.os.VibrationEffect.createOneShot(50, android.os.VibrationEffect.DEFAULT_AMPLITUDE))
        } else {
            @Suppress("DEPRECATION")
            vibrator.vibrate(50)
        }
    }
    
    override fun onDestroy() {
        super.onDestroy()
        coroutineScope.cancel()
    }
}

// Storage status
sealed class StorageStatus {
    object Ok : StorageStatus()
    data class Warning(val bytes: Long) : StorageStatus()
    object Critical : StorageStatus()
}

// Data classes
data class CaptureRecord(
    val id: UUID,
    val filename: String,
    val path: String,
    val capturedAt: Date,
    val sizeBytes: Long,
    val status: CaptureStatus,
    val appContext: AppContext?
)

data class AppContext(
    val packageName: String,
    val appName: String,
    val windowTitle: String?
)

enum class CaptureStatus {
    PENDING, PROCESSING, COMPLETED, FAILED, QUARANTINED
}

// Database helper
class CaptureDatabase(context: android.content.Context) : android.database.sqlite.SQLiteOpenHelper(
    context, "captures.db", null, 1
) {
    override fun onCreate(db: android.database.sqlite.SQLiteDatabase) {
        db.execSQL("""
            CREATE TABLE captures (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                path TEXT NOT NULL,
                captured_at INTEGER NOT NULL,
                size_bytes INTEGER NOT NULL,
                status TEXT NOT NULL,
                retry_count INTEGER DEFAULT 0,
                checkpoint_data TEXT
            )
        """)
    }
    
    override fun onUpgrade(db: android.database.sqlite.SQLiteDatabase, oldVersion: Int, newVersion: Int) {}
    
    fun addCapture(capture: CaptureRecord) {
        val values = android.content.ContentValues().apply {
            put("id", capture.id.toString())
            put("filename", capture.filename)
            put("path", capture.path)
            put("captured_at", capture.capturedAt.time)
            put("size_bytes", capture.sizeBytes)
            put("status", capture.status.name)
        }
        writableDatabase.insert("captures", null, values)
    }
}
