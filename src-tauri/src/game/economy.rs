use super::*;
use rand::Rng;

impl GameState {
    pub fn process_economy(&mut self) -> Vec<EconomicChange> {
        let mut changes = Vec::new();

        for country in self.countries.values_mut() {
            let mut growth_rate = country.gdp_growth;
            let country_id = country.id.clone();
            let old_gdp = country.gdp;

            // Subtract 2% if at war
            if !country.active_wars.is_empty() {
                growth_rate -= 2.0;
            }

            // Subtract 1.5% if mobilization > 50%
            if country.military_power.mobilization_pct > 50.0 {
                growth_rate -= 1.5;
            }

            // Subtract 1% if any embargoed trade
            if country
                .trade_partners
                .iter()
                .any(|t| t.status == TradeStatus::Embargoed)
            {
                growth_rate -= 1.0;
            }

            // Subtract 1% if stability < 30
            if country.stability < 30.0 {
                growth_rate -= 1.0;
            }

            country.gdp_growth = growth_rate;

            // Apply growth rate to GDP
            let gdp_change = old_gdp * (growth_rate / 100.0);
            let new_gdp = (old_gdp + gdp_change).max(0.01);
            country.gdp = new_gdp;

            changes.push(EconomicChange {
                country_id,
                gdp_change,
                gdp_growth_new: growth_rate,
                description: format!(
                    "{}: GDP changed by {:.2}B (growth: {:.1}%)",
                    country.name, gdp_change, growth_rate
                ),
            });
        }

        changes
    }

    pub fn process_war_resolution(&mut self) -> Vec<WarUpdate> {
        let mut rng = rand::thread_rng();
        let mut updates = Vec::new();
        let mut territory_to_transfer: Vec<(String, String)> = Vec::new(); // (from, to)

        let ongoing_war_ids: Vec<String> = self
            .wars
            .iter()
            .filter(|(_, w)| w.status == WarStatus::Ongoing)
            .map(|(id, _)| id.clone())
            .collect();

        for war_id in &ongoing_war_ids {
            // Destructure to get mutable wars and immutable countries separately
            let GameState {
                ref mut wars,
                ref countries,
                ..
            } = *self;

            let war = match wars.get_mut(war_id) {
                Some(w) => w,
                None => continue,
            };

            for front in &mut war.fronts {
                if !front.active {
                    continue;
                }

                let attacker_id = match war.attackers.first() {
                    Some(id) => id.clone(),
                    None => continue,
                };
                let defender_id = match war.defenders.first() {
                    Some(id) => id.clone(),
                    None => continue,
                };

                let attacker = match countries.get(&attacker_id) {
                    Some(c) => c,
                    None => continue,
                };
                let defender = match countries.get(&defender_id) {
                    Some(c) => c,
                    None => continue,
                };

                let (attacker_won, description) =
                    war.resolve_military_engagement(attacker, defender, front, &mut rng);

                let attacker_gains = if attacker_won {
                    vec![front.name.clone()]
                } else {
                    vec![]
                };
                let defender_gains = if !attacker_won {
                    vec![front.name.clone()]
                } else {
                    vec![]
                };

                updates.push(WarUpdate {
                    war_id: war_id.clone(),
                    description,
                    attacker_gains,
                    defender_gains,
                });

                if front.attacker_progress >= 100.0 {
                    front.active = false;
                    territory_to_transfer.push((defender_id.clone(), attacker_id.clone()));
                }
            }
        }

        // Apply territory transfers after the destructured borrow ends
        for (from_id, to_id) in &territory_to_transfer {
            if let Some(to_country) = self.countries.get_mut(to_id) {
                if !to_country.controlled_territory.contains(from_id) {
                    to_country.controlled_territory.push(from_id.clone());
                }
            }
            if let Some(from_country) = self.countries.get_mut(from_id) {
                from_country.controlled_territory.retain(|id| id != from_id);
            }
        }

        updates
    }
}
