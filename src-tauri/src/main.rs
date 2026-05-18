mod advisor;
mod context;
mod db;
mod game;
mod llm;
mod tools;

use std::sync::Arc;

use game::TurnSummary;
use llm::provider::{ChatMessage, LlmConfig, LlmProvider};
use llm::OllamaProvider;
use llm::OpenAiProvider;
use rusqlite::Connection;
use tauri::Manager;
use tokio::sync::Mutex;

pub struct AppState {
    pub game_state: Arc<Mutex<game::GameState>>,
    pub llm_config: Arc<Mutex<Option<LlmConfig>>>,
    pub llm_provider: Arc<Mutex<Option<Box<dyn LlmProvider>>>>,
    pub db: Arc<Mutex<Connection>>,
    pub advisor_enabled: Arc<Mutex<bool>>,
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
async fn get_game_state(state: tauri::State<'_, AppState>) -> Result<game::GameState, String> {
    let game_state = state
        .game_state
        .lock()
        .await;
    Ok(game_state.clone())
}

#[tauri::command]
async fn select_country(
    state: tauri::State<'_, AppState>,
    country_id: String,
) -> Result<game::Country, String> {
    let mut game_state = state
        .game_state
        .lock()
        .await;

    game_state.player_country_id = Some(country_id.clone());

    let country = game_state
        .countries
        .get_mut(&country_id)
        .ok_or_else(|| format!("Country with id '{}' not found", country_id))?;

    country.is_player_controlled = true;

    Ok(country.clone())
}

#[tauri::command]
async fn get_country_info(
    state: tauri::State<'_, AppState>,
    country_id: String,
) -> Result<game::Country, String> {
    let game_state = state
        .game_state
        .lock()
        .await;

    game_state
        .countries
        .get(&country_id)
        .cloned()
        .ok_or_else(|| format!("Country with id '{}' not found", country_id))
}

#[tauri::command]
async fn get_world_situation(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let game_state = state
        .game_state
        .lock()
        .await;

    let mut summary = String::new();

    // Active wars
    summary.push_str("=== ACTIVE WARS ===\n");
    if game_state.wars.is_empty() {
        summary.push_str("No active wars.\n");
    } else {
        for (_, war) in &game_state.wars {
            summary.push_str(&format!(
                "- {}: {:?} (since turn {})\n",
                war.name,
                war.status,
                war.start_turn
            ));
        }
    }

    // Recent events
    summary.push_str("\n=== RECENT EVENTS ===\n");
    if game_state.global_events.is_empty() {
        summary.push_str("No recent events.\n");
    } else {
        let start = if game_state.global_events.len() > 5 {
            game_state.global_events.len() - 5
        } else {
            0
        };
        for event in &game_state.global_events[start..] {
            summary.push_str(&format!("- {}\n", event.description));
        }
    }

    // Player country overview
    summary.push_str("\n=== PLAYER COUNTRY ===\n");
    let player_country = game_state
        .player_country_id
        .as_ref()
        .and_then(|id| game_state.countries.get(id));

    if let Some(country) = player_country {
        summary.push_str(&format!(
            "{} | GDP: ${:.2}B | Stability: {:.0}% | Army: {:.0} | Navy: {:.0} | Air: {:.0}\n",
            country.name,
            country.gdp,
            country.stability,
            country.military_power.army_strength,
            country.military_power.navy_strength,
            country.military_power.air_force_strength,
        ));
    } else {
        summary.push_str("No country selected.\n");
    }

    Ok(summary)
}

#[tauri::command]
async fn configure_llm(
    state: tauri::State<'_, AppState>,
    provider: String,
    api_key: Option<String>,
    model: String,
    endpoint: String,
) -> Result<(), String> {
    let config = LlmConfig {
        provider_type: provider.clone(),
        api_key: api_key.clone(),
        model: model.clone(),
        endpoint: if endpoint.is_empty() {
            None
        } else {
            Some(endpoint.clone())
        },
    };

    // Store config
    {
        let mut config_guard = state
            .llm_config
            .lock()
            .await;
        *config_guard = Some(config.clone());
    }

    // Instantiate the right provider
    let provider_box: Box<dyn LlmProvider> = match provider.to_lowercase().as_str() {
        "openai" | "open_ai" => {
            let key = api_key
                .clone()
                .ok_or_else(|| "OpenAI provider requires an API key".to_string())?;
            let ep = if endpoint.is_empty() { None } else { Some(endpoint.clone()) };
            Box::new(OpenAiProvider::new(key, model, ep))
        }
        "ollama" => {
            let ep = if endpoint.is_empty() { None } else { Some(endpoint.clone()) };
            Box::new(OllamaProvider::new(model, ep))
        }
        other => {
            return Err(format!(
                "Unknown provider '{}'. Supported: openai, ollama",
                other
            ));
        }
    };

    {
        let mut provider_guard = state
            .llm_provider
            .lock()
            .await;
        *provider_guard = Some(provider_box);
    }

    Ok(())
}

#[tauri::command]
fn get_scenario_list(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to resolve resource directory: {}", e))?;

    let scenarios_dir = resource_dir.join("resources").join("scenarios");

    if !scenarios_dir.exists() {
        // Fallback: check current working directory
        let cwd_scenarios = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .join("resources")
            .join("scenarios");

        if cwd_scenarios.exists() {
            return read_scenario_files(&cwd_scenarios);
        }

        return Ok(Vec::new());
    }

    read_scenario_files(&scenarios_dir)
}

fn read_scenario_files(dir: &std::path::Path) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read scenarios directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        if path.extension().map(|ext| ext == "json").unwrap_or(false) {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }

    names.sort();
    Ok(names)
}

#[tauri::command]
async fn start_new_game(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    scenario: String,
) -> Result<game::GameState, String> {
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to resolve resource directory: {}", e))?;

    let scenario_path = resource_dir
        .join("resources")
        .join("scenarios")
        .join(format!("{}.json", scenario));

    // Fallback to cwd if not in resource dir
    let scenario_path = if scenario_path.exists() {
        scenario_path
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .join("resources")
            .join("scenarios")
            .join(format!("{}.json", scenario))
    };

    let json_data = std::fs::read_to_string(&scenario_path)
        .map_err(|e| format!("Failed to read scenario file '{}': {}", scenario_path.display(), e))?;

    let game_state: game::GameState = serde_json::from_str(&json_data)
        .map_err(|e| format!("Failed to deserialize scenario '{}': {}", scenario, e))?;

    let cloned = game_state.clone();

    {
        let mut state_guard = state
            .game_state
            .lock()
            .await;
        *state_guard = game_state;
    }

    Ok(cloned)
}

#[tauri::command]
async fn save_game(
    state: tauri::State<'_, AppState>,
    slot: String,
) -> Result<(), String> {
    let game_state = state
        .game_state
        .lock()
        .await;

    let db = state
        .db
        .lock()
        .await;

    db::save_game_state(&db, &slot, &game_state)
        .map_err(|e| format!("Failed to save game: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn load_game(
    state: tauri::State<'_, AppState>,
    slot: String,
) -> Result<game::GameState, String> {
    let db = state
        .db
        .lock()
        .await;

    let loaded = db::load_game_state(&db, &slot)
        .map_err(|e| format!("Failed to load save '{}': {}", slot, e))?;
    drop(db);

    let mut state_guard = state
        .game_state
        .lock()
        .await;
    *state_guard = loaded.clone();

    Ok(loaded)
}

#[tauri::command]
async fn list_saves(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = state
        .db
        .lock()
        .await;

    let saves = db::list_saves(&db)
        .map_err(|e| format!("Failed to list saves: {}", e))?;

    Ok(saves)
}

#[tauri::command]
async fn submit_directive(
    state: tauri::State<'_, AppState>,
    directive: String,
    use_advisor: bool,
) -> Result<String, String> {
    let mut final_directive = directive.clone();

    // Optionally run through advisor
    if use_advisor {
        let game_state = state.game_state.lock().await;
        let config_guard = state.llm_config.lock().await;
        let provider_guard = state.llm_provider.lock().await;

        if let (Some(config), Some(provider)) = (config_guard.as_ref(), provider_guard.as_ref()) {
            let suggestion = advisor::get_advisor_suggestion(
                &game_state, &final_directive, config, provider.as_ref(),
            ).await?;
            final_directive = suggestion;
        }
        drop(provider_guard);
        drop(config_guard);
        drop(game_state);
    }

    let game_state_arc = state.game_state.clone();
    let provider_arc = state.llm_provider.clone();
    let config_arc = state.llm_config.clone();

    // Build context and run LLM
    let result = execute_llm_turn(
        &game_state_arc,
        &provider_arc,
        &config_arc,
        &final_directive,
    ).await?;

    Ok(result)
}

#[tauri::command]
async fn get_advisor_suggestion(
    state: tauri::State<'_, AppState>,
    directive: String,
) -> Result<String, String> {
    let game_state = state.game_state.lock().await;
    let config_guard = state.llm_config.lock().await;
    let provider_guard = state.llm_provider.lock().await;

    let (config, provider) = match (config_guard.as_ref(), provider_guard.as_ref()) {
        (Some(c), Some(p)) => (c, p),
        _ => return Err("LLM not configured. Please configure an LLM provider first.".to_string()),
    };

    advisor::get_advisor_suggestion(&game_state, &directive, config, provider.as_ref()).await
}

#[tauri::command]
async fn execute_turn(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let mut game_state = state.game_state.lock().await;

    // Process economy
    let economic_changes = game_state.process_economy();

    // Process wars
    let war_updates = game_state.process_war_resolution();

    // Advance time
    game_state.advance_time();

    // Build summary
    let mut summary = format!(
        "=== Turn {} Complete ===\nDate: {}/{}\n\n",
        game_state.turn_number,
        game_state.current_month,
        game_state.current_year
    );

    summary.push_str("-- Economic Changes --\n");
    for change in &economic_changes {
        summary.push_str(&format!("  {}\n", change.description));
    }

    summary.push_str("\n-- War Updates --\n");
    if war_updates.is_empty() {
        summary.push_str("  No changes on active fronts.\n");
    } else {
        for update in &war_updates {
            summary.push_str(&format!("  {}\n", update.description));
        }
    }

    // Add to history
    let turn = game_state.turn_number;
    let year = game_state.current_year;
    let month = game_state.current_month;
    game_state.turn_history.push(TurnSummary {
        turn,
        year,
        month,
        player_action: "Turn advanced".to_string(),
        results: economic_changes.iter().map(|e| e.description.clone()).collect(),
        territory_changes: Vec::new(),
        war_updates,
        economic_changes,
        new_events: Vec::new(),
    });

    Ok(summary)
}

// Helper: run a directive through the LLM with tool execution
async fn execute_llm_turn(
    game_state_arc: &Arc<Mutex<game::GameState>>,
    provider_arc: &Arc<Mutex<Option<Box<dyn LlmProvider>>>>,
    config_arc: &Arc<Mutex<Option<LlmConfig>>>,
    directive: &str,
) -> Result<String, String> {
    use tools::definitions::get_all_tools;

    let (system_prompt, turn_context, all_tools) = {
        let game_state = game_state_arc.lock().await;

        let sp = context::build_system_prompt(&game_state);
        let tc = context::build_turn_context(&game_state);
        let tools = get_all_tools();
        (sp, tc, tools)
    };

    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        ChatMessage {
            role: "system".to_string(),
            content: turn_context,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        ChatMessage {
            role: "user".to_string(),
            content: directive.to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ];

    let mut narrative_response = String::new();

    // First LLM call — lock provider only for the call
    let response = {
        let provider_guard = provider_arc.lock().await;
        let provider = provider_guard.as_ref()
            .ok_or_else(|| "LLM not configured".to_string())?;
        provider.chat(messages.clone(), Some(all_tools.clone())).await?
    };

    if let Some(ref usage) = response.usage {
        eprintln!(
            "LLM call: {} prompt + {} completion = {} total tokens",
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens
        );
    }

    // Check for tool calls
    if let Some(ref tool_calls) = response.message.tool_calls {
        if !tool_calls.is_empty() {
            // Add assistant message
            messages.push(response.message.clone());

            // Execute each tool
            let mut game_state = game_state_arc.lock().await;
            let mut executor = tools::executors::ToolExecutor::new(&mut *game_state);

            for tc in tool_calls {
                let result = executor.execute(&tc.function.name, &tc.function.arguments);
                messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: result,
                    tool_calls: None,
                    tool_call_id: Some(tc.id.clone()),
                    name: Some(tc.function.name.clone()),
                });
            }
            drop(game_state);

            // Second LLM call to get narrative after tool execution
            let response2 = {
                let provider_guard = provider_arc.lock().await;
                let provider = provider_guard.as_ref()
                    .ok_or_else(|| "LLM not configured".to_string())?;
                provider.chat(messages.clone(), Some(all_tools)).await?
            };
            narrative_response = response2.message.content.clone();

            // Handle any further tool calls (recursive, up to 3 rounds)
            if let Some(ref tc2) = response2.message.tool_calls {
                if !tc2.is_empty() {
                    messages.push(response2.message.clone());
                    let mut game_state = game_state_arc.lock().await;
                    let mut executor = tools::executors::ToolExecutor::new(&mut *game_state);
                    for tc in tc2 {
                        let result = executor.execute(&tc.function.name, &tc.function.arguments);
                        messages.push(ChatMessage {
                            role: "tool".to_string(),
                            content: result,
                            tool_calls: None,
                            tool_call_id: Some(tc.id.clone()),
                            name: Some(tc.function.name.clone()),
                        });
                    }
                    drop(game_state);

                    let response3 = {
                        let provider_guard = provider_arc.lock().await;
                        let provider = provider_guard.as_ref()
                            .ok_or_else(|| "LLM not configured".to_string())?;
                        provider.chat(messages, None).await?
                    };
                    if !response3.message.content.is_empty() {
                        narrative_response.push_str("\n\n");
                        narrative_response.push_str(&response3.message.content);
                    }
                }
            }
        } else {
            narrative_response = response.message.content.clone();
        }
    } else {
        narrative_response = response.message.content.clone();
    }

    Ok(narrative_response)
}

// ---------------------------------------------------------------------------
// Application entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize SQLite database
    let db_path = dirs_next().unwrap_or_else(|_| {
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push("map_game.db");
        path
    });

    let conn = Connection::open(&db_path)
        .expect("Failed to open database");

    db::initialize_db(&conn)
        .expect("Failed to initialize database");

    let app_state = AppState {
        game_state: Arc::new(Mutex::new(game::GameState::new("default"))),
        llm_config: Arc::new(Mutex::new(None)),
        llm_provider: Arc::new(Mutex::new(None)),
        db: Arc::new(Mutex::new(conn)),
        advisor_enabled: Arc::new(Mutex::new(false)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_game_state,
            select_country,
            get_country_info,
            get_world_situation,
            configure_llm,
            get_scenario_list,
            start_new_game,
            save_game,
            load_game,
            list_saves,
            submit_directive,
            get_advisor_suggestion,
            execute_turn,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn dirs_next() -> Result<std::path::PathBuf, String> {
    // Simple platform-appropriate data directory
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Failed to get APPDATA".to_string())?;
        let dir = std::path::PathBuf::from(appdata).join("map-game");
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
        Ok(dir.join("map_game.db"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "Failed to get HOME".to_string())?;
        let dir = std::path::PathBuf::from(home).join(".local").join("share").join("map-game");
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
        Ok(dir.join("map_game.db"))
    }
}

fn main() {
    run();
}
