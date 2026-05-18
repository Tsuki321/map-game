use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticAction {
    pub actor: String,
    pub target: String,
    pub action_type: DiplomaticActionType,
    pub timestamp_turn: u64,
    pub accepted: Option<bool>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiplomaticActionType {
    ProposeAlliance,
    BreakAlliance,
    ProposeTrade,
    DeclareWar,
    SueForPeace,
    NonAggressionPact,
    GuaranteeIndependence,
    SendAid { amount: f64 },
    ImposeEmbargo,
    LiftEmbargo,
}
