use serde::{Deserialize, Serialize};

/// Token counts from raw JSONL (snake_case field names)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenCounts {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

impl TokenCounts {
    pub fn total(&self) -> u64 {
        self.input_tokens
            + self.output_tokens
            + self.cache_creation_input_tokens
            + self.cache_read_input_tokens
    }

    pub fn add(&mut self, other: &TokenCounts) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_creation_input_tokens += other.cache_creation_input_tokens;
        self.cache_read_input_tokens += other.cache_read_input_tokens;
    }
}

/// Aggregated token counts (camelCase field names for consistency with original)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregatedTokenCounts {
    #[serde(rename = "inputTokens")]
    pub input_tokens: u64,
    #[serde(rename = "outputTokens")]
    pub output_tokens: u64,
    #[serde(rename = "cacheCreationTokens")]
    pub cache_creation_tokens: u64,
    #[serde(rename = "cacheReadTokens")]
    pub cache_read_tokens: u64,
}

impl AggregatedTokenCounts {
    pub fn add_from_raw(&mut self, tokens: &TokenCounts) {
        self.input_tokens += tokens.input_tokens;
        self.output_tokens += tokens.output_tokens;
        self.cache_creation_tokens += tokens.cache_creation_input_tokens;
        self.cache_read_tokens += tokens.cache_read_input_tokens;
    }
}

impl From<TokenCounts> for AggregatedTokenCounts {
    fn from(tokens: TokenCounts) -> Self {
        Self {
            input_tokens: tokens.input_tokens,
            output_tokens: tokens.output_tokens,
            cache_creation_tokens: tokens.cache_creation_input_tokens,
            cache_read_tokens: tokens.cache_read_input_tokens,
        }
    }
}

impl From<&TokenCounts> for AggregatedTokenCounts {
    fn from(tokens: &TokenCounts) -> Self {
        Self {
            input_tokens: tokens.input_tokens,
            output_tokens: tokens.output_tokens,
            cache_creation_tokens: tokens.cache_creation_input_tokens,
            cache_read_tokens: tokens.cache_read_input_tokens,
        }
    }
}