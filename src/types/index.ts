export interface Country {
  id: string;
  name: string;
  capital: string;
  continent: string;
  population: number;
  gdp: number;
  gdp_growth: number;
  military_power: MilitaryPower;
  tech_level: TechLevel;
  government_type: string;
  ideology: string;
  stability: number;
  resources: string[];
  neighbors: string[];
  active_wars: string[];
  alliances: string[];
  trade_partners: TradeRelation[];
  controlled_territory: string[];
  is_player_controlled: boolean;
  color: string;
}

export interface MilitaryPower {
  army_strength: number;
  navy_strength: number;
  air_force_strength: number;
  mobilization_pct: number;
  manpower_reserves: number;
  technology_bonus: number;
  morale: number;
}

export type TechLevel =
  | 'PreIndustrial'
  | 'EarlyIndustrial'
  | 'Industrial'
  | 'Modern'
  | 'Advanced'
  | 'Futuristic';

export interface TradeRelation {
  partner_id: string;
  export_goods: string[];
  import_goods: string[];
  trade_volume: number;
  status: 'Active' | 'Strained' | 'Embargoed';
}

export interface GameState {
  scenario_name: string;
  current_year: number;
  current_month: number;
  turn_number: number;
  countries: Record<string, Country>;
  player_country_id: string | null;
  global_events: GlobalEvent[];
  turn_history: TurnSummary[];
  active: boolean;
}

export interface GlobalEvent {
  turn: number;
  year: number;
  month: number;
  event_type: string;
  description: string;
  involved_countries: string[];
}

export interface TurnSummary {
  turn: number;
  year: number;
  month: number;
  player_action: string;
  results: string[];
  territory_changes: TerritoryChange[];
  war_updates: any[];
  economic_changes: any[];
  new_events: string[];
}

export interface TerritoryChange {
  region: string;
  from_country: string;
  to_country: string;
  change_type: string;
}

export interface LlmConfig {
  provider_type: string;
  api_key?: string;
  model: string;
  endpoint?: string;
}

export interface AppSettings {
  llmConfig: LlmConfig | null;
  advisorEnabled: boolean;
}
