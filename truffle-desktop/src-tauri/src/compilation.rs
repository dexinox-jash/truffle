// SPEC v2.0 SECTION 2.2: Compilation Pipeline Module
// AI-powered screenshot compilation

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::database::Database;
use crate::models::*;

pub struct Compiler {
    db: Arc<Database>,
    is_compiling: Arc<RwLock<bool>>,
    current_job: Arc<RwLock<Option<CompilationJob>>>,
    cancel_token: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl Compiler {
    pub async fn new(db: Arc<Database>) -> Result<Self> {
        Ok(Self {
            db,
            is_compiling: Arc::new(RwLock::new(false)),
            current_job: Arc::new(RwLock::new(None)),
            cancel_token: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn compile_artifact(&self, artifact_id: Uuid) -> Result<CompilationResult> {
        // Check if already compiling
        {
            let is_compiling = self.is_compiling.read().await;
            if *is_compiling {
                return Err(anyhow::anyhow!("Compilation already in progress"));
            }
        }

        // Set compiling flag
        {
            let mut is_compiling = self.is_compiling.write().await;
            *is_compiling = true;
        }

        // Create cancel channel
        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
        {
            let mut token = self.cancel_token.lock().await;
            *token = Some(cancel_tx);
        }

        // Create job
        let job = CompilationJob {
            id: Uuid::new_v4().to_string(),
            artifact_id: artifact_id.to_string(),
            status: CompilationStatus::Processing,
            progress: 0.0,
            started_at: Some(chrono::Utc::now().to_rfc3339()),
            completed_at: None,
            error: None,
        };

        {
            let mut current = self.current_job.write().await;
            *current = Some(job.clone());
        }

        info!("Starting compilation for artifact: {}", artifact_id);

        let start_time = std::time::Instant::now();

        // Simulate compilation (replace with actual Gemma 4 integration)
        let result = self.run_compilation(artifact_id, cancel_rx).await;

        // Clean up
        {
            let mut is_compiling = self.is_compiling.write().await;
            *is_compiling = false;
        }
        {
            let mut current = self.current_job.write().await;
            *current = None;
        }
        {
            let mut token = self.cancel_token.lock().await;
            *token = None;
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(nodes) => {
                info!(
                    "Compilation completed for artifact: {} in {}ms, created {} nodes",
                    artifact_id, duration_ms, nodes.len()
                );

                Ok(CompilationResult {
                    artifact_id: artifact_id.to_string(),
                    success: true,
                    nodes_created: nodes,
                    nodes_updated: vec![],
                    duration_ms,
                })
            }
            Err(e) => {
                error!("Compilation failed for artifact: {}: {}", artifact_id, e);

                Ok(CompilationResult {
                    artifact_id: artifact_id.to_string(),
                    success: false,
                    nodes_created: vec![],
                    nodes_updated: vec![],
                    duration_ms,
                })
            }
        }
    }

    pub async fn compile_all_pending(&self) -> Result<BatchCompilationResult> {
        // Get all pending artifacts
        let artifacts = self.db.get_raw_artifacts(1000, 0).await?;
        let pending: Vec<_> = artifacts
            .into_iter()
            .filter(|a| matches!(a.status, ArtifactStatus::Pending))
            .collect();

        let total = pending.len();
        let mut successful = 0;
        let mut failed = 0;
        let mut results = vec![];

        for artifact in pending {
            match self.compile_artifact(Uuid::parse_str(&artifact.uuid)?).await {
                Ok(result) => {
                    if result.success {
                        successful += 1;
                    } else {
                        failed += 1;
                    }
                    results.push(result);
                }
                Err(e) => {
                    failed += 1;
                    results.push(CompilationResult {
                        artifact_id: artifact.uuid,
                        success: false,
                        nodes_created: vec![],
                        nodes_updated: vec![],
                        duration_ms: 0,
                    });
                    warn!("Failed to compile artifact {}: {}", artifact.uuid, e);
                }
            }
        }

        Ok(BatchCompilationResult {
            processed: total,
            successful,
            failed,
            results,
        })
    }

    pub async fn get_status(&self) -> Result<CompilationStatus> {
        let is_compiling = *self.is_compiling.read().await;
        let current = self.current_job.read().await.clone();

        // Get queue length
        let artifacts = self.db.get_raw_artifacts(1000, 0).await?;
        let queue_length = artifacts
            .iter()
            .filter(|a| matches!(a.status, ArtifactStatus::Pending))
            .count();

        Ok(CompilationStatus {
            is_compiling,
            queue_length,
            current_artifact: current.map(|j| j.artifact_id),
            progress_percent: current.map(|j| j.progress).unwrap_or(0.0),
        })
    }

    pub async fn cancel(&self) -> Result<()> {
        let token = self.cancel_token.lock().await;
        if let Some(tx) = token.as_ref() {
            let _ = tx.send(());
        }
        Ok(())
    }

    // ==================== PRIVATE METHODS ====================

    async fn run_compilation(
        &self,
        artifact_id: Uuid,
        mut cancel_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<Vec<String>> {
        // TODO: Integrate with Gemma 4 via llama.cpp
        // This is a simulation

        let stages = vec![
            ("Loading model", 10),
            ("Preprocessing image", 25),
            ("Running OCR", 40),
            ("Analyzing content", 60),
            ("Extracting entities", 75),
            ("Creating wiki nodes", 90),
            ("Finalizing", 100),
        ];

        for (stage, progress) in stages {
            // Check for cancellation
            if cancel_rx.try_recv().is_ok() {
                return Err(anyhow::anyhow!("Compilation cancelled"));
            }

            // Update progress
            {
                let mut current = self.current_job.write().await;
                if let Some(ref mut job) = *current {
                    job.progress = progress as f32;
                }
            }

            info!("Compilation stage: {} ({}%)", stage, progress);

            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }

        // Create mock wiki nodes
        let node_id = Uuid::new_v4().to_string();
        
        // In real implementation, this would:
        // 1. Load the image
        // 2. Run Gemma 4 inference
        // 3. Parse the output
        // 4. Create wiki nodes
        // 5. Update artifact status

        Ok(vec![node_id])
    }
}
