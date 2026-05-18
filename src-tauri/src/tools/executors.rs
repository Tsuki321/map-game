use crate::game::{
    self, Battle, BattleOutcome, Country, DiplomaticRelation, DiplomaticStance, Front,
    GameState, GlobalEvent, MilitaryPower, TechLevel, TerrainType, TradeRelation, TradeStatus,
    Treaty, War, WarStatus, WarType,
};
use rand::Rng;
use serde_json::{json, Value};
use uuid::Uuid;

/// Executes game-state tools by reading from / mutating a `GameState` reference.
/// Uses `&'a mut GameState` so it composes seamlessly with the borrow from `main.rs`.
pub struct ToolExecutor<'a> {
    state: &'a mut GameState,
}

impl<'a> ToolExecutor<'a> {
    pub fn new(state: &'a mut GameState) -> Self {
        Self { state }
    }

    /// Dispatch a tool by name with its JSON arguments string.
    /// Returns a JSON string — either the tool's result or an error envelope.
    pub fn execute(&mut self, tool_name: &str, arguments_json: &str) -> String {
        let args: Value =
            serde_json::from_str(arguments_json).unwrap_or(json!({}));

        let result: Result<String, String> = match tool_name {
            "get_country_status" => self.get_country_status(&args),
            "get_player_status" => self.get_player_status(),
            "get_neighbors" => self.get_neighbors(&args),
            "get_world_situation" => self.get_world_situation(),
            "declare_war" => self.declare_war(&args),
            "mobilize_forces" => self.mobilize_forces(&args),
            "sue_for_peace" => self.sue_for_peace(&args),
            "negotiate_trade" => self.negotiate_trade(&args),
            "form_alliance" => self.form_alliance(&args),
            "pass_policy" => self.pass_policy(&args),
            "set_budget" => self.set_budget(&args),
            "advance_turn" => self.advance_turn(),
            _ => Err(format!("Unknown tool: {}", tool_name)),
        };

        match result {
            Ok(s) => s,
            Err(e) => json!({"error": e}).to_string(),
        }
    }

    // -----------------------------------------------------------------------
    // Tool handlers
    // -----------------------------------------------------------------------

    fn get_country_status(&self, args: &Value) -> Result<String, String> {
        let country_id = args["country_id"]
            .as_str()
            .ok_or("Missing 'country_id'")?;

        let country = self
            .state
            .countries
            .get(country_id)
            .ok_or_else(|| format!("Country not found: {}", country_id))?;

        serde_json::to_string_pretty(country)
            .map_err(|e| format!("Serialize error: {}", e))
    }

    fn get_player_status(&self) -> Result<String, String> {
        let pid = self
            .state
            .player_country_id
            .as_ref()
            .ok_or("No player country selected")?;

        let country = self
            .state
            .countries
            .get(pid)
            .ok_or("Player country not found in state")?;

        serde_json::to_string_pretty(country)
            .map_err(|e| format!("Serialize error: {}", e))
    }

    fn get_neighbors(&self, args: &Value) -> Result<String, String> {
        let country_id = args["country_id"]
            .as_str()
            .ok_or("Missing 'country_id'")?;

        let country = self
            .state
            .countries
            .get(country_id)
            .ok_or_else(|| format!("Country not found: {}", country_id))?;

        let mut neighbors_info: Vec<Value> = Vec::new();

        for nid in &country.neighbors {
            let stance = self.state.get_diplomatic_stance(country_id, nid);
            let name = self
                .state
                .countries
                .get(nid)
                .map(|c| c.name.as_str())
                .unwrap_or(nid.as_str());

            neighbors_info.push(json!({
                "country_id": nid,
                "name": name,
                "diplomatic_stance": serde_json::to_value(&stance).unwrap_or(json!("Neutral")),
            }));
        }

        serde_json::to_string_pretty(&neighbors_info)
            .map_err(|e| format!("Serialize error: {}", e))
    }

    fn get_world_situation(&self) -> Result<String, String> {
        let state = &self.state;

        // Active wars
        let wars_summary: Vec<Value> = state
            .wars
            .values()
            .filter(|w| w.status == WarStatus::Ongoing)
            .map(|w| {
                let a_names: Vec<&str> = w
                    .attackers
                    .iter()
                    .filter_map(|id| state.countries.get(id).map(|c| c.name.as_str()))
                    .collect();
                let d_names: Vec<&str> = w
                    .defenders
                    .iter()
                    .filter_map(|id| state.countries.get(id).map(|c| c.name.as_str()))
                    .collect();

                json!({
                    "war_id": w.id,
                    "name": w.name,
                    "attackers": a_names,
                    "defenders": d_names,
                    "justification": w.justification,
                    "war_score": w.war_score,
                    "war_type": serde_json::to_value(&w.war_type).unwrap_or(json!(null)),
                    "status": serde_json::to_value(&w.status).unwrap_or(json!(null)),
                })
            })
            .collect();

        // Last 5 global events
        let events_summary: Vec<Value> = {
            let start = if state.global_events.len() > 5 {
                state.global_events.len() - 5
            } else {
                0
            };
            state.global_events[start..]
                .iter()
                .map(|e| {
                    json!({
                        "turn": e.turn,
                        "year": e.year,
                        "month": e.month,
                        "type": e.event_type,
                        "description": e.description,
                    })
                })
                .collect()
        };

        // Player country summary
        let player_summary = state.get_player_country().map(|c| {
            json!({
                "name": c.name,
                "capital": c.capital,
                "gdp": c.gdp,
                "population": c.population,
                "stability": c.stability,
                "government_type": c.government_type,
                "ideology": c.ideology,
                "military_power": {
                    "army_strength": c.military_power.army_strength,
                    "navy_strength": c.military_power.navy_strength,
                    "air_force_strength": c.military_power.air_force_strength,
                    "mobilization_pct": c.military_power.mobilization_pct,
                    "morale": c.military_power.morale,
                },
            })
        });

        let summary = json!({
            "turn": state.turn_number,
            "month": state.current_month,
            "year": state.current_year,
            "scenario": state.scenario_name,
            "active_wars": wars_summary,
            "recent_events": events_summary,
            "player_country": player_summary,
            "total_countries": state.countries.len(),
        });

        serde_json::to_string_pretty(&summary)
            .map_err(|e| format!("Serialize error: {}", e))
    }

    fn declare_war(&mut self, args: &Value) -> Result<String, String> {
        let target_id = args["target_country"]
            .as_str()
            .ok_or("Missing 'target_country'")?;
        let justification = args["justification"]
            .as_str()
            .ok_or("Missing 'justification'")?;
        let allocation_pct = args["military_allocation_pct"]
            .as_f64()
            .ok_or("Missing 'military_allocation_pct'")?;

        let attacker_id = self
            .state
            .player_country_id
            .clone()
            .ok_or("No player country selected")?;

        if attacker_id == target_id {
            return Err("Cannot declare war on yourself".to_string());
        }
        if !self.state.countries.contains_key(target_id) {
            return Err(format!("Target country not found: {}", target_id));
        }

        // Check not already at war
        let enemies = self.state.find_countries_at_war_with(&attacker_id);
        if enemies.contains(&target_id.to_string()) {
            return Err("Already at war with this country".to_string());
        }

        // Update diplomatic stance
        self.state
            .set_diplomatic_stance(&attacker_id, target_id, DiplomaticStance::AtWar);

        // Create the War struct
        let war_id = Uuid::new_v4().to_string();
        let target_name = self
            .state
            .countries
            .get(target_id)
            .map(|c| c.name.as_str())
            .unwrap_or(target_id);
        let attacker_name = self
            .state
            .countries
            .get(&attacker_id)
            .map(|c| c.name.as_str())
            .unwrap_or(attacker_id.as_str());

        let war = War {
            id: war_id.clone(),
            name: format!("{} vs {}", attacker_name, target_name),
            attackers: vec![attacker_id.clone()],
            defenders: vec![target_id.to_string()],
            start_turn: self.state.turn_number,
            start_year: self.state.current_year,
            start_month: self.state.current_month,
            end_turn: None,
            status: WarStatus::Ongoing,
            fronts: Vec::new(),
            war_type: WarType::TotalWar,
            justification: justification.to_string(),
            war_score: 0.0,
            battles: Vec::new(),
        };

        self.state.wars.insert(war_id.clone(), war);

        // Add war to both countries' active_wars
        if let Some(attacker) = self.state.countries.get_mut(&attacker_id) {
            attacker.active_wars.push(war_id.clone());
            attacker.military_power.mobilization_pct =
                attacker.military_power.mobilization_pct.max(allocation_pct);
        }
        if let Some(defender) = self.state.countries.get_mut(target_id) {
            defender.active_wars.push(war_id.clone());
        }

        // Record global event
        self.state.global_events.push(GlobalEvent {
            turn: self.state.turn_number,
            year: self.state.current_year,
            month: self.state.current_month,
            event_type: "war_declared".to_string(),
            description: format!(
                "{} declared war on {}: {}",
                attacker_name, target_name, justification
            ),
            involved_countries: vec![attacker_id.clone(), target_id.to_string()],
        });

        let result = json!({
            "status": "success",
            "message": format!("War declared on {}", target_name),
            "war_id": war_id,
            "allocation_pct": allocation_pct,
        });

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }

    fn mobilize_forces(&mut self, args: &Value) -> Result<String, String> {
        let target_border = args["target_border"]
            .as_str()
            .ok_or("Missing 'target_border'")?;
        let force_size_pct = args["force_size_pct"]
            .as_f64()
            .ok_or("Missing 'force_size_pct'")?;
        let focus = args["focus"].as_str().ok_or("Missing 'focus'")?;

        let valid_foci = ["land", "air", "naval"];
        if !valid_foci.contains(&focus) {
            return Err(format!(
                "Invalid focus '{}'. Must be one of: land, air, naval",
                focus
            ));
        }

        let pid = self
            .state
            .player_country_id
            .as_ref()
            .ok_or("No player country selected")?
            .clone();

        let player_country = self
            .state
            .countries
            .get_mut(&pid)
            .ok_or("Player country not found")?;

        let border_name = self
            .state
            .countries
            .get(target_border)
            .map(|c| c.name.as_str())
            .unwrap_or(target_border);

        player_country.military_power.mobilization_pct = force_size_pct;

        self.state.global_events.push(GlobalEvent {
            turn: self.state.turn_number,
            year: self.state.current_year,
            month: self.state.current_month,
            event_type: "mobilization".to_string(),
            description: format!(
                "{} mobilized {}% of forces toward the {} border (focus: {})",
                player_country.name, force_size_pct, border_name, focus
            ),
            involved_countries: vec![pid, target_border.to_string()],
        });

        let result = json!({
            "status": "success",
            "message": format!(
                "Mobilized {}% of forces toward {} border (focus: {})",
                force_size_pct, border_name, focus
            ),
            "mobilization_pct": force_size_pct,
        });

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }

    fn sue_for_peace(&mut self, args: &Value) -> Result<String, String> {
        let target_id = args["target_country"]
            .as_str()
            .ok_or("Missing 'target_country'")?;
        let terms = args["terms"].as_str().ok_or("Missing 'terms'")?;

        let pid = self
            .state
            .player_country_id
            .as_ref()
            .ok_or("No player country selected")?
            .clone();

        if pid == target_id {
            return Err("Cannot negotiate peace with yourself".to_string());
        }

        // Find the active war between player and target
        let war_entry: Option<(String, f64)> = {
            let mut found = None;
            for (war_id, war) in &self.state.wars {
                if war.status != WarStatus::Ongoing {
                    continue;
                }
                let involved = war.is_involved(&pid) && war.is_involved(target_id);
                if involved {
                    found = Some((war_id.clone(), war.war_score));
                    break;
                }
            }
            found
        };

        let (war_id, war_score) = match war_entry {
            Some(w) => w,
            None => {
                return Err(format!(
                    "No active war found between you and {}",
                    target_id
                ))
            }
        };

        // Determine acceptance chance based on war score
        let player_is_attacker = self
            .state
            .wars
            .get(&war_id)
            .map(|w| w.attackers.contains(&pid))
            .unwrap_or(false);

        let mut rng = rand::thread_rng();
        // Score relative to player: positive if player is winning
        let relative_score = if player_is_attacker {
            war_score
        } else {
            -war_score
        };
        let acceptance_chance = ((relative_score + 100.0) / 2.0).clamp(10.0, 90.0) / 100.0;
        let accepted = rng.gen::<f64>() < acceptance_chance;

        let target_name = self
            .state
            .countries
            .get(target_id)
            .map(|c| c.name.as_str())
            .unwrap_or(target_id);

        let player_name = self
            .state
            .countries
            .get(&pid)
            .map(|c| c.name.as_str())
            .unwrap_or("You");

        if accepted {
            // End the war
            if let Some(war) = self.state.wars.get_mut(&war_id) {
                war.status = WarStatus::Ceasefire;
                war.end_turn = Some(self.state.turn_number);
            }

            // Update diplomatic stance
            self.state
                .set_diplomatic_stance(&pid, target_id, DiplomaticStance::Wary);

            // Remove from active_wars
            if let Some(c) = self.state.countries.get_mut(&pid) {
                c.active_wars.retain(|w| w != &war_id);
            }
            if let Some(c) = self.state.countries.get_mut(target_id) {
                c.active_wars.retain(|w| w != &war_id);
            }

            self.state.global_events.push(GlobalEvent {
                turn: self.state.turn_number,
                year: self.state.current_year,
                month: self.state.current_month,
                event_type: "peace_accepted".to_string(),
                description: format!(
                    "{} accepted peace terms from {}: {}",
                    target_name, player_name, terms
                ),
                involved_countries: vec![pid, target_id.to_string()],
            });

            let result = json!({
                "status": "success",
                "message": format!("{} accepted your peace terms", target_name),
                "terms": terms,
                "war_id": war_id,
                "accepted": true,
            });
            Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
        } else {
            self.state.global_events.push(GlobalEvent {
                turn: self.state.turn_number,
                year: self.state.current_year,
                month: self.state.current_month,
                event_type: "peace_rejected".to_string(),
                description: format!(
                    "{} rejected peace overtures from {}",
                    target_name, player_name
                ),
                involved_countries: vec![pid, target_id.to_string()],
            });

            let result = json!({
                "status": "rejected",
                "message": format!("{} rejected your peace terms", target_name),
                "terms": terms,
                "war_id": war_id,
                "accepted": false,
            });
            Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
    }

    fn negotiate_trade(&mut self, args: &Value) -> Result<String, String> {
        let target_id = args["target_country"]
            .as_str()
            .ok_or("Missing 'target_country'")?;

        let offer_resources: Vec<String> = args["offer_resources"]
            .as_array()
            .ok_or("Missing 'offer_resources' array")?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let request_resources: Vec<String> = args["request_resources"]
            .as_array()
            .ok_or("Missing 'request_resources' array")?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let pid = self
            .state
            .player_country_id
            .as_ref()
            .ok_or("No player country selected")?
            .clone();

        if pid == target_id {
            return Err("Cannot trade with yourself".to_string());
        }
        if !self.state.countries.contains_key(target_id) {
            return Err(format!("Target country not found: {}", target_id));
        }

        let target_name = self
            .state
            .countries
            .get(target_id)
            .map(|c| c.name.as_str())
            .unwrap_or(target_id);

        // Add trade relation to player
        if let Some(country) = self.state.countries.get_mut(&pid) {
            let existing = country.trade_partners.iter().any(|t| t.partner_id == target_id);
            if existing {
                // Update existing
                country.trade_partners.retain(|t| t.partner_id != target_id);
            }
            country.trade_partners.push(TradeRelation {
                partner_id: target_id.to_string(),
                export_goods: offer_resources.clone(),
                import_goods: request_resources.clone(),
                trade_volume: 1.0,
                status: TradeStatus::Active,
            });
        }

        // Add reciprocal trade relation to target
        if let Some(country) = self.state.countries.get_mut(target_id) {
            let existing = country.trade_partners.iter().any(|t| t.partner_id == pid);
            if existing {
                country.trade_partners.retain(|t| t.partner_id != pid);
            }
            country.trade_partners.push(TradeRelation {
                partner_id: pid.clone(),
                export_goods: request_resources.clone(),
                import_goods: offer_resources.clone(),
                trade_volume: 1.0,
                status: TradeStatus::Active,
            });
        }

        // Bump diplomatic stance to Friendly (unless already Allied)
        let current_stance = self.state.get_diplomatic_stance(&pid, target_id);
        if current_stance != DiplomaticStance::Allied {
            self.state
                .set_diplomatic_stance(&pid, target_id, DiplomaticStance::Friendly);
        }

        // Add TradeAgreement treaty
        let has_trade = self
            .state
            .diplomatic_relations
            .iter_mut()
            .find(|r| {
                (r.country_a == pid && r.country_b == target_id)
                    || (r.country_a == target_id && r.country_b == pid)
            })
            .map(|r| {
                if !r.treaties.contains(&Treaty::TradeAgreement) {
                    r.treaties.push(Treaty::TradeAgreement);
                }
                true
            })
            .unwrap_or(false);

        if !has_trade {
            self.state
                .diplomatic_relations
                .push(DiplomaticRelation {
                    country_a: pid.clone(),
                    country_b: target_id.to_string(),
                    stance: DiplomaticStance::Friendly,
                    trust: 10.0,
                    treaties: vec![Treaty::TradeAgreement],
                });
        }

        self.state.global_events.push(GlobalEvent {
            turn: self.state.turn_number,
            year: self.state.current_year,
            month: self.state.current_month,
            event_type: "trade_deal".to_string(),
            description: format!(
                "Trade deal established with {}: offering [{}], requesting [{}]",
                target_name,
                offer_resources.join(", "),
                request_resources.join(", ")
            ),
            involved_countries: vec![pid, target_id.to_string()],
        });

        let result = json!({
            "status": "success",
            "message": format!("Trade deal established with {}", target_name),
            "offer": offer_resources,
            "request": request_resources,
        });

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }

    fn form_alliance(&mut self, args: &Value) -> Result<String, String> {
        let target_id = args["target_country"]
            .as_str()
            .ok_or("Missing 'target_country'")?;
        let alliance_type = args["alliance_type"]
            .as_str()
            .ok_or("Missing 'alliance_type'")?;

        let valid_types = ["defensive", "full", "economic"];
        if !valid_types.contains(&alliance_type) {
            return Err(format!(
                "Invalid alliance_type '{}'. Must be one of: defensive, full, economic",
                alliance_type
            ));
        }

        let pid = self
            .state
            .player_country_id
            .as_ref()
            .ok_or("No player country selected")?
            .clone();

        if pid == target_id {
            return Err("Cannot ally with yourself".to_string());
        }
        if !self.state.countries.contains_key(target_id) {
            return Err(format!("Target country not found: {}", target_id));
        }

        let target_name = self
            .state
            .countries
            .get(target_id)
            .map(|c| c.name.as_str())
            .unwrap_or(target_id);
        let player_name = self
            .state
            .countries
            .get(&pid)
            .map(|c| c.name.as_str())
            .unwrap_or("You");

        // Set diplomatic stance to Allied
        self.state
            .set_diplomatic_stance(&pid, target_id, DiplomaticStance::Allied);

        // Add MilitaryAlliance treaty
        if let Some(rel) = self.state.diplomatic_relations.iter_mut().find(|r| {
            (r.country_a == pid && r.country_b == target_id)
                || (r.country_a == target_id && r.country_b == pid)
        }) {
            if !rel.treaties.contains(&Treaty::MilitaryAlliance) {
                rel.treaties.push(Treaty::MilitaryAlliance);
            }
            rel.trust = (rel.trust + 30.0).min(100.0);
        }

        // Add to alliances vec on both sides
        if let Some(country) = self.state.countries.get_mut(&pid) {
            if !country.alliances.contains(&target_id.to_string()) {
                country.alliances.push(target_id.to_string());
            }
        }
        if let Some(country) = self.state.countries.get_mut(target_id) {
            if !country.alliances.contains(&pid) {
                country.alliances.push(pid.clone());
            }
        }

        self.state.global_events.push(GlobalEvent {
            turn: self.state.turn_number,
            year: self.state.current_year,
            month: self.state.current_month,
            event_type: "alliance_formed".to_string(),
            description: format!(
                "{} and {} formed a {} alliance",
                player_name, target_name, alliance_type
            ),
            involved_countries: vec![pid, target_id.to_string()],
        });

        let result = json!({
            "status": "success",
            "message": format!("Alliance ({}) formed with {}", alliance_type, target_name),
            "alliance_type": alliance_type,
        });

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }

    fn pass_policy(&mut self, args: &Value) -> Result<String, String> {
        let policy_name = args["policy_name"]
            .as_str()
            .ok_or("Missing 'policy_name'")?;
        let description = args["description"]
            .as_str()
            .ok_or("Missing 'description'")?;

        let pid = self
            .state
            .player_country_id
            .as_ref()
            .ok_or("No player country selected")?
            .clone();

        let player_name = self
            .state
            .countries
            .get(&pid)
            .map(|c| c.name.as_str())
            .unwrap_or("Your country");

        // Adjust stability slightly upward (simulates decisive government action)
        if let Some(country) = self.state.countries.get_mut(&pid) {
            country.stability = (country.stability + 2.0).min(100.0);
        }

        self.state.global_events.push(GlobalEvent {
            turn: self.state.turn_number,
            year: self.state.current_year,
            month: self.state.current_month,
            event_type: "policy_passed".to_string(),
            description: format!("{} enacted policy '{}': {}", player_name, policy_name, description),
            involved_countries: vec![pid],
        });

        let result = json!({
            "status": "success",
            "message": format!("Policy '{}' enacted in {}", policy_name, player_name),
            "policy_name": policy_name,
            "description": description,
        });

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }

    fn set_budget(&mut self, args: &Value) -> Result<String, String> {
        let military_pct = args["military_pct"].as_f64().ok_or("Missing 'military_pct'")?;
        let infrastructure_pct = args["infrastructure_pct"]
            .as_f64()
            .ok_or("Missing 'infrastructure_pct'")?;
        let welfare_pct = args["welfare_pct"].as_f64().ok_or("Missing 'welfare_pct'")?;
        let research_pct = args["research_pct"].as_f64().ok_or("Missing 'research_pct'")?;

        let pid = self
            .state
            .player_country_id
            .as_ref()
            .ok_or("No player country selected")?
            .clone();

        let player_name = self
            .state
            .countries
            .get(&pid)
            .map(|c| c.name.as_str())
            .unwrap_or("Your country");

        self.state.global_events.push(GlobalEvent {
            turn: self.state.turn_number,
            year: self.state.current_year,
            month: self.state.current_month,
            event_type: "budget_set".to_string(),
            description: format!(
                "{} set budget allocation: Military {}%, Infrastructure {}%, Welfare {}%, Research {}%",
                player_name, military_pct, infrastructure_pct, welfare_pct, research_pct
            ),
            involved_countries: vec![pid],
        });

        let result = json!({
            "status": "success",
            "message": format!("Budget allocated for {}", player_name),
            "military_pct": military_pct,
            "infrastructure_pct": infrastructure_pct,
            "welfare_pct": welfare_pct,
            "research_pct": research_pct,
        });

        Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
    }

    fn advance_turn(&mut self) -> Result<String, String> {
        // Advance time
        self.state.advance_time();

        // Process economy
        let economic_changes = self.state.process_economy();

        // Process war resolution
        let war_updates = self.state.process_war_resolution();

        // Build summary
        let mut summary = format!(
            "=== Turn {} Complete ===\nDate: {}/{}\nScenario: {}\n\n",
            self.state.turn_number,
            self.state.current_month,
            self.state.current_year,
            self.state.scenario_name,
        );

        summary.push_str("-- Economic Changes --\n");
        if economic_changes.is_empty() {
            summary.push_str("  No economic changes.\n");
        } else {
            for change in &economic_changes {
                let cname = self
                    .state
                    .countries
                    .get(&change.country_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or(&change.country_id);
                summary.push_str(&format!("  {}: GDP {:.2}B (growth {:.1}%)\n", cname, change.gdp_change, change.gdp_growth_new));
            }
        }

        summary.push_str("\n-- War Updates --\n");
        if war_updates.is_empty() {
            summary.push_str("  No changes on active fronts.\n");
        } else {
            for update in &war_updates {
                summary.push_str(&format!("  {}\n", update.description));
            }
        }

        // Build TurnSummary
        let turn_summary = game::TurnSummary {
            turn: self.state.turn_number,
            year: self.state.current_year,
            month: self.state.current_month,
            player_action: "Turn advanced".to_string(),
            results: economic_changes
                .iter()
                .map(|e| e.description.clone())
                .collect(),
            territory_changes: Vec::new(),
            war_updates,
            economic_changes,
            new_events: Vec::new(),
        };

        self.state.turn_history.push(turn_summary);

        Ok(serde_json::to_string_pretty(&json!({
            "status": "success",
            "summary": summary,
            "turn": self.state.turn_number,
            "month": self.state.current_month,
            "year": self.state.current_year,
        }))
        .unwrap_or_default())
    }
}
