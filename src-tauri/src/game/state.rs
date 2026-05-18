use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub scenario_name: String,
    pub current_year: i32,
    pub current_month: i32,
    pub turn_number: u64,
    pub countries: HashMap<String, Country>,
    pub wars: HashMap<String, War>,
    pub diplomatic_relations: Vec<DiplomaticRelation>,
    pub player_country_id: Option<String>,
    pub global_events: Vec<GlobalEvent>,
    pub turn_history: Vec<TurnSummary>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEvent {
    pub turn: u64,
    pub year: i32,
    pub month: i32,
    pub event_type: String,
    pub description: String,
    pub involved_countries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnSummary {
    pub turn: u64,
    pub year: i32,
    pub month: i32,
    pub player_action: String,
    pub results: Vec<String>,
    pub territory_changes: Vec<TerritoryChange>,
    pub war_updates: Vec<WarUpdate>,
    pub economic_changes: Vec<EconomicChange>,
    pub new_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryChange {
    pub region: String,
    pub from_country: String,
    pub to_country: String,
    pub change_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarUpdate {
    pub war_id: String,
    pub description: String,
    pub attacker_gains: Vec<String>,
    pub defender_gains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicChange {
    pub country_id: String,
    pub gdp_change: f64,
    pub gdp_growth_new: f64,
    pub description: String,
}

impl GameState {
    pub fn new(scenario_name: &str) -> Self {
        GameState {
            scenario_name: scenario_name.to_string(),
            current_year: 1939,
            current_month: 9,
            turn_number: 0,
            countries: HashMap::new(),
            wars: HashMap::new(),
            diplomatic_relations: Vec::new(),
            player_country_id: None,
            global_events: Vec::new(),
            turn_history: Vec::new(),
            active: false,
        }
    }

    pub fn get_player_country(&self) -> Option<&Country> {
        self.player_country_id
            .as_ref()
            .and_then(|id| self.countries.get(id))
    }

    pub fn get_player_country_mut(&mut self) -> Option<&mut Country> {
        self.player_country_id
            .as_ref()
            .and_then(|id| self.countries.get_mut(id))
    }

    pub fn find_countries_at_war_with(&self, country_id: &str) -> Vec<String> {
        let mut enemies = Vec::new();
        for war in self.wars.values() {
            if war.status == WarStatus::Ongoing {
                if war.attackers.contains(&country_id.to_string()) {
                    enemies.extend(war.defenders.iter().cloned());
                }
                if war.defenders.contains(&country_id.to_string()) {
                    enemies.extend(war.attackers.iter().cloned());
                }
            }
        }
        enemies.sort();
        enemies.dedup();
        enemies
    }

    pub fn get_diplomatic_stance(&self, country_a: &str, country_b: &str) -> DiplomaticStance {
        self.diplomatic_relations
            .iter()
            .find(|r| {
                (r.country_a == country_a && r.country_b == country_b)
                    || (r.country_a == country_b && r.country_b == country_a)
            })
            .map(|r| r.stance.clone())
            .unwrap_or(DiplomaticStance::Neutral)
    }

    pub fn set_diplomatic_stance(&mut self, country_a: &str, country_b: &str, stance: DiplomaticStance) {
        if let Some(relation) = self.diplomatic_relations.iter_mut().find(|r| {
            (r.country_a == country_a && r.country_b == country_b)
                || (r.country_a == country_b && r.country_b == country_a)
        }) {
            relation.stance = stance;
        } else {
            self.diplomatic_relations.push(DiplomaticRelation {
                country_a: country_a.to_string(),
                country_b: country_b.to_string(),
                stance,
                trust: 0.0,
                treaties: Vec::new(),
            });
        }
    }

    pub fn advance_time(&mut self) {
        self.current_month += 1;
        if self.current_month > 12 {
            self.current_month = 1;
            self.current_year += 1;
        }
        self.turn_number += 1;
    }
}
