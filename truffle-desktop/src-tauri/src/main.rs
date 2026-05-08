// SPEC v2.0 SECTION 5.2: Tauri Desktop Application Entry Point
// Project Truffle - Local-First Knowledge Compiler

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tracing::{info, warn, error};

mod commands;
mod database;
mod compilation;
mod sync;
mod crypto;
mod models;
mod schema;
mod knowledge_graph;
mod validation;

use commands::*;
use knowledge_graph::*;
use validation::*;

/// Application state shared across commands
pub struct AppState {
    pub db: Arc<database::Database>,
    pub compiler: Arc<compilation::Compiler>,
    pub sync_manager: Arc<sync::SyncManager>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter("info,truffle_desktop=debug")
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!("Starting Truffle Desktop v{}", env!("CARGO_PKG_VERSION"));

    // Initialize database
    let db = Arc::new(database::Database::new().await?);
    info!("Database initialized");

    // Initialize compiler
    let compiler = Arc::new(compilation::Compiler::new(db.clone()).await?);
    info!("Compiler initialized");

    // Initialize sync manager
    let sync_manager = Arc::new(sync::SyncManager::new(db.clone()).await?);
    info!("Sync manager initialized");

    let app_state = AppState {
        db,
        compiler,
        sync_manager,
    };

    // Build system tray
    let tray_menu = SystemTrayMenu::new()
        .add_item(SystemTrayMenuItem::new("Show Truffle", "show"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(SystemTrayMenuItem::new("Compile All Pending", "compile_all"))
        .add_item(SystemTrayMenuItem::new("Sync Now", "sync_now"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(SystemTrayMenuItem::new("Quit", "quit"));

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            info!("Tauri application setup complete");
            
            // Set up window event handlers
            let window = app.get_window("main").unwrap();
            
            // Platform-specific window configuration
            #[cfg(target_os = "macos")]
            {
                use tauri::TitleBarStyle;
                window.set_title_bar_style(TitleBarStyle::Transparent)?;
            }
            
            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            match event {
                SystemTrayEvent::LeftClick { .. } => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "show" => {
                            let window = app.get_window("main").unwrap();
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                        "compile_all" => {
                            let state = app.state::<AppState>();
                            tokio::spawn(async move {
                                if let Err(e) = state.compiler.compile_all_pending().await {
                                    error!("Failed to compile all: {}", e);
                                }
                            });
                        }
                        "sync_now" => {
                            let state = app.state::<AppState>();
                            tokio::spawn(async move {
                                if let Err(e) = state.sync_manager.trigger_sync().await {
                                    error!("Failed to sync: {}", e);
                                }
                            });
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Raw artifact commands
            get_raw_artifacts,
            get_raw_artifact,
            import_raw_artifact,
            delete_raw_artifact,
            get_thumbnail,
            
            // Wiki commands
            get_wiki_nodes,
            get_wiki_node,
            create_wiki_node,
            update_wiki_node,
            delete_wiki_node,
            search_wiki_nodes,
            get_backlinks,
            
            // Compilation commands
            compile_artifact,
            compile_all_pending,
            get_compilation_status,
            get_compilation_logs,
            cancel_compilation,
            
            // Search commands
            search_content,
            semantic_search,
            
            // Sync commands
            get_sync_status,
            trigger_sync,
            pair_device,
            verify_pairing,
            
            // Schema commands
            get_schema,
            update_schema,
            validate_schema,
            
            // System commands
            get_storage_info,
            export_to_markdown,
            get_app_info,
            
            // Knowledge graph commands
            get_entities,
            get_entity,
            search_entities,
            create_entity,
            update_entity,
            delete_entity,
            get_relationships,
            create_relationship,
            get_entity_graph,
            find_path,
            
            // Validation commands
            validate_entity,
            get_contradictions,
            get_confidence_score,
            find_duplicates,
            merge_entities,
            resolve_contradiction,
            
            // Meeting commands
            get_meetings,
            extract_from_meeting,
            get_decisions,
            get_action_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
