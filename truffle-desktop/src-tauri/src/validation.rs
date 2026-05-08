// SPEC v2.0 SECTION 5.4: Validation Commands
// Tauri command handlers for validation and quality operations

use tauri::State;
use uuid::Uuid;
use crate::AppState;
use crate::models::*;

// ==================== VALIDATION COMMANDS ====================

#[tauri::command]
pub async fn validate_entity(
    state: State<'_, AppState>,
    entity_id: String,
) -> Result<ValidationResult, String> {
    let uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    state.db
        .validate_entity(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_contradictions(
    state: State<'_, AppState>,
    entity_id: Option<String>,
    severity: Option<String>,
) -> Result<Vec<Contradiction>, String> {
    let uuid = entity_id.and_then(|id| Uuid::parse_str(&id).ok());
    let severity_enum = severity.and_then(|s| match s.as_str() {
        "low" => Some(ContradictionSeverity::Low),
        "medium" => Some(ContradictionSeverity::Medium),
        "high" => Some(ContradictionSeverity::High),
        "critical" => Some(ContradictionSeverity::Critical),
        _ => None,
    });
    state.db
        .get_contradictions(uuid, severity_enum)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_confidence_score(
    state: State<'_, AppState>,
    entity_id: String,
) -> Result<ConfidenceScore, String> {
    let uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    state.db
        .get_confidence_score(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_duplicates(
    state: State<'_, AppState>,
    threshold: Option<f32>,
) -> Result<Vec<DuplicateCandidate>, String> {
    state.db
        .find_duplicates(threshold.unwrap_or(0.85))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_entities(
    state: State<'_, AppState>,
    primary_id: String,
    secondary_id: String,
) -> Result<Entity, String> {
    let primary_uuid = Uuid::parse_str(&primary_id).map_err(|e| e.to_string())?;
    let secondary_uuid = Uuid::parse_str(&secondary_id).map_err(|e| e.to_string())?;
    state.db
        .merge_entities(primary_uuid, secondary_uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_contradiction(
    state: State<'_, AppState>,
    contradiction_id: String,
    resolution: ResolutionInput,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&contradiction_id).map_err(|e| e.to_string())?;
    state.db
        .resolve_contradiction(uuid, resolution)
        .await
        .map_err(|e| e.to_string())
}
