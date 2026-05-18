use super::*;
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct War {
    pub id: String,
    pub name: String,
    pub attackers: Vec<String>,
    pub defenders: Vec<String>,
    pub start_turn: u64,
    pub start_year: i32,
    pub start_month: i32,
    pub end_turn: Option<u64>,
    pub status: WarStatus,
    pub fronts: Vec<Front>,
    pub war_type: WarType,
    pub justification: String,
    pub war_score: f64,
    pub battles: Vec<Battle>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarStatus {
    Ongoing,
    AttackerVictory,
    DefenderVictory,
    Stalemate,
    Ceasefire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarType {
    TotalWar,
    LimitedWar,
    ProxyWar,
    NavalWar,
    BorderConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Front {
    pub name: String,
    pub location: String,
    pub attacker_progress: f64,
    pub involved_countries: Vec<String>,
    pub terrain: TerrainType,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerrainType {
    Plains,
    Mountains,
    Desert,
    Forest,
    Urban,
    Naval,
    Coastal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Battle {
    pub name: String,
    pub location: String,
    pub turn: u64,
    pub attacker: String,
    pub defender: String,
    pub outcome: BattleOutcome,
    pub casualties_attacker: u64,
    pub casualties_defender: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleOutcome {
    AttackerVictory,
    DefenderVictory,
    PyrrhicAttacker,
    PyrrhicDefender,
    Stalemate,
    Ongoing,
}

impl War {
    pub fn is_involved(&self, country_id: &str) -> bool {
        self.attackers.contains(&country_id.to_string())
            || self.defenders.contains(&country_id.to_string())
    }

    pub fn resolve_military_engagement(
        &mut self,
        attacker: &Country,
        defender: &Country,
        front: &mut Front,
        rng: &mut impl Rng,
    ) -> (bool, String) {
        let mp_a = &attacker.military_power;
        let mp_d = &defender.military_power;

        let attacker_power = mp_a.army_strength
            * (1.0 + mp_a.technology_bonus)
            * (mp_a.morale / 100.0)
            * (mp_a.mobilization_pct / 100.0);

        let defender_power = mp_d.army_strength
            * (1.0 + mp_d.technology_bonus)
            * (mp_d.morale / 100.0)
            * (mp_d.mobilization_pct / 100.0)
            * 1.2;

        let terrain_modifier = match front.terrain {
            TerrainType::Plains => 1.0,
            TerrainType::Mountains => 0.5,
            TerrainType::Desert => 0.7,
            TerrainType::Forest => 0.8,
            TerrainType::Urban => 0.4,
            TerrainType::Coastal => 0.8,
            TerrainType::Naval => 1.0,
        };

        let luck = rng.gen_range(0.8..1.2);
        let ratio = (attacker_power * terrain_modifier * luck) / defender_power.max(0.1);

        if ratio > 1.3 {
            let advance = rng.gen_range(15.0..30.0);
            front.attacker_progress = (front.attacker_progress + advance).min(100.0);
            let desc = format!(
                "{} advances on the {} front! Progress: {}%",
                attacker.name,
                front.name,
                front.attacker_progress as i32
            );
            (true, desc)
        } else if ratio > 0.8 {
            let advance = rng.gen_range(0.0..10.0);
            front.attacker_progress = (front.attacker_progress + advance).min(100.0);
            let desc = format!(
                "Minimal gains for {} on the {} front. Progress: {}%",
                attacker.name,
                front.name,
                front.attacker_progress as i32
            );
            (false, desc)
        } else {
            let pushback = rng.gen_range(10.0..20.0);
            front.attacker_progress = (front.attacker_progress - pushback).max(0.0);
            let desc = format!(
                "{} pushes {} back on the {} front! Progress: {}%",
                defender.name,
                attacker.name,
                front.name,
                front.attacker_progress as i32
            );
            (false, desc)
        }
    }
}
