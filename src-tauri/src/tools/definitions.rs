use crate::llm::ToolDefinition;
use serde_json::json;

/// Returns all available game tools as LLM-compatible tool definitions.
pub fn get_all_tools() -> Vec<ToolDefinition> {
    vec![
        // 1. get_country_status
        ToolDefinition {
            name: "get_country_status".to_string(),
            description: "Get full information about a country by its ID.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "country_id": {
                        "type": "string",
                        "description": "The unique identifier of the country"
                    }
                },
                "required": ["country_id"]
            }),
        },
        // 2. get_player_status
        ToolDefinition {
            name: "get_player_status".to_string(),
            description: "Get information about the player's own country.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        // 3. get_neighbors
        ToolDefinition {
            name: "get_neighbors".to_string(),
            description: "Get a list of neighboring countries and their diplomatic stances.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "country_id": {
                        "type": "string",
                        "description": "The country ID to get neighbors for"
                    }
                },
                "required": ["country_id"]
            }),
        },
        // 4. get_world_situation
        ToolDefinition {
            name: "get_world_situation".to_string(),
            description: "Get a summary of all active wars, world events, and global economy.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        // 5. declare_war
        ToolDefinition {
            name: "declare_war".to_string(),
            description: "Declare war on a target country with a justification and military resource allocation.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "target_country": {
                        "type": "string",
                        "description": "The country ID to declare war on"
                    },
                    "justification": {
                        "type": "string",
                        "description": "Reason for declaring war"
                    },
                    "military_allocation_pct": {
                        "type": "number",
                        "description": "Percentage of military forces to allocate (0-100)",
                        "minimum": 0,
                        "maximum": 100
                    }
                },
                "required": ["target_country", "justification", "military_allocation_pct"]
            }),
        },
        // 6. mobilize_forces
        ToolDefinition {
            name: "mobilize_forces".to_string(),
            description: "Mobilize military forces to a target border with a specified force size and focus (land, air, or naval).".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "target_border": {
                        "type": "string",
                        "description": "Country ID of the target border"
                    },
                    "force_size_pct": {
                        "type": "number",
                        "description": "Percentage of forces to mobilize (0-100)",
                        "minimum": 0,
                        "maximum": 100
                    },
                    "focus": {
                        "type": "string",
                        "description": "Military focus area",
                        "enum": ["land", "air", "naval"]
                    }
                },
                "required": ["target_border", "force_size_pct", "focus"]
            }),
        },
        // 7. sue_for_peace
        ToolDefinition {
            name: "sue_for_peace".to_string(),
            description: "Offer peace terms to a country you are at war with.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "target_country": {
                        "type": "string",
                        "description": "The country ID to offer peace to"
                    },
                    "terms": {
                        "type": "string",
                        "description": "The peace terms being offered"
                    }
                },
                "required": ["target_country", "terms"]
            }),
        },
        // 8. negotiate_trade
        ToolDefinition {
            name: "negotiate_trade".to_string(),
            description: "Propose a trade deal with another country, offering and requesting resources.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "target_country": {
                        "type": "string",
                        "description": "The country ID to trade with"
                    },
                    "offer_resources": {
                        "type": "array",
                        "description": "List of resources being offered",
                        "items": { "type": "string" }
                    },
                    "request_resources": {
                        "type": "array",
                        "description": "List of resources being requested",
                        "items": { "type": "string" }
                    }
                },
                "required": ["target_country", "offer_resources", "request_resources"]
            }),
        },
        // 9. form_alliance
        ToolDefinition {
            name: "form_alliance".to_string(),
            description: "Form an alliance pact with another country (defensive, full, or economic).".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "target_country": {
                        "type": "string",
                        "description": "The country ID to ally with"
                    },
                    "alliance_type": {
                        "type": "string",
                        "description": "Type of alliance to form",
                        "enum": ["defensive", "full", "economic"]
                    }
                },
                "required": ["target_country", "alliance_type"]
            }),
        },
        // 10. pass_policy
        ToolDefinition {
            name: "pass_policy".to_string(),
            description: "Enact a domestic policy affecting your country.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "policy_name": {
                        "type": "string",
                        "description": "Name of the policy to enact"
                    },
                    "description": {
                        "type": "string",
                        "description": "Detailed description of the policy"
                    }
                },
                "required": ["policy_name", "description"]
            }),
        },
        // 11. set_budget
        ToolDefinition {
            name: "set_budget".to_string(),
            description: "Allocate the national budget across military, infrastructure, welfare, and research as percentages of GDP.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "military_pct": {
                        "type": "number",
                        "description": "Percentage of GDP for military spending",
                        "minimum": 0,
                        "maximum": 100
                    },
                    "infrastructure_pct": {
                        "type": "number",
                        "description": "Percentage of GDP for infrastructure",
                        "minimum": 0,
                        "maximum": 100
                    },
                    "welfare_pct": {
                        "type": "number",
                        "description": "Percentage of GDP for welfare programs",
                        "minimum": 0,
                        "maximum": 100
                    },
                    "research_pct": {
                        "type": "number",
                        "description": "Percentage of GDP for research and development",
                        "minimum": 0,
                        "maximum": 100
                    }
                },
                "required": ["military_pct", "infrastructure_pct", "welfare_pct", "research_pct"]
            }),
        },
        // 12. advance_turn
        ToolDefinition {
            name: "advance_turn".to_string(),
            description: "Advance the game by one turn, resolving all pending actions, wars, and economic updates.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}
