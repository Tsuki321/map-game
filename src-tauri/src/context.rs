use crate::game::GameState;

pub fn build_system_prompt(state: &GameState) -> String {
    let player_name = state
        .get_player_country()
        .map(|c| c.name.as_str())
        .unwrap_or("Unknown");

    format!(
        r#"You are the simulation engine for an alternate history grand strategy game.

Current Scenario: {}
Date: {}/{}
Turn: {}
Player Country: {}

You control the world's response to the player's decisions. You must simulate realistic geopolitical, economic, and military outcomes. Use the provided tools to interact with the game state — query information, declare wars, negotiate, pass policies, and advance turns.

IMPORTANT RULES:
1. Always use tools to interact with the game state, do not make up outcomes
2. When the player issues a directive, determine what tools to call to execute it
3. After executing tools, provide a narrative summary of what happened
4. Be realistic — consider military strength, economy, geography, and diplomacy
5. AI-controlled countries act in their own strategic interest
6. Call advance_turn when the player wants to see results of their actions
7. Territory changes happen when war progress reaches 100% on a front
8. Always respond with the narrative of events, not just raw tool output"#,
        state.scenario_name,
        state.current_month,
        state.current_year,
        state.turn_number,
        player_name
    )
}

pub fn build_turn_context(state: &GameState) -> String {
    let mut ctx = String::new();

    // Player country status
    if let Some(country) = state.get_player_country() {
        ctx.push_str(&format!("=== YOUR COUNTRY: {} ===\n", country.name));
        ctx.push_str(&format!("Capital: {}, Population: {}k\n", country.capital, country.population));
        ctx.push_str(&format!("GDP: ${:.2}B, Growth: {:.1}%\n", country.gdp, country.gdp_growth));
        ctx.push_str(&format!("Government: {}, Ideology: {}\n", country.government_type, country.ideology));
        ctx.push_str(&format!("Stability: {:.0}%\n", country.stability));
        ctx.push_str(&format!("Tech Level: {:?}\n", country.tech_level));
        ctx.push_str(&format!("Resources: {}\n", country.resources.join(", ")));

        ctx.push_str("\n-- Military --\n");
        let mp = &country.military_power;
        ctx.push_str(&format!("Army: {:.0}, Navy: {:.0}, Air Force: {:.0}\n", mp.army_strength, mp.navy_strength, mp.air_force_strength));
        ctx.push_str(&format!("Mobilization: {:.0}%, Morale: {:.0}%\n", mp.mobilization_pct, mp.morale));
        ctx.push_str(&format!("Manpower Reserves: {}k\n", mp.manpower_reserves));

        ctx.push_str(&format!("\nNeighbors: {}\n", country.neighbors.join(", ")));
        ctx.push_str(&format!("Alliances: {}\n", if country.alliances.is_empty() { "None".to_string() } else { country.alliances.join(", ") }));

        if !country.active_wars.is_empty() {
            ctx.push_str(&format!("\nActive Wars: {}\n", country.active_wars.join(", ")));
        }

        if !country.controlled_territory.is_empty() {
            ctx.push_str(&format!("Controlled Territory: {}\n", country.controlled_territory.join(", ")));
        }
    }

    // Active wars
    let active_wars: Vec<_> = state.wars.values().filter(|w| w.status == crate::game::WarStatus::Ongoing).collect();
    if !active_wars.is_empty() {
        ctx.push_str("\n=== ACTIVE WARS ===\n");
        for war in active_wars {
            let a_names: Vec<_> = war.attackers.iter().filter_map(|id| state.countries.get(id).map(|c| c.name.as_str())).collect();
            let d_names: Vec<_> = war.defenders.iter().filter_map(|id| state.countries.get(id).map(|c| c.name.as_str())).collect();
            ctx.push_str(&format!("- {}: {} vs {} (Score: {:.1}, Status: {:?})\n", 
                war.name, a_names.join("/"), d_names.join("/"), war.war_score, war.status));
            for front in &war.fronts {
                if front.active {
                    ctx.push_str(&format!("  Front '{}': {:?} terrain, {:.0}% attacker progress\n", 
                        front.name, front.terrain, front.attacker_progress));
                }
            }
        }
    }

    // Recent history (last 3 turns)
    let history_start = if state.turn_history.len() > 3 { state.turn_history.len() - 3 } else { 0 };
    if history_start < state.turn_history.len() {
        ctx.push_str("\n=== RECENT TURN HISTORY ===\n");
        for summary in &state.turn_history[history_start..] {
            ctx.push_str(&format!("Turn {} ({}): {} | Results: {}\n", 
                summary.turn, summary.player_action, 
                summary.results.join("; "),
                if summary.new_events.is_empty() { "None".to_string() } else { summary.new_events.join("; ") }
            ));
        }
    }

    ctx
}

pub fn compress_context(state: &GameState, _max_tokens: usize) -> String {
    // Simplified compression: just returns a compact summary
    let mut summary = String::new();

    summary.push_str(&format!("Game: {}, Turn: {}, Date: {}/{}\n", 
        state.scenario_name, state.turn_number, state.current_month, state.current_year));

    if let Some(pc) = state.get_player_country() {
        summary.push_str(&format!("Player: {} (GDP: {:.1}B, Army: {:.0}, Stability: {:.0}%)\n",
            pc.name, pc.gdp, pc.military_power.army_strength, pc.stability));
    }

    let active_war_count = state.wars.values().filter(|w| w.status == crate::game::WarStatus::Ongoing).count();
    summary.push_str(&format!("Active wars: {}, Countries: {}\n", active_war_count, state.countries.len()));

    summary
}
