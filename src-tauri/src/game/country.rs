use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: String,
    pub name: String,
    pub capital: String,
    pub continent: String,
    pub population: u64,
    pub gdp: f64,
    pub gdp_growth: f64,
    pub military_power: MilitaryPower,
    pub tech_level: TechLevel,
    pub government_type: String,
    pub ideology: String,
    pub stability: f64,
    pub resources: Vec<String>,
    pub neighbors: Vec<String>,
    pub active_wars: Vec<String>,
    pub alliances: Vec<String>,
    pub trade_partners: Vec<TradeRelation>,
    pub controlled_territory: Vec<String>,
    pub is_player_controlled: bool,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilitaryPower {
    pub army_strength: f64,
    pub navy_strength: f64,
    pub air_force_strength: f64,
    pub mobilization_pct: f64,
    pub manpower_reserves: u64,
    pub technology_bonus: f64,
    pub morale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechLevel {
    PreIndustrial,
    EarlyIndustrial,
    Industrial,
    Modern,
    Advanced,
    Futuristic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRelation {
    pub partner_id: String,
    pub export_goods: Vec<String>,
    pub import_goods: Vec<String>,
    pub trade_volume: f64,
    pub status: TradeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeStatus {
    Active,
    Strained,
    Embargoed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticRelation {
    pub country_a: String,
    pub country_b: String,
    pub stance: DiplomaticStance,
    pub trust: f64,
    pub treaties: Vec<Treaty>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiplomaticStance {
    Allied,
    Friendly,
    Neutral,
    Wary,
    Hostile,
    AtWar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Treaty {
    NonAggressionPact,
    TradeAgreement,
    MilitaryAlliance,
    PeaceTreaty { conditions: String },
}
