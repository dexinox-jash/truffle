// SPEC v2.0 SECTION 5.2: Tauri Commands
// Command handlers for frontend-backend communication

use tauri::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;
use crate::models::*;

// ==================== RAW ARTIFACT COMMANDS ====================

#[tauri::command]
pub async fn get_raw_artifacts(
    state: State<'_, AppState>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<RawArtifact>, String> {
    state.db
        .get_raw_artifacts(limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_raw_artifact(
    state: State<'_, AppState>,
    id: String,
) -> Result<RawArtifact, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .get_raw_artifact(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_raw_artifact(
    state: State<'_, AppState>,
    path: String,
    app_context: Option<AppContext>,
) -> Result<RawArtifact, String> {
    state.db
        .import_raw_artifact(&path, app_context)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_raw_artifact(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .delete_raw_artifact(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_thumbnail(
    state: State<'_, AppState>,
    id: String,
    width: Option<u32>,
) -> Result<Vec<u8>, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .get_thumbnail(uuid, width.unwrap_or(256))
        .await
        .map_err(|e| e.to_string())
}

// ==================== WIKI NODE COMMANDS ====================

#[tauri::command]
pub async fn get_wiki_nodes(
    state: State<'_, AppState>,
    node_type: Option<String>,
) -> Result<Vec<WikiNodeSummary>, String> {
    state.db
        .get_wiki_nodes(node_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_wiki_node(
    state: State<'_, AppState>,
    id: String,
) -> Result<WikiNode, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .get_wiki_node(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_wiki_node(
    state: State<'_, AppState>,
    title: String,
    content: String,
    node_type: String,
) -> Result<WikiNode, String> {
    state.db
        .create_wiki_node(&title, &content, &node_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_wiki_node(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    content: Option<String>,
) -> Result<WikiNode, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .update_wiki_node(uuid, title.as_deref(), content.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_wiki_node(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .delete_wiki_node(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_wiki_nodes(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<WikiNodeSummary>, String> {
    state.db
        .search_wiki_nodes(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_backlinks(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<WikiLink>, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .get_backlinks(uuid)
        .await
        .map_err(|e| e.to_string())
}

// ==================== COMPILATION COMMANDS ====================

#[tauri::command]
pub async fn compile_artifact(
    state: State<'_, AppState>,
    id: String,
) -> Result<CompilationResult, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.compiler
        .compile_artifact(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn compile_all_pending(
    state: State<'_, AppState>,
) -> Result<BatchCompilationResult, String> {
    state.compiler
        .compile_all_pending()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_compilation_status(
    state: State<'_, AppState>,
) -> Result<CompilationStatus, String> {
    state.compiler
        .get_status()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_compilation_logs(
    state: State<'_, AppState>,
    artifact_id: Option<String>,
) -> Result<Vec<CompilationLog>, String> {
    let uuid = artifact_id.and_then(|id| Uuid::parse_str(&id).ok());
    state.db
        .get_compilation_logs(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_compilation(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.compiler
        .cancel()
        .await
        .map_err(|e| e.to_string())
}

// ==================== SEARCH COMMANDS ====================

#[tauri::command]
pub async fn search_content(
    state: State<'_, AppState>,
    query: String,
    search_type: String,
) -> Result<SearchResults, String> {
    state.db
        .search_content(&query, &search_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn semantic_search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SemanticSearchResult>, String> {
    state.db
        .semantic_search(&query, limit.unwrap_or(10))
        .await
        .map_err(|e| e.to_string())
}

// ==================== SYNC COMMANDS ====================

#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, AppState>,
) -> Result<SyncStatus, String> {
    state.sync_manager
        .get_status()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trigger_sync(
    state: State<'_, AppState>,
) -> Result<SyncResult, String> {
    state.sync_manager
        .trigger_sync()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pair_device(
    state: State<'_, AppState>,
) -> Result<PairingInfo, String> {
    state.sync_manager
        .initiate_pairing()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_pairing(
    state: State<'_, AppState>,
    code: String,
) -> Result<bool, String> {
    state.sync_manager
        .verify_pairing(&code)
        .await
        .map_err(|e| e.to_string())
}

// ==================== SCHEMA COMMANDS ====================

#[tauri::command]
pub async fn get_schema(
    state: State<'_, AppState>,
) -> Result<SchemaDefinition, String> {
    state.db
        .get_schema()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_schema(
    state: State<'_, AppState>,
    schema: String,
) -> Result<(), String> {
    state.db
        .update_schema(&schema)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_schema(
    state: State<'_, AppState>,
    schema: String,
) -> Result<SchemaValidationResult, String> {
    state.schema
        .validate(&schema)
        .await
        .map_err(|e| e.to_string())
}

// ==================== SYSTEM COMMANDS ====================

#[tauri::command]
pub async fn get_storage_info(
    state: State<'_, AppState>,
) -> Result<StorageInfo, String> {
    state.db
        .get_storage_info()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_to_markdown(
    state: State<'_, AppState>,
    path: String,
) -> Result<ExportResult, String> {
    state.db
        .export_to_markdown(&path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_info() -> Result<AppInfo, String> {
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: "Truffle".to_string(),
        build_date: option_env!("BUILD_DATE").unwrap_or("unknown").to_string(),
    })
}
