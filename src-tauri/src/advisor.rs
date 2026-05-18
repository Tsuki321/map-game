use crate::game::GameState;
use crate::llm::provider::{ChatMessage, LlmConfig, LlmProvider};

pub async fn get_advisor_suggestion(
    state: &GameState,
    directive: &str,
    _config: &LlmConfig,
    provider: &dyn LlmProvider,
) -> Result<String, String> {
    let player_name = state
        .get_player_country()
        .map(|c| c.name.as_str())
        .unwrap_or("Unknown");

    let system_prompt = format!(
        r#"You are a strategic advisor to the leader of {}. Your role is to analyze the proposed course of action and provide:

1. RISK ASSESSMENT: What could go wrong?
2. STRATEGIC CONTEXT: What factors should be considered?
3. ALTERNATIVE APPROACHES: What other options exist?
4. REFINED DIRECTIVE: A clearer, more strategic version of the directive.

Be concise but thorough. The player will see your advice before executing the directive."#,
        player_name
    );

    let mut context_str = String::new();
    if let Some(country) = state.get_player_country() {
        context_str.push_str(&format!("Your country: {}\n", country.name));
        context_str.push_str(&format!("Military: Army {:.0}, Navy {:.0}, Air {:.0}\n",
            country.military_power.army_strength, country.military_power.navy_strength, country.military_power.air_force_strength));
        context_str.push_str(&format!("Economy: GDP ${:.2}B, Growth {:.1}%\n", country.gdp, country.gdp_growth));
        context_str.push_str(&format!("Stability: {:.0}%, Tech: {:?}\n", country.stability, country.tech_level));
        if !country.active_wars.is_empty() {
            context_str.push_str(&format!("Currently at war with: {}\n", country.active_wars.join(", ")));
        }
    }

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!("Country context:\n{}\n\nPlayer's proposed directive:\n{}", context_str, directive),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ];

    let response = provider.chat(messages, None).await?;
    Ok(response.message.content)
}
