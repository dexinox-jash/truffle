package com.truffle.workmanager

import android.content.Context
import android.graphics.BitmapFactory
import androidx.work.*
import com.google.mlkit.vision.common.InputImage
import com.google.mlkit.vision.text.TextRecognition
import com.google.mlkit.vision.text.latin.TextRecognizerOptions
import kotlinx.coroutines.*
import org.json.JSONObject
import java.io.File
import java.util.*
import java.util.concurrent.TimeUnit

/**
 * CompilationWorker - Background Compilation with Checkpoint/Resume
 * 
 * SPEC v2.0 COMPLIANCE:
 * - 30-second execution budget per capture (checkpoint/resume)
 * - OCR, classification, extraction, linking, persistence stages
 * - Resume from checkpoint on next run
 * - Battery-aware scheduling
 */
class CompilationWorker(
    context: Context,
    params: WorkerParameters
) : CoroutineWorker(context, params) {
    
    companion object {
        private const val TAG = "CompilationWorker"
        private const val MAX_EXECUTION_TIME_MS = 25000L // 25s (5s buffer)
        private const val CHECKPOINT_DIR = "checkpoints"
        
        // Work tags
        const val TAG_COMPILATION = "truffle_compilation"
        const val TAG_PERIODIC = "truffle_periodic_compilation"
        
        fun schedule(context: Context) {
            val constraints = Constraints.Builder()
                .setRequiresBatteryNotLow(true)
                .setRequiresStorageNotLow(true)
                .build()
            
            val workRequest = PeriodicWorkRequestBuilder<CompilationWorker>(
                15, TimeUnit.MINUTES
            )
                .setConstraints(constraints)
                .addTag(TAG_PERIODIC)
                .build()
            
            WorkManager.getInstance(context).enqueueUniquePeriodicWork(
                TAG_PERIODIC,
                ExistingPeriodicWorkPolicy.KEEP,
                workRequest
            )
        }
    }
    
    private val checkpointManager = CheckpointManager(applicationContext)
    private val startTime = System.currentTimeMillis()
    
    override suspend fun doWork(): Result {
        return try {
            setForeground(createForegroundInfo())
            
            // Get pending captures
            val pendingCaptures = getPendingCaptures()
            
            if (pendingCaptures.isEmpty()) {
                return Result.success()
            }
            
            var processedCount = 0
            var failedCount = 0
            
            for (capture in pendingCaptures) {
                // Check if we're running out of time
                if (System.currentTimeMillis() - startTime > MAX_EXECUTION_TIME_MS) {
                    // Save checkpoint and retry later
                    return Result.retry()
                }
                
                try {
                    val success = processCapture(capture)
                    if (success) {
                        processedCount++
                    } else {
                        failedCount++
                    }
                } catch (e: Exception) {
                    failedCount++
                    incrementRetryCount(capture.id)
                }
            }
            
            // Return success if we processed anything, retry if all failed
            when {
                processedCount > 0 -> Result.success()
                failedCount > 0 && processedCount == 0 -> Result.retry()
                else -> Result.success()
            }
            
        } catch (e: Exception) {
            Result.retry()
        }
    }
    
    private suspend fun processCapture(capture: CaptureRecord): Boolean {
        // Update status to processing
        updateCaptureStatus(capture.id, CaptureStatus.PROCESSING)
        
        // Check for existing checkpoint
        val checkpoint = checkpointManager.loadCheckpoint(capture.id)
        val startStage = checkpoint?.stage ?: CompilationStage.OCR
        
        try {
            // Stage 1: OCR (if needed)
            if (startStage <= CompilationStage.OCR) {
                if (!performOCR(capture)) {
                    saveCheckpoint(capture.id, CompilationStage.OCR, 0.2)
                    return false
                }
                saveCheckpoint(capture.id, CompilationStage.CLASSIFICATION, 0.2)
            }
            
            // Stage 2: Classification
            if (startStage <= CompilationStage.CLASSIFICATION) {
                if (!classifyCapture(capture)) {
                    saveCheckpoint(capture.id, CompilationStage.CLASSIFICATION, 0.4)
                    return false
                }
                saveCheckpoint(capture.id, CompilationStage.EXTRACTION, 0.4)
            }
            
            // Stage 3: Entity Extraction
            if (startStage <= CompilationStage.EXTRACTION) {
                if (!extractEntities(capture)) {
                    saveCheckpoint(capture.id, CompilationStage.EXTRACTION, 0.6)
                    return false
                }
                saveCheckpoint(capture.id, CompilationStage.LINKING, 0.6)
            }
            
            // Stage 4: Linking
            if (startStage <= CompilationStage.LINKING) {
                if (!linkEntities(capture)) {
                    saveCheckpoint(capture.id, CompilationStage.LINKING, 0.8)
                    return false
                }
                saveCheckpoint(capture.id, CompilationStage.PERSISTENCE, 0.8)
            }
            
            // Stage 5: Persistence
            if (startStage <= CompilationStage.PERSISTENCE) {
                if (!saveToWiki(capture)) {
                    saveCheckpoint(capture.id, CompilationStage.PERSISTENCE, 0.9)
                    return false
                }
            }
            
            // Clear checkpoint on success
            checkpointManager.deleteCheckpoint(capture.id)
            
            // Update status to completed
            updateCaptureStatus(capture.id, CaptureStatus.COMPLETED)
            
            return true
            
        } catch (e: Exception) {
            // Save checkpoint for retry
            saveCheckpoint(capture.id, startStage, 0.0)
            throw e
        }
    }
    
    private suspend fun performOCR(capture: CaptureRecord): Boolean {
        return try {
            val file = File(capture.path)
            if (!file.exists()) return false
            
            val image = InputImage.fromFilePath(applicationContext, android.net.Uri.fromFile(file))
            val recognizer = TextRecognition.getClient(TextRecognizerOptions.DEFAULT_OPTIONS)
            
            val result = recognizer.process(image).await()
            val ocrText = result.text
            
            // Store OCR text
            storeOCRResult(capture.id, ocrText)
            
            true
        } catch (e: Exception) {
            false
        }
    }
    
    private suspend fun classifyCapture(capture: CaptureRecord): Boolean {
        // Use on-device ML model to classify screenshot type
        // Returns: receipt, contact, travel, screenshot, etc.
        
        // For now, use simple heuristics based on OCR text
        val ocrText = getOCRText(capture.id) ?: return false
        
        val classification = when {
            ocrText.contains("receipt", ignoreCase = true) ||
            ocrText.contains("total", ignoreCase = true) -> "receipt"
            
            ocrText.contains("flight", ignoreCase = true) ||
            ocrText.contains("boarding", ignoreCase = true) -> "travel"
            
            ocrText.contains("contact", ignoreCase = true) ||
            ocrText.contains("phone", ignoreCase = true) -> "contact"
            
            else -> "screenshot"
        }
        
        storeClassification(capture.id, classification)
        return true
    }
    
    private suspend fun extractEntities(capture: CaptureRecord): Boolean {
        val ocrText = getOCRText(capture.id) ?: return false
        val classification = getClassification(capture.id) ?: "screenshot"
        
        val entities = when (classification) {
            "receipt" -> extractReceiptEntities(ocrText)
            "travel" -> extractTravelEntities(ocrText)
            "contact" -> extractContactEntities(ocrText)
            else -> emptyMap()
        }
        
        storeEntities(capture.id, entities)
        return true
    }
    
    private fun extractReceiptEntities(text: String): Map<String, String> {
        val entities = mutableMapOf<String, String>()
        
        // Extract total amount
        val totalRegex = "total[\\s:]*\\$?([\\d,.]+)".toRegex(RegexOption.IGNORE_CASE)
        totalRegex.find(text)?.let {
            entities["total"] = it.groupValues[1]
        }
        
        // Extract merchant name (first line often contains it)
        text.lines().firstOrNull()?.let {
            entities["merchant"] = it.trim()
        }
        
        // Extract date
        val dateRegex = "(\\d{1,2}[/-]\\d{1,2}[/-]\\d{2,4})".toRegex()
        dateRegex.find(text)?.let {
            entities["date"] = it.groupValues[1]
        }
        
        return entities
    }
    
    private fun extractTravelEntities(text: String): Map<String, String> {
        val entities = mutableMapOf<String, String>()
        
        // Extract flight number
        val flightRegex = "([A-Z]{2}\\d{3,4})".toRegex()
        flightRegex.find(text)?.let {
            entities["flight_number"] = it.groupValues[1]
        }
        
        // Extract confirmation code
        val confirmRegex = "confirmation[\\s:]*([A-Z0-9]{6})".toRegex(RegexOption.IGNORE_CASE)
        confirmRegex.find(text)?.let {
            entities["confirmation"] = it.groupValues[1]
        }
        
        return entities
    }
    
    private fun extractContactEntities(text: String): Map<String, String> {
        val entities = mutableMapOf<String, String>()
        
        // Extract phone number
        val phoneRegex = "(\\+?\\d{1,3}[-.\\s]?\\(?\\d{3}\\)?[-.\\s]?\\d{3}[-.\\s]?\\d{4})".toRegex()
        phoneRegex.find(text)?.let {
            entities["phone"] = it.groupValues[1]
        }
        
        // Extract email
        val emailRegex = "([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,})".toRegex()
        emailRegex.find(text)?.let {
            entities["email"] = it.groupValues[1]
        }
        
        return entities
    }
    
    private suspend fun linkEntities(capture: CaptureRecord): Boolean {
        // Link extracted entities to existing wiki nodes
        // Create backlinks and forward links
        
        val entities = getEntities(capture.id) ?: return true
        
        // Find or create wiki nodes for entities
        for ((key, value) in entities) {
            linkToWikiNode(capture.id, key, value)
        }
        
        return true
    }
    
    private suspend fun saveToWiki(capture: CaptureRecord): Boolean {
        // Generate wiki markdown from extracted entities
        // Save to wiki directory
        
        val entities = getEntities(capture.id) ?: emptyMap()
        val classification = getClassification(capture.id) ?: "screenshot"
        
        val markdown = generateWikiMarkdown(capture, classification, entities)
        
        val wikiDir = File(applicationContext.filesDir, "wiki").apply { mkdirs() }
        val wikiFile = File(wikiDir, "${capture.id}.md")
        
        wikiFile.writeText(markdown)
        
        return true
    }
    
    private fun generateWikiMarkdown(
        capture: CaptureRecord,
        classification: String,
        entities: Map<String, String>
    ): String {
        val sb = StringBuilder()
        
        sb.appendLine("# ${classification.capitalize()}: ${capture.filename}")
        sb.appendLine()
        sb.appendLine("**Captured:** ${Date(capture.capturedAt)}")
        sb.appendLine("**Type:** $classification")
        sb.appendLine()
        sb.appendLine("## Extracted Information")
        sb.appendLine()
        
        for ((key, value) in entities) {
            sb.appendLine("- **${key.capitalize()}:** $value")
        }
        
        sb.appendLine()
        sb.appendLine("## Source")
        sb.appendLine()
        sb.appendLine("![Screenshot](${capture.path})")
        
        return sb.toString()
    }
    
    // Checkpoint management
    private fun saveCheckpoint(captureId: UUID, stage: CompilationStage, progress: Double) {
        val checkpoint = CompilationCheckpoint(
            captureId = captureId,
            stage = stage,
            progress = progress,
            timestamp = System.currentTimeMillis()
        )
        checkpointManager.saveCheckpoint(checkpoint)
    }
    
    // Database operations
    private fun getPendingCaptures(): List<CaptureRecord> {
        val db = CaptureDatabase(applicationContext)
        val captures = db.getPendingCaptures()
        db.close()
        return captures
    }
    
    private fun updateCaptureStatus(id: UUID, status: CaptureStatus) {
        val db = CaptureDatabase(applicationContext)
        db.updateStatus(id, status)
        db.close()
    }
    
    private fun incrementRetryCount(id: UUID) {
        val db = CaptureDatabase(applicationContext)
        db.incrementRetryCount(id)
        db.close()
    }
    
    private fun storeOCRResult(id: UUID, text: String) {
        val db = CaptureDatabase(applicationContext)
        db.storeOCRText(id, text)
        db.close()
    }
    
    private fun getOCRText(id: UUID): String? {
        val db = CaptureDatabase(applicationContext)
        val text = db.getOCRText(id)
        db.close()
        return text
    }
    
    private fun storeClassification(id: UUID, classification: String) {
        val db = CaptureDatabase(applicationContext)
        db.storeClassification(id, classification)
        db.close()
    }
    
    private fun getClassification(id: UUID): String? {
        val db = CaptureDatabase(applicationContext)
        val classification = db.getClassification(id)
        db.close()
        return classification
    }
    
    private fun storeEntities(id: UUID, entities: Map<String, String>) {
        val db = CaptureDatabase(applicationContext)
        db.storeEntities(id, entities)
        db.close()
    }
    
    private fun getEntities(id: UUID): Map<String, String>? {
        val db = CaptureDatabase(applicationContext)
        val entities = db.getEntities(id)
        db.close()
        return entities
    }
    
    private fun linkToWikiNode(captureId: UUID, entityType: String, entityValue: String) {
        // Create or update wiki node links
    }
    
    private fun createForegroundInfo(): ForegroundInfo {
        val notification = androidx.core.app.NotificationCompat.Builder(
            applicationContext,
            "truffle_compilation"
        )
            .setContentTitle("Truffle")
            .setContentText("Compiling screenshots...")
            .setSmallIcon(android.R.drawable.ic_menu_gallery)
            .setOngoing(true)
            .build()
        
        return ForegroundInfo(1, notification)
    }
}

// Compilation stages
enum class CompilationStage {
    OCR, CLASSIFICATION, EXTRACTION, LINKING, PERSISTENCE
}

// Checkpoint data
data class CompilationCheckpoint(
    val captureId: UUID,
    val stage: CompilationStage,
    val progress: Double,
    val timestamp: Long
)

// Checkpoint manager
class CheckpointManager(private val context: Context) {
    private val checkpointDir = File(context.filesDir, "checkpoints").apply { mkdirs() }
    
    fun saveCheckpoint(checkpoint: CompilationCheckpoint) {
        val file = File(checkpointDir, "${checkpoint.captureId}.json")
        val json = JSONObject().apply {
            put("captureId", checkpoint.captureId.toString())
            put("stage", checkpoint.stage.name)
            put("progress", checkpoint.progress)
            put("timestamp", checkpoint.timestamp)
        }
        file.writeText(json.toString())
    }
    
    fun loadCheckpoint(captureId: UUID): CompilationCheckpoint? {
        val file = File(checkpointDir, "$captureId.json")
        if (!file.exists()) return null
        
        return try {
            val json = JSONObject(file.readText())
            CompilationCheckpoint(
                captureId = UUID.fromString(json.getString("captureId")),
                stage = CompilationStage.valueOf(json.getString("stage")),
                progress = json.getDouble("progress"),
                timestamp = json.getLong("timestamp")
            )
        } catch (e: Exception) {
            null
        }
    }
    
    fun deleteCheckpoint(captureId: UUID) {
        File(checkpointDir, "$captureId.json").delete()
    }
}

// Data classes
enum class CaptureStatus {
    PENDING, PROCESSING, COMPLETED, FAILED, QUARANTINED
}

data class CaptureRecord(
    val id: UUID,
    val filename: String,
    val path: String,
    val capturedAt: Long,
    val sizeBytes: Long,
    val status: CaptureStatus
)

// Database helper (extended)
class CaptureDatabase(context: Context) : android.database.sqlite.SQLiteOpenHelper(
    context, "captures.db", null, 2
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
                ocr_text TEXT,
                classification TEXT,
                entities TEXT
            )
        """)
    }
    
    override fun onUpgrade(db: android.database.sqlite.SQLiteDatabase, oldVersion: Int, newVersion: Int) {
        if (oldVersion < 2) {
            db.execSQL("ALTER TABLE captures ADD COLUMN ocr_text TEXT")
            db.execSQL("ALTER TABLE captures ADD COLUMN classification TEXT")
            db.execSQL("ALTER TABLE captures ADD COLUMN entities TEXT")
        }
    }
    
    fun getPendingCaptures(): List<CaptureRecord> {
        val captures = mutableListOf<CaptureRecord>()
        val cursor = readableDatabase.query(
            "captures",
            null,
            "status = ? AND retry_count < ?",
            arrayOf(CaptureStatus.PENDING.name, "3"),
            null, null,
            "captured_at ASC"
        )
        
        while (cursor.moveToNext()) {
            captures.add(CaptureRecord(
                id = UUID.fromString(cursor.getString(cursor.getColumnIndexOrThrow("id"))),
                filename = cursor.getString(cursor.getColumnIndexOrThrow("filename")),
                path = cursor.getString(cursor.getColumnIndexOrThrow("path")),
                capturedAt = cursor.getLong(cursor.getColumnIndexOrThrow("captured_at")),
                sizeBytes = cursor.getLong(cursor.getColumnIndexOrThrow("size_bytes")),
                status = CaptureStatus.valueOf(cursor.getString(cursor.getColumnIndexOrThrow("status")))
            ))
        }
        cursor.close()
        
        return captures
    }
    
    fun updateStatus(id: UUID, status: CaptureStatus) {
        val values = android.content.ContentValues().apply {
            put("status", status.name)
        }
        writableDatabase.update("captures", values, "id = ?", arrayOf(id.toString()))
    }
    
    fun incrementRetryCount(id: UUID) {
        writableDatabase.execSQL(
            "UPDATE captures SET retry_count = retry_count + 1 WHERE id = ?",
            arrayOf(id.toString())
        )
    }
    
    fun storeOCRText(id: UUID, text: String) {
        val values = android.content.ContentValues().apply {
            put("ocr_text", text)
        }
        writableDatabase.update("captures", values, "id = ?", arrayOf(id.toString()))
    }
    
    fun getOCRText(id: UUID): String? {
        val cursor = readableDatabase.query(
            "captures",
            arrayOf("ocr_text"),
            "id = ?",
            arrayOf(id.toString()),
            null, null, null
        )
        val text = if (cursor.moveToFirst()) {
            cursor.getString(cursor.getColumnIndexOrThrow("ocr_text"))
        } else null
        cursor.close()
        return text
    }
    
    fun storeClassification(id: UUID, classification: String) {
        val values = android.content.ContentValues().apply {
            put("classification", classification)
        }
        writableDatabase.update("captures", values, "id = ?", arrayOf(id.toString()))
    }
    
    fun getClassification(id: UUID): String? {
        val cursor = readableDatabase.query(
            "captures",
            arrayOf("classification"),
            "id = ?",
            arrayOf(id.toString()),
            null, null, null
        )
        val classification = if (cursor.moveToFirst()) {
            cursor.getString(cursor.getColumnIndexOrThrow("classification"))
        } else null
        cursor.close()
        return classification
    }
    
    fun storeEntities(id: UUID, entities: Map<String, String>) {
        val values = android.content.ContentValues().apply {
            put("entities", JSONObject(entities).toString())
        }
        writableDatabase.update("captures", values, "id = ?", arrayOf(id.toString()))
    }
    
    fun getEntities(id: UUID): Map<String, String>? {
        val cursor = readableDatabase.query(
            "captures",
            arrayOf("entities"),
            "id = ?",
            arrayOf(id.toString()),
            null, null, null
        )
        val entities = if (cursor.moveToFirst()) {
            val jsonStr = cursor.getString(cursor.getColumnIndexOrThrow("entities"))
            jsonStr?.let {
                val json = JSONObject(it)
                json.keys().asSequence().associate { key -> key to json.getString(key) }
            }
        } else null
        cursor.close()
        return entities
    }
}
