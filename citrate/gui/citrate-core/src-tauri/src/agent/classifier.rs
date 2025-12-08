//! Intent classification for user messages
//!
//! Two-tier classification:
//! 1. Fast regex/pattern matching for common intents
//! 2. LLM fallback for complex or ambiguous messages

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::config::ClassifierConfig;
use super::intent::{ClassificationSource, Intent, IntentMatch, IntentParams};

/// Error during classification
#[derive(Debug, Clone)]
pub struct ClassificationError(pub String);

impl std::fmt::Display for ClassificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Classification error: {}", self.0)
    }
}

impl std::error::Error for ClassificationError {}

/// Intent classifier with pattern matching and LLM fallback
pub struct IntentClassifier {
    config: ClassifierConfig,
    patterns: Vec<PatternRule>,
}

/// A pattern rule for intent matching
#[derive(Debug, Clone)]
struct PatternRule {
    intent: Intent,
    patterns: Vec<String>,
    param_extractors: Vec<ParamExtractor>,
}

/// Extracts parameters from matched text
#[derive(Debug, Clone)]
struct ParamExtractor {
    name: String,
    pattern: String,
}

impl IntentClassifier {
    /// Create a new classifier with default patterns
    pub fn new(config: ClassifierConfig) -> Self {
        let patterns = Self::default_patterns();
        Self { config, patterns }
    }

    /// Classify a user message
    pub async fn classify(&self, message: &str) -> Result<IntentMatch, ClassificationError> {
        let normalized = message.to_lowercase().trim().to_string();

        // Try pattern matching first
        if let Some(pattern_match) = self.match_patterns(&normalized) {
            if pattern_match.confidence >= self.config.pattern_confidence_threshold {
                return Ok(pattern_match);
            }
        }

        // Fallback to unknown intent
        // In the full implementation, this would call LLM for classification
        Ok(IntentMatch {
            intent: Intent::GeneralChat,
            confidence: 0.5,
            params: IntentParams::default(),
            source: ClassificationSource::Fallback,
            alternatives: Vec::new(),
        })
    }

    /// Match against pattern rules
    fn match_patterns(&self, message: &str) -> Option<IntentMatch> {
        let mut best_match: Option<(Intent, f32, IntentParams)> = None;

        for rule in &self.patterns {
            for pattern in &rule.patterns {
                if self.pattern_matches(message, pattern) {
                    let confidence = self.calculate_confidence(message, pattern);
                    let params = self.extract_params(message, &rule.param_extractors);

                    if best_match
                        .as_ref()
                        .map(|(_, c, _)| confidence > *c)
                        .unwrap_or(true)
                    {
                        best_match = Some((rule.intent.clone(), confidence, params));
                    }
                }
            }
        }

        best_match.map(|(intent, confidence, params)| IntentMatch {
            intent,
            confidence,
            params,
            source: ClassificationSource::Pattern,
            alternatives: Vec::new(),
        })
    }

    /// Check if message matches pattern
    fn pattern_matches(&self, message: &str, pattern: &str) -> bool {
        // Simple contains matching - could be replaced with regex
        let pattern_words: Vec<&str> = pattern.split_whitespace().collect();
        pattern_words.iter().all(|word| message.contains(word))
    }

    /// Calculate match confidence
    fn calculate_confidence(&self, message: &str, pattern: &str) -> f32 {
        let pattern_words: Vec<&str> = pattern.split_whitespace().collect();
        // Normalize message words by stripping common punctuation for comparison
        let message_words: Vec<String> = message
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation()))
            .filter(|w| !w.is_empty())
            .map(|w| w.to_string())
            .collect();

        let matched = pattern_words
            .iter()
            .filter(|&pw| message_words.iter().any(|mw| mw == pw))
            .count();

        matched as f32 / pattern_words.len().max(1) as f32
    }

    /// Extract parameters from message
    fn extract_params(&self, message: &str, _extractors: &[ParamExtractor]) -> IntentParams {
        let mut params = IntentParams::default();

        // Extract addresses (0x...)
        if let Some(addr) = Self::extract_address(message) {
            params.address = Some(addr);
        }

        // Extract amounts
        if let Some(amount) = Self::extract_amount(message) {
            params.amount = Some(amount);
        }

        // Extract block numbers/hashes
        if let Some(block) = Self::extract_block_ref(message) {
            params.block_ref = Some(block);
        }

        // Extract model names
        if let Some(model) = Self::extract_model_name(message) {
            params.model_name = Some(model);
        }

        params
    }

    /// Extract Ethereum address from message
    fn extract_address(message: &str) -> Option<String> {
        // Look for 0x followed by 40 hex chars
        let re = regex::Regex::new(r"0x[a-fA-F0-9]{40}").ok()?;
        re.find(message).map(|m| m.as_str().to_string())
    }

    /// Extract amount from message
    fn extract_amount(message: &str) -> Option<String> {
        // Look for numbers possibly followed by units
        let re = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*(?:SALT|salt|eth|ETH)?").ok()?;
        re.captures(message)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract block reference (number or hash)
    fn extract_block_ref(message: &str) -> Option<String> {
        // Look for block number or hash
        let hash_re = regex::Regex::new(r"0x[a-fA-F0-9]{64}").ok()?;
        if let Some(m) = hash_re.find(message) {
            return Some(m.as_str().to_string());
        }

        // Look for "block X" pattern
        let num_re = regex::Regex::new(r"block\s+#?(\d+)").ok()?;
        num_re
            .captures(message)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract model name from message
    fn extract_model_name(message: &str) -> Option<String> {
        // Common model name patterns
        let patterns = [
            r"(?i)(llama[- ]?\d*[bB]?)",
            r"(?i)(gpt[- ]?\d+(?:\.\d+)?(?:-turbo)?)",
            r"(?i)(claude[- ]?\d*(?:\.\d+)?(?:-sonnet|-opus|-haiku)?)",
            r"(?i)(mistral[- ]?\d*[bB]?)",
        ];

        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(m) = re.find(message) {
                    return Some(m.as_str().to_string());
                }
            }
        }
        None
    }

    /// Default pattern rules
    fn default_patterns() -> Vec<PatternRule> {
        vec![
            // Balance queries
            PatternRule {
                intent: Intent::QueryBalance,
                patterns: vec![
                    "balance".to_string(),
                    "how much".to_string(),
                    "check balance".to_string(),
                    "my balance".to_string(),
                    "wallet balance".to_string(),
                ],
                param_extractors: vec![],
            },
            // Send transactions
            PatternRule {
                intent: Intent::SendTransaction,
                patterns: vec![
                    "send".to_string(),
                    "transfer".to_string(),
                    "pay".to_string(),
                    "send to".to_string(),
                ],
                param_extractors: vec![],
            },
            // Transaction history
            PatternRule {
                intent: Intent::GetTransactionHistory,
                patterns: vec![
                    "transaction history".to_string(),
                    "my transactions".to_string(),
                    "tx history".to_string(),
                    "recent transactions".to_string(),
                ],
                param_extractors: vec![],
            },
            // Contract deployment
            PatternRule {
                intent: Intent::DeployContract,
                patterns: vec![
                    "deploy contract".to_string(),
                    "deploy".to_string(),
                    "publish contract".to_string(),
                ],
                param_extractors: vec![],
            },
            // Contract calls
            PatternRule {
                intent: Intent::CallContract,
                patterns: vec![
                    "call contract".to_string(),
                    "read contract".to_string(),
                    "contract call".to_string(),
                ],
                param_extractors: vec![],
            },
            // Write to contract
            PatternRule {
                intent: Intent::WriteContract,
                patterns: vec![
                    "write contract".to_string(),
                    "execute contract".to_string(),
                    "contract write".to_string(),
                ],
                param_extractors: vec![],
            },
            // Block info - requires explicit action words
            PatternRule {
                intent: Intent::GetBlockInfo,
                patterns: vec![
                    "show block info".to_string(),
                    "get block info".to_string(),
                    "show me block".to_string(),
                    "block details".to_string(),
                    "block number".to_string(),
                    "latest block".to_string(),
                ],
                param_extractors: vec![],
            },
            // DAG status - requires explicit requests
            PatternRule {
                intent: Intent::GetDAGStatus,
                patterns: vec![
                    "show dag status".to_string(),
                    "get dag status".to_string(),
                    "dag metrics".to_string(),
                    "dag tips".to_string(),
                    "ghostdag status".to_string(),
                    "show dag".to_string(),
                ],
                param_extractors: vec![],
            },
            // Node status - requires explicit status request
            PatternRule {
                intent: Intent::GetNodeStatus,
                patterns: vec![
                    "node status".to_string(),
                    "show node status".to_string(),
                    "connection status".to_string(),
                    "am i connected".to_string(),
                    "is node running".to_string(),
                ],
                param_extractors: vec![],
            },
            // List models
            PatternRule {
                intent: Intent::ListModels,
                patterns: vec![
                    "list models".to_string(),
                    "available models".to_string(),
                    "show models".to_string(),
                    "what models".to_string(),
                ],
                param_extractors: vec![],
            },
            // Run inference
            PatternRule {
                intent: Intent::RunInference,
                patterns: vec![
                    "run inference".to_string(),
                    "generate".to_string(),
                    "complete".to_string(),
                    "predict".to_string(),
                ],
                param_extractors: vec![],
            },
            // Deploy model
            PatternRule {
                intent: Intent::DeployModel,
                patterns: vec![
                    "deploy model".to_string(),
                    "upload model".to_string(),
                    "register model".to_string(),
                ],
                param_extractors: vec![],
            },
            // Help
            PatternRule {
                intent: Intent::Help,
                patterns: vec![
                    "help".to_string(),
                    "what can you do".to_string(),
                    "how do i".to_string(),
                    "commands".to_string(),
                ],
                param_extractors: vec![],
            },
        ]
    }

    /// Add custom pattern
    pub fn add_pattern(&mut self, intent: Intent, patterns: Vec<String>) {
        self.patterns.push(PatternRule {
            intent,
            patterns,
            param_extractors: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_balance_classification() {
        let classifier = IntentClassifier::new(ClassifierConfig::default());

        let result = classifier.classify("what's my balance?").await.unwrap();
        assert_eq!(result.intent, Intent::QueryBalance);
        assert_eq!(result.source, ClassificationSource::Pattern);
    }

    #[tokio::test]
    async fn test_send_classification() {
        let classifier = IntentClassifier::new(ClassifierConfig::default());

        let result = classifier
            .classify("send 10 SALT to 0x1234567890123456789012345678901234567890")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::SendTransaction);
        assert!(result.params.address.is_some());
        assert!(result.params.amount.is_some());
    }

    #[tokio::test]
    async fn test_address_extraction() {
        let addr = IntentClassifier::extract_address(
            "send to 0x1234567890123456789012345678901234567890",
        );
        assert_eq!(
            addr,
            Some("0x1234567890123456789012345678901234567890".to_string())
        );
    }

    #[tokio::test]
    async fn test_amount_extraction() {
        let amount = IntentClassifier::extract_amount("send 100 SALT");
        assert_eq!(amount, Some("100".to_string()));

        let amount = IntentClassifier::extract_amount("transfer 50.5 eth");
        assert_eq!(amount, Some("50.5".to_string()));
    }

    #[tokio::test]
    async fn test_fallback_to_chat() {
        let classifier = IntentClassifier::new(ClassifierConfig::default());

        let result = classifier
            .classify("tell me a joke about blockchains")
            .await
            .unwrap();
        assert_eq!(result.intent, Intent::GeneralChat);
        assert_eq!(result.source, ClassificationSource::Fallback);
    }
}
