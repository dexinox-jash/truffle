// SPEC v2.0 SECTION 5.3: Knowledge Graph Commands
// Tauri command handlers for knowledge graph operations

use tauri::State;
use uuid::Uuid;
use crate::AppState;
use crate::models::*;

// ==================== ENTITY COMMANDS ====================

#[tauri::command]
pub async fn get_entities(
    state: State<'_, AppState>,
    filter: Option<EntityFilter>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<EntitySummary>, String> {
    state.db
        .get_entities(filter, limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_entity(
    state: State<'_, AppState>,
    id: String,
) -> Result<Entity, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .get_entity(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_entities(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<EntitySummary>, String> {
    state.db
        .search_entities(&query, limit.unwrap_or(10))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_entity(
    state: State<'_, AppState>,
    input: CreateEntityInput,
) -> Result<Entity, String> {
    state.db
        .create_entity(input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_entity(
    state: State<'_, AppState>,
    id: String,
    input: UpdateEntityInput,
) -> Result<Entity, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .update_entity(uuid, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_entity(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.db
        .delete_entity(uuid)
        .await
        .map_err(|e| e.to_string())
}

// ==================== RELATIONSHIP COMMANDS ====================

#[tauri::command]
pub async fn get_relationships(
    state: State<'_, AppState>,
    entity_id: String,
) -> Result<Vec<Relationship>, String> {
    let uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    state.db
        .get_relationships(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_relationship(
    state: State<'_, AppState>,
    input: CreateRelationshipInput,
) -> Result<Relationship, String> {
    state.db
        .create_relationship(input)
        .await
        .map_err(|e| e.to_string())
}

// ==================== GRAPH TRAVERSAL COMMANDS ====================

#[tauri::command]
pub async fn get_entity_graph(
    state: State<'_, AppState>,
    entity_id: String,
    depth: Option<usize>,
) -> Result<GraphNode, String> {
    let uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    state.db
        .get_entity_graph(uuid, depth.unwrap_or(2))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_path(
    state: State<'_, AppState>,
    from_id: String,
    to_id: String,
    max_depth: Option<usize>,
) -> Result<Vec<Entity>, String> {
    let from_uuid = Uuid::parse_str(&from_id).map_err(|e| e.to_string())?;
    let to_uuid = Uuid::parse_str(&to_id).map_err(|e| e.to_string())?;
    state.db
        .find_path(from_uuid, to_uuid, max_depth.unwrap_or(5))
        .await
        .map_err(|e| e.to_string())
}

// ==================== MEETING COMMANDS ====================

#[tauri::command]
pub async fn get_meetings(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<Meeting>, String> {
    state.db
        .get_meetings(limit.unwrap_or(50))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_from_meeting(
    state: State<'_, AppState>,
    meeting_id: String,
) -> Result<ExtractionResult, String> {
    let uuid = Uuid::parse_str(&meeting_id).map_err(|e| e.to_string())?;
    state.db
        .extract_from_meeting(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_decisions(
    state: State<'_, AppState>,
    entity_id: Option<String>,
) -> Result<Vec<Decision>, String> {
    let uuid = entity_id.and_then(|id| Uuid::parse_str(&id).ok());
    state.db
        .get_decisions(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_action_items(
    state: State<'_, AppState>,
    entity_id: Option<String>,
    status: Option<String>,
) -> Result<Vec<ActionItem>, String> {
    let uuid = entity_id.and_then(|id| Uuid::parse_str(&id).ok());
    let status_enum = status.and_then(|s| match s.as_str() {
        "pending" => Some(ActionItemStatus::Pending),
        "in_progress" => Some(ActionItemStatus::InProgress),
        "completed" => Some(ActionItemStatus::Completed),
        "cancelled" => Some(ActionItemStatus::Cancelled),
        _ => None,
    });
    state.db
        .get_action_items(uuid, status_enum)
        .await
        .map_err(|e| e.to_string())
}
