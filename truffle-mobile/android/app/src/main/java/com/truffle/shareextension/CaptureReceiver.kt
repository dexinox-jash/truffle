package com.truffle.shareextension

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

/**
 * CaptureReceiver - Receives capture notifications from system
 * 
 * SPEC v2.0 COMPLIANCE:
 * - Receive screenshot detection (on supported devices)
 * - Trigger immediate capture processing
 * - Handle background capture events
 */
class CaptureReceiver : BroadcastReceiver() {
    
    companion object {
        private const val TAG = "CaptureReceiver"
        const val ACTION_NEW_CAPTURE = "io.truffle.NEW_CAPTURE"
        const val ACTION_PROCESS_COMPLETE = "io.truffle.PROCESS_COMPLETE"
        const val EXTRA_CAPTURE_ID = "capture_id"
        const val EXTRA_FILE_PATH = "file_path"
    }
    
    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            ACTION_NEW_CAPTURE -> {
                val captureId = intent.getStringExtra(EXTRA_CAPTURE_ID)
                val filePath = intent.getStringExtra(EXTRA_FILE_PATH)
                
                Log.d(TAG, "New capture received: $captureId at $filePath")
                
                // Trigger background processing
                triggerCompilation(context)
            }
            
            ACTION_PROCESS_COMPLETE -> {
                val captureId = intent.getStringExtra(EXTRA_CAPTURE_ID)
                Log.d(TAG, "Capture processing complete: $captureId")
                
                // Notify UI of completion
                notifyUI(context, captureId)
            }
            
            Intent.ACTION_SCREEN_OFF -> {
                // Screen turned off - good time to do background work
                scheduleBackgroundWork(context)
            }
            
            Intent.ACTION_POWER_CONNECTED -> {
                // Device charging - good time for intensive work
                scheduleIntensiveWork(context)
            }
        }
    }
    
    private fun triggerCompilation(context: Context) {
        val workRequest = androidx.work.OneTimeWorkRequestBuilder<com.truffle.workmanager.CompilationWorker>()
            .setConstraints(
                androidx.work.Constraints.Builder()
                    .setRequiresBatteryNotLow(true)
                    .build()
            )
            .addTag("truffle_compilation")
            .build()
        
        androidx.work.WorkManager.getInstance(context).enqueue(workRequest)
    }
    
    private fun scheduleBackgroundWork(context: Context) {
        // Schedule cleanup and maintenance
        val cleanupWork = androidx.work.OneTimeWorkRequestBuilder<com.truffle.workmanager.CleanupWorker>()
            .setConstraints(
                androidx.work.Constraints.Builder()
                    .setRequiresDeviceIdle(true)
                    .build()
            )
            .addTag("truffle_cleanup")
            .build()
        
        androidx.work.WorkManager.getInstance(context).enqueue(cleanupWork)
    }
    
    private fun scheduleIntensiveWork(context: Context) {
        // Schedule intensive compilation when charging
        val compilationWork = androidx.work.PeriodicWorkRequestBuilder<com.truffle.workmanager.CompilationWorker>(
            15, java.util.concurrent.TimeUnit.MINUTES
        )
            .setConstraints(
                androidx.work.Constraints.Builder()
                    .setRequiresCharging(true)
                    .build()
            )
            .addTag("truffle_periodic_compilation")
            .build()
        
        androidx.work.WorkManager.getInstance(context).enqueueUniquePeriodicWork(
            "truffle_periodic_compilation",
            androidx.work.ExistingPeriodicWorkPolicy.KEEP,
            compilationWork
        )
    }
    
    private fun notifyUI(context: Context, captureId: String?) {
        // Send local broadcast to update UI
        val uiIntent = Intent("io.truffle.UI_UPDATE").apply {
            putExtra(EXTRA_CAPTURE_ID, captureId)
        }
        androidx.localbroadcastmanager.content.LocalBroadcastManager.getInstance(context)
            .sendBroadcast(uiIntent)
    }
}

/**
 * CleanupWorker - Background cleanup task
 */
class CleanupWorker(context: Context, params: androidx.work.WorkerParameters) : 
    androidx.work.CoroutineWorker(context, params) {
    
    override suspend fun doWork(): Result {
        return try {
            // Clean up old checkpoints
            cleanupCheckpoints()
            
            // Clean up failed captures older than 30 days
            cleanupOldCaptures()
            
            // Clean up temporary files
            cleanupTempFiles()
            
            Result.success()
        } catch (e: Exception) {
            Result.failure()
        }
    }
    
    private fun cleanupCheckpoints() {
        val checkpointDir = applicationContext.filesDir.resolve("checkpoints")
        val oneWeekAgo = System.currentTimeMillis() - (7 * 24 * 60 * 60 * 1000)
        
        checkpointDir.listFiles()?.forEach { file ->
            if (file.lastModified() < oneWeekAgo) {
                file.delete()
            }
        }
    }
    
    private fun cleanupOldCaptures() {
        val db = CaptureDatabase(applicationContext)
        val thirtyDaysAgo = System.currentTimeMillis() - (30L * 24 * 60 * 60 * 1000)
        
        db.writableDatabase.delete(
            "captures",
            "status = ? AND captured_at < ?",
            arrayOf(CaptureStatus.FAILED.name, thirtyDaysAgo.toString())
        )
        db.close()
    }
    
    private fun cleanupTempFiles() {
        val tempDir = applicationContext.cacheDir
        tempDir.listFiles()?.forEach { file ->
            if (file.name.startsWith("truffle_temp_")) {
                file.delete()
            }
        }
    }
}
