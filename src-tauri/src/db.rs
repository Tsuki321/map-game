use crate::game::GameState;
use rusqlite::Connection;

pub fn initialize_db(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS saves (
            slot TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL,
            scenario TEXT NOT NULL
        )",
        [],
    ).map_err(|e| format!("DB init error: {}", e))?;
    Ok(())
}

pub fn save_game_state(conn: &Connection, slot: &str, state: &GameState) -> Result<(), String> {
    let data = serde_json::to_string(state)
        .map_err(|e| format!("Serialize error: {}", e))?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT OR REPLACE INTO saves (slot, data, created_at, scenario) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![slot, data, now, state.scenario_name],
    ).map_err(|e| format!("Save error: {}", e))?;
    Ok(())
}

pub fn load_game_state(conn: &Connection, slot: &str) -> Result<GameState, String> {
    let data: String = conn.query_row(
        "SELECT data FROM saves WHERE slot = ?1",
        rusqlite::params![slot],
        |row| row.get(0),
    ).map_err(|e| format!("Load error: {}", e))?;

    serde_json::from_str(&data)
        .map_err(|e| format!("Deserialize error: {}", e))
}

pub fn list_saves(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn.prepare("SELECT slot, scenario, created_at FROM saves ORDER BY created_at DESC")
        .map_err(|e| format!("Query error: {}", e))?;

    let saves = stmt.query_map([], |row| {
        let slot: String = row.get(0)?;
        let scenario: String = row.get(1)?;
        let created: String = row.get(2)?;
        Ok(format!("{} | {} | {}", slot, scenario, created))
    }).map_err(|e| format!("Query error: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(saves)
}
