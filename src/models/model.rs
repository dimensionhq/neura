use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Model {
    GPT4,
    GPT3Turbo,
    ClaudeV1,
}

impl Model {
    pub fn from_input(input: &str) -> Self {
        match input {
            "🤖 GPT 4.0" => Self::GPT4,
            "🐇 GPT 3.5 Turbo" => Self::GPT3Turbo,
            "💫 Claude v1" => Self::ClaudeV1,
            _ => panic!("Invalid model"),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::GPT4 => "GPT 4.0".to_string(),
            Self::GPT3Turbo => "GPT 3.5 Turbo".to_string(),
            Self::ClaudeV1 => "Claude v1".to_string(),
        }
    }

    pub fn code(&self) -> String {
        match self {
            Self::GPT4 => "gpt-4".to_string(),
            Self::GPT3Turbo => "gpt-3.5-turbo".to_string(),
            Self::ClaudeV1 => "claude-v1".to_string(),
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "gpt-4" => Self::GPT4,
            "gpt-3.5-turbo" => Self::GPT3Turbo,
            "claude-v1" => Self::ClaudeV1,
            _ => panic!("Invalid model"),
        }
    }
}
