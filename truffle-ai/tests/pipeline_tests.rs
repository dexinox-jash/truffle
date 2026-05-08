//! Pipeline Tests

use truffle_ai::pipeline::{
    CompilationStatus, CompilationPriority, CompilationJob,
    PipelineStats, QueueConfig, AppContext,
};

#[test]
fn test_compilation_status_enum() {
    assert_ne!(CompilationStatus::Pending, CompilationStatus::Completed);
    assert_eq!(CompilationStatus::Failed, CompilationStatus::Failed);
}

#[test]
fn test_compilation_priority_ordering() {
    assert!(CompilationPriority::Immediate < CompilationPriority::High);
    assert!(CompilationPriority::High < CompilationPriority::Normal);
    assert!(CompilationPriority::Normal < CompilationPriority::Low);
}

#[test]
fn test_compilation_priority_values() {
    assert_eq!(CompilationPriority::Immediate as i32, 0);
    assert_eq!(CompilationPriority::High as i32, 1);
    assert_eq!(CompilationPriority::Normal as i32, 2);
    assert_eq!(CompilationPriority::Low as i32, 3);
}

#[test]
fn test_pipeline_stats_default() {
    let stats = PipelineStats::default();
    
    assert_eq!(stats.total_attempted, 0);
    assert_eq!(stats.successful, 0);
    assert_eq!(stats.failed, 0);
    assert_eq!(stats.success_rate(), 100.0);
}

#[test]
fn test_pipeline_stats_success_rate() {
    let mut stats = PipelineStats::default();
    
    stats.total_attempted = 100;
    stats.successful = 95;
    stats.failed = 5;
    
    assert_eq!(stats.success_rate(), 95.0);
}

#[test]
fn test_pipeline_stats_update_avg_time() {
    let mut stats = PipelineStats::default();
    
    stats.update_avg_time(2000);
    assert_eq!(stats.avg_time_ms, 2000);
    assert_eq!(stats.total_attempted, 1);
    
    stats.update_avg_time(3000);
    assert_eq!(stats.avg_time_ms, 2500);
    assert_eq!(stats.total_attempted, 2);
}

#[test]
fn test_queue_config_default() {
    let config = QueueConfig::default();
    
    assert_eq!(config.max_size, 1000);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.retry_delay_secs, 60);
    assert_eq!(config.batch_size, 10);
    assert!(config.auto_retry);
}

#[test]
fn test_app_context_creation() {
    use chrono::Utc;
    
    let ctx = AppContext {
        bundle_id: "com.example.app".to_string(),
        app_name: "Example App".to_string(),
        window_title: Some("Main Window".to_string()),
        captured_at: Utc::now(),
    };
    
    assert_eq!(ctx.bundle_id, "com.example.app");
    assert_eq!(ctx.app_name, "Example App");
    assert_eq!(ctx.window_title, Some("Main Window".to_string()));
}

#[test]
fn test_compilation_job_creation() {
    use uuid::Uuid;
    use std::path::PathBuf;
    use chrono::Utc;
    
    let job = CompilationJob {
        id: Uuid::new_v4(),
        image_path: PathBuf::from("/test/image.png"),
        priority: CompilationPriority::High,
        status: CompilationStatus::Pending,
        retry_count: 0,
        created_at: Utc::now(),
        app_context: None,
    };
    
    assert_eq!(job.status, CompilationStatus::Pending);
    assert_eq!(job.priority, CompilationPriority::High);
    assert_eq!(job.retry_count, 0);
}
