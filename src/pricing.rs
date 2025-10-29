use crate::types::{ModelName, TokenCounts};
use anyhow::Result;
use lazy_static::lazy_static;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

/// Model pricing information
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_price: Decimal,         // Price per 1M input tokens
    pub output_price: Decimal,        // Price per 1M output tokens
    pub cache_creation_price: Decimal, // Price per 1M cache creation tokens
    pub cache_read_price: Decimal,    // Price per 1M cache read tokens
}

impl ModelPricing {
    pub fn calculate_cost(&self, tokens: &TokenCounts) -> Decimal {
        let million = dec!(1_000_000);

        let input_cost = (Decimal::from(tokens.input_tokens) / million) * self.input_price;
        let output_cost = (Decimal::from(tokens.output_tokens) / million) * self.output_price;
        let cache_creation_cost =
            (Decimal::from(tokens.cache_creation_input_tokens) / million) * self.cache_creation_price;
        let cache_read_cost =
            (Decimal::from(tokens.cache_read_input_tokens) / million) * self.cache_read_price;

        input_cost + output_cost + cache_creation_cost + cache_read_cost
    }
}

lazy_static! {
    /// Hard-coded pricing data for Claude models (as of 2025)
    /// Prices are per 1M tokens
    static ref MODEL_PRICING: HashMap<&'static str, ModelPricing> = {
        let mut m = HashMap::new();

        // Claude 4 Opus
        m.insert("claude-opus-4-20250514", ModelPricing {
            input_price: dec!(15.00),
            output_price: dec!(75.00),
            cache_creation_price: dec!(18.75),  // 1.25x input
            cache_read_price: dec!(1.50),       // 0.1x input
        });

        m.insert("claude-opus-4-1-20250805", ModelPricing {
            input_price: dec!(15.00),
            output_price: dec!(75.00),
            cache_creation_price: dec!(18.75),
            cache_read_price: dec!(1.50),
        });

        // Claude 4.5 Sonnet (new)
        m.insert("claude-sonnet-4-5-20250929", ModelPricing {
            input_price: dec!(3.00),
            output_price: dec!(15.00),
            cache_creation_price: dec!(3.75),   // 1.25x input
            cache_read_price: dec!(0.30),       // 0.1x input
        });

        // Claude 4 Sonnet
        m.insert("claude-sonnet-4-20250514", ModelPricing {
            input_price: dec!(3.00),
            output_price: dec!(15.00),
            cache_creation_price: dec!(3.75),   // 1.25x input
            cache_read_price: dec!(0.30),       // 0.1x input
        });

        m.insert("claude-sonnet-4-1-20250805", ModelPricing {
            input_price: dec!(3.00),
            output_price: dec!(15.00),
            cache_creation_price: dec!(3.75),
            cache_read_price: dec!(0.30),
        });

        // Claude 3.5 Sonnet (legacy)
        m.insert("claude-3-5-sonnet-20241022", ModelPricing {
            input_price: dec!(3.00),
            output_price: dec!(15.00),
            cache_creation_price: dec!(3.75),
            cache_read_price: dec!(0.30),
        });

        m.insert("claude-3-5-sonnet-20240620", ModelPricing {
            input_price: dec!(3.00),
            output_price: dec!(15.00),
            cache_creation_price: dec!(3.75),
            cache_read_price: dec!(0.30),
        });

        // Claude 3 Opus (legacy)
        m.insert("claude-3-opus-20240229", ModelPricing {
            input_price: dec!(15.00),
            output_price: dec!(75.00),
            cache_creation_price: dec!(18.75),
            cache_read_price: dec!(1.50),
        });

        // Claude 4.5 Haiku (new)
        m.insert("claude-haiku-4-5-20251001", ModelPricing {
            input_price: dec!(1.00),
            output_price: dec!(5.00),
            cache_creation_price: dec!(1.25),
            cache_read_price: dec!(0.10),
        });

        // Claude 3.5 Haiku
        m.insert("claude-3-5-haiku-20241022", ModelPricing {
            input_price: dec!(1.00),
            output_price: dec!(5.00),
            cache_creation_price: dec!(1.25),
            cache_read_price: dec!(0.10),
        });

        // Claude 3 Haiku
        m.insert("claude-3-haiku-20240307", ModelPricing {
            input_price: dec!(0.25),
            output_price: dec!(1.25),
            cache_creation_price: dec!(0.30),
            cache_read_price: dec!(0.03),
        });

        m
    };
}

/// Pricing fetcher for calculating costs
#[derive(Clone)]
pub struct PricingFetcher {
    offline: bool,
    #[cfg(feature = "online-pricing")]
    client: Option<Arc<reqwest::Client>>,
    custom_pricing: Arc<HashMap<String, ModelPricing>>,
}

impl PricingFetcher {
    pub fn new(offline: bool) -> Self {
        Self {
            offline,
            #[cfg(feature = "online-pricing")]
            client: if !offline {
                Some(Arc::new(reqwest::Client::new()))
            } else {
                None
            },
            custom_pricing: Arc::new(HashMap::new()),
        }
    }

    /// Fuzzy match model names to find pricing
    /// Handles variations like claude-sonnet-4-5-YYYYMMDD -> claude-sonnet-4-5
    fn find_matching_model(model_name: &str) -> Option<&'static str> {
        // First try exact match - find the key in the static map
        for key in MODEL_PRICING.keys() {
            if *key == model_name {
                return Some(*key);
            }
        }

        // Try fuzzy matching for known patterns
        let model_lower = model_name.to_lowercase();

        // Extract model family and version
        let parts: Vec<&str> = model_lower.split('-').collect();

        if parts.len() >= 3 {
            // Match patterns like claude-{type}-{version}-{date}
            // Examples: claude-sonnet-4-5-20250929, claude-haiku-4-5-20251001

            // Check for Opus models
            if model_lower.contains("opus") {
                if model_lower.contains("4-1") || model_lower.contains("4.1") {
                    return Some("claude-opus-4-1-20250805");
                } else if model_lower.contains("4") {
                    return Some("claude-opus-4-20250514");
                } else if model_lower.contains("3") {
                    return Some("claude-3-opus-20240229");
                }
            }

            // Check for Sonnet models
            if model_lower.contains("sonnet") {
                if model_lower.contains("4-5") || model_lower.contains("4.5") {
                    return Some("claude-sonnet-4-5-20250929");
                } else if model_lower.contains("4-1") || model_lower.contains("4.1") {
                    return Some("claude-sonnet-4-1-20250805");
                } else if model_lower.contains("4") {
                    return Some("claude-sonnet-4-20250514");
                } else if model_lower.contains("3-5") || model_lower.contains("3.5") {
                    return Some("claude-3-5-sonnet-20241022");
                }
            }

            // Check for Haiku models
            if model_lower.contains("haiku") {
                if model_lower.contains("4-5") || model_lower.contains("4.5") {
                    return Some("claude-haiku-4-5-20251001");
                } else if model_lower.contains("3-5") || model_lower.contains("3.5") {
                    return Some("claude-3-5-haiku-20241022");
                } else if model_lower.contains("3") {
                    return Some("claude-3-haiku-20240307");
                }
            }
        }

        None
    }

    /// Calculate cost for a given model and token counts
    pub async fn calculate_cost(
        &self,
        model: &ModelName,
        tokens: &TokenCounts,
    ) -> Result<Decimal> {
        // Check custom pricing first
        if let Some(pricing) = self.custom_pricing.get(model.as_str()) {
            return Ok(pricing.calculate_cost(tokens));
        }

        // Try fuzzy matching to find a known model
        if let Some(matched_model) = Self::find_matching_model(model.as_str()) {
            if let Some(pricing) = MODEL_PRICING.get(matched_model) {
                debug!(
                    "Matched model '{}' to pricing for '{}'",
                    model.as_str(),
                    matched_model
                );
                return Ok(pricing.calculate_cost(tokens));
            }
        }

        // Try online pricing if enabled
        #[cfg(feature = "online-pricing")]
        if !self.offline {
            if let Some(client) = &self.client {
                match self.fetch_online_pricing(client, model).await {
                    Ok(pricing) => return Ok(pricing.calculate_cost(tokens)),
                    Err(e) => {
                        debug!(
                            "Failed to fetch online pricing for model {}: {}",
                            model.as_str(),
                            e
                        );
                    }
                }
            }
        }

        // Default to zero cost if model not found
        warn!("No pricing found for model: {}", model.as_str());
        Ok(Decimal::ZERO)
    }

    #[cfg(feature = "online-pricing")]
    async fn fetch_online_pricing(
        &self,
        client: &reqwest::Client,
        model: &ModelName,
    ) -> Result<ModelPricing> {
        // This would fetch from LiteLLM API or similar pricing service
        // For now, we'll just return an error to use offline pricing
        anyhow::bail!("Online pricing not yet implemented for model: {}", model.as_str())
    }

    /// Get pricing for a model (for display purposes)
    pub fn get_pricing(&self, model: &ModelName) -> Option<ModelPricing> {
        self.custom_pricing
            .get(model.as_str())
            .cloned()
            .or_else(|| MODEL_PRICING.get(model.as_str()).cloned())
    }

    /// List all known models with pricing
    pub fn list_models(&self) -> Vec<String> {
        let mut models: Vec<_> = MODEL_PRICING.keys().map(|s| s.to_string()).collect();
        models.extend(self.custom_pricing.keys().cloned());
        models.sort();
        models.dedup();
        models
    }
}