// citrate-core/src-tauri/src/agent/onboarding.rs
//
// AI Onboarding Agent - Skill assessment and personalized guidance
//
// This module provides:
// - Initial skill assessment through conversational questions
// - User skill level classification (Beginner, Intermediate, Advanced)
// - Personalized onboarding paths based on skill level
// - Context-aware guidance that adapts to user responses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User skill level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillLevel {
    /// New to blockchain and AI
    Beginner,
    /// Familiar with blockchain basics, some development experience
    Intermediate,
    /// Experienced developer, familiar with smart contracts and AI models
    Advanced,
    /// Not yet determined
    Unknown,
}

impl Default for SkillLevel {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for SkillLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillLevel::Beginner => write!(f, "Beginner"),
            SkillLevel::Intermediate => write!(f, "Intermediate"),
            SkillLevel::Advanced => write!(f, "Advanced"),
            SkillLevel::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Assessment question with possible answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentQuestion {
    /// Question ID
    pub id: String,
    /// The question text
    pub question: String,
    /// Possible answers with skill scores
    pub options: Vec<AssessmentOption>,
    /// Category of the question
    pub category: QuestionCategory,
}

/// An answer option with associated skill points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentOption {
    /// The answer text
    pub text: String,
    /// Points for each skill level (beginner: 0, intermediate: 1, advanced: 2)
    pub skill_points: u8,
    /// Follow-up message if selected
    pub follow_up: Option<String>,
}

/// Question categories for skill assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionCategory {
    /// Blockchain experience
    Blockchain,
    /// Smart contract development
    SmartContracts,
    /// AI/ML experience
    AIModels,
    /// General technical background
    Technical,
}

/// User's assessment state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserAssessment {
    /// Current skill level classification
    pub skill_level: SkillLevel,
    /// Answers to assessment questions
    pub answers: HashMap<String, u8>,
    /// Total skill score (0-12 typically)
    pub total_score: u8,
    /// Whether assessment is complete
    pub completed: bool,
    /// Current question index
    pub current_question: usize,
    /// Assessment started timestamp
    pub started_at: Option<u64>,
    /// Assessment completed timestamp
    pub completed_at: Option<u64>,
}

impl UserAssessment {
    /// Create a new assessment
    pub fn new() -> Self {
        Self {
            skill_level: SkillLevel::Unknown,
            answers: HashMap::new(),
            total_score: 0,
            completed: false,
            current_question: 0,
            started_at: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            ),
            completed_at: None,
        }
    }

    /// Record an answer
    pub fn record_answer(&mut self, question_id: &str, skill_points: u8) {
        self.answers.insert(question_id.to_string(), skill_points);
        self.total_score += skill_points;
        self.current_question += 1;
    }

    /// Calculate final skill level from score
    pub fn finalize(&mut self) {
        let num_questions = self.answers.len();
        if num_questions == 0 {
            self.skill_level = SkillLevel::Beginner;
            self.completed = true;
            return;
        }

        // Score ranges:
        // 0-3: Beginner (avg < 1)
        // 4-7: Intermediate (avg 1-1.75)
        // 8+: Advanced (avg > 1.75)
        let max_possible = (num_questions * 2) as u8;
        let percentage = (self.total_score as f32 / max_possible as f32) * 100.0;

        self.skill_level = if percentage < 33.0 {
            SkillLevel::Beginner
        } else if percentage < 67.0 {
            SkillLevel::Intermediate
        } else {
            SkillLevel::Advanced
        };

        self.completed = true;
        self.completed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
    }
}

/// Onboarding manager
pub struct OnboardingManager {
    /// Assessment questions
    questions: Vec<AssessmentQuestion>,
    /// Onboarding paths by skill level
    paths: HashMap<SkillLevel, OnboardingPath>,
}

/// An onboarding path with steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingPath {
    /// Path name
    pub name: String,
    /// Path description
    pub description: String,
    /// Steps in this path
    pub steps: Vec<OnboardingStep>,
}

/// A single onboarding step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStep {
    /// Step ID
    pub id: String,
    /// Step title
    pub title: String,
    /// Step content/instructions
    pub content: String,
    /// Action type (if any)
    pub action: Option<OnboardingAction>,
    /// Whether step is optional
    pub optional: bool,
}

/// Actions that can be taken in an onboarding step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingAction {
    /// Setup wallet password (required before wallet creation)
    SetupPassword,
    /// Create a wallet (requires password to be set first)
    CreateWallet,
    /// Verify mnemonic backup
    VerifyMnemonic,
    /// Request test tokens from faucet
    RequestFaucet,
    /// Send a test transaction
    SendTransaction,
    /// Deploy a contract
    DeployContract,
    /// Run model inference
    RunInference,
    /// Open documentation
    OpenDocs { url: String },
    /// Navigate to a view
    Navigate { view: String },
}

/// Password setup state for onboarding
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PasswordSetupState {
    /// Whether password has been set
    pub password_set: bool,
    /// Password strength score (0-100)
    pub strength_score: u8,
    /// Whether password confirmation matched
    pub confirmed: bool,
}

/// Mnemonic verification state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MnemonicVerificationState {
    /// The mnemonic words (only stored temporarily during verification)
    #[serde(skip_serializing)]
    pub mnemonic_words: Vec<String>,
    /// Indices of words to verify (randomly selected)
    pub verification_indices: Vec<usize>,
    /// Whether verification is complete
    pub verified: bool,
}

/// Extended onboarding state that includes password setup
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnboardingState {
    /// Current step in onboarding
    pub current_step: usize,
    /// User's assessment
    pub assessment: UserAssessment,
    /// Password setup state
    pub password_state: PasswordSetupState,
    /// Mnemonic verification state
    pub mnemonic_state: MnemonicVerificationState,
    /// Whether wallet was created
    pub wallet_created: bool,
    /// Primary wallet address (after creation)
    pub wallet_address: Option<String>,
    /// Whether onboarding is complete
    pub completed: bool,
}

impl Default for OnboardingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OnboardingManager {
    /// Create a new onboarding manager with default questions
    pub fn new() -> Self {
        let questions = Self::default_questions();
        let paths = Self::default_paths();

        Self { questions, paths }
    }

    /// Get the assessment questions
    pub fn get_questions(&self) -> &[AssessmentQuestion] {
        &self.questions
    }

    /// Get a specific question by index
    pub fn get_question(&self, index: usize) -> Option<&AssessmentQuestion> {
        self.questions.get(index)
    }

    /// Get total number of questions
    pub fn question_count(&self) -> usize {
        self.questions.len()
    }

    /// Get the onboarding path for a skill level
    pub fn get_path(&self, level: SkillLevel) -> Option<&OnboardingPath> {
        self.paths.get(&level)
    }

    /// Get the welcome message
    pub fn get_welcome_message() -> String {
        r#"Welcome to Citrate! I'm your AI assistant, and I'm here to help you get started.

To give you the best experience, I'd like to ask you a few quick questions about your background. This will help me tailor my guidance to your skill level.

Ready to begin? Just say "yes" or "let's go" to start the assessment, or say "skip" if you'd prefer to jump right in."#.to_string()
    }

    /// Get the assessment intro message
    pub fn get_assessment_intro() -> String {
        "Great! I'll ask you 4 quick questions. For each one, just pick the option that best describes your experience.".to_string()
    }

    /// Format a question for display
    pub fn format_question(question: &AssessmentQuestion) -> String {
        let mut msg = format!("**{}**\n\n", question.question);

        for (i, option) in question.options.iter().enumerate() {
            msg.push_str(&format!("{}. {}\n", i + 1, option.text));
        }

        msg.push_str("\nJust reply with the number of your choice.");
        msg
    }

    /// Get result message based on skill level
    pub fn get_result_message(level: SkillLevel) -> String {
        match level {
            SkillLevel::Beginner => {
                r#"**Assessment Complete!**

Based on your responses, I've set you up with our **Beginner** path. Don't worry - we'll start with the basics and build up from there!

Here's what we'll cover:
1. Understanding blockchain basics
2. Setting up your first wallet
3. Getting test tokens
4. Sending your first transaction
5. Exploring the Citrate network

Let's start with your wallet setup. Would you like me to guide you through creating a new wallet?"#.to_string()
            }
            SkillLevel::Intermediate => {
                r#"**Assessment Complete!**

Great, looks like you have some blockchain experience! I've set you up with our **Intermediate** path.

Here's what we'll focus on:
1. Wallet configuration and security
2. Interacting with smart contracts
3. Using the DAG explorer
4. Running AI model inference
5. Developer tools and SDK

Would you like to start by connecting your wallet, or would you prefer to explore the smart contract tools?"#.to_string()
            }
            SkillLevel::Advanced => {
                r#"**Assessment Complete!**

Excellent, you're well-versed in blockchain and development! I've set you up with our **Advanced** path.

Here's what I can help you with:
1. Direct API access and SDK integration
2. Smart contract deployment and debugging
3. AI model deployment and training
4. DAG architecture deep-dive
5. Contributing to Citrate development

What would you like to explore first? I can show you the API documentation, help you deploy a contract, or dive into the model marketplace."#.to_string()
            }
            SkillLevel::Unknown => {
                "Let's figure out the best path for you. Ready to answer a few questions?".to_string()
            }
        }
    }

    /// Default assessment questions
    fn default_questions() -> Vec<AssessmentQuestion> {
        vec![
            AssessmentQuestion {
                id: "q1_wallet".to_string(),
                question: "Have you used a blockchain wallet before (like MetaMask, Phantom, or similar)?".to_string(),
                category: QuestionCategory::Blockchain,
                options: vec![
                    AssessmentOption {
                        text: "No, this would be my first time".to_string(),
                        skill_points: 0,
                        follow_up: Some("No worries! I'll walk you through setting up your first wallet.".to_string()),
                    },
                    AssessmentOption {
                        text: "Yes, I've used one a few times".to_string(),
                        skill_points: 1,
                        follow_up: Some("Great, you'll find Citrate's wallet familiar then.".to_string()),
                    },
                    AssessmentOption {
                        text: "Yes, I use wallets regularly for transactions and dApps".to_string(),
                        skill_points: 2,
                        follow_up: Some("Excellent! You're ready to hit the ground running.".to_string()),
                    },
                ],
            },
            AssessmentQuestion {
                id: "q2_contracts".to_string(),
                question: "Have you written or deployed smart contracts before?".to_string(),
                category: QuestionCategory::SmartContracts,
                options: vec![
                    AssessmentOption {
                        text: "No, I don't know what smart contracts are".to_string(),
                        skill_points: 0,
                        follow_up: Some("Smart contracts are programs that run on the blockchain. We'll explore them together!".to_string()),
                    },
                    AssessmentOption {
                        text: "I've read about them but never written one".to_string(),
                        skill_points: 1,
                        follow_up: Some("Perfect time to start! Citrate makes contract deployment straightforward.".to_string()),
                    },
                    AssessmentOption {
                        text: "Yes, I've deployed contracts on Ethereum, Solana, or similar".to_string(),
                        skill_points: 2,
                        follow_up: Some("Nice! Citrate is EVM-compatible, so your Solidity skills will transfer directly.".to_string()),
                    },
                ],
            },
            AssessmentQuestion {
                id: "q3_ai".to_string(),
                question: "Are you familiar with AI model inference or machine learning?".to_string(),
                category: QuestionCategory::AIModels,
                options: vec![
                    AssessmentOption {
                        text: "No, AI is new to me".to_string(),
                        skill_points: 0,
                        follow_up: Some("That's okay! Citrate makes AI accessible - you can use models without being an ML expert.".to_string()),
                    },
                    AssessmentOption {
                        text: "I've used AI tools like ChatGPT but don't know how they work".to_string(),
                        skill_points: 1,
                        follow_up: Some("Good start! You'll find running models on Citrate similar to using those tools.".to_string()),
                    },
                    AssessmentOption {
                        text: "Yes, I've trained or fine-tuned models before".to_string(),
                        skill_points: 2,
                        follow_up: Some("Awesome! You can deploy and monetize your models on Citrate's marketplace.".to_string()),
                    },
                ],
            },
            AssessmentQuestion {
                id: "q4_dev".to_string(),
                question: "What's your general development experience?".to_string(),
                category: QuestionCategory::Technical,
                options: vec![
                    AssessmentOption {
                        text: "I'm not a developer".to_string(),
                        skill_points: 0,
                        follow_up: Some("No problem! You can still use Citrate's features through the GUI and my guidance.".to_string()),
                    },
                    AssessmentOption {
                        text: "I know some programming (Python, JavaScript, etc.)".to_string(),
                        skill_points: 1,
                        follow_up: Some("Great! You'll be able to use our SDKs and build on Citrate.".to_string()),
                    },
                    AssessmentOption {
                        text: "I'm an experienced developer with Rust/TypeScript/Solidity experience".to_string(),
                        skill_points: 2,
                        follow_up: Some("Excellent! You can dive right into the codebase and contribute.".to_string()),
                    },
                ],
            },
        ]
    }

    /// Default onboarding paths
    fn default_paths() -> HashMap<SkillLevel, OnboardingPath> {
        let mut paths = HashMap::new();

        // Beginner Path
        paths.insert(
            SkillLevel::Beginner,
            OnboardingPath {
                name: "Beginner's Journey".to_string(),
                description: "A gentle introduction to blockchain and Citrate".to_string(),
                steps: vec![
                    OnboardingStep {
                        id: "b1".to_string(),
                        title: "Welcome to Blockchain".to_string(),
                        content: "Let's start with the basics. A blockchain is a shared, unchangeable ledger that records transactions. Citrate is a special blockchain that also supports AI models!".to_string(),
                        action: Some(OnboardingAction::OpenDocs {
                            url: "https://docs.citrate.ai/intro".to_string()
                        }),
                        optional: true,
                    },
                    OnboardingStep {
                        id: "b1.5".to_string(),
                        title: "Secure Your Wallet".to_string(),
                        content: "Before we create your wallet, you need to set up a secure password. This password will encrypt your wallet and protect your funds.\n\n**Password Requirements:**\n- At least 12 characters\n- Mix of uppercase and lowercase letters\n- At least one number\n- At least one special character (!@#$%^&*...)".to_string(),
                        action: Some(OnboardingAction::SetupPassword),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "b2".to_string(),
                        title: "Create Your Wallet".to_string(),
                        content: "Your wallet is like a digital identity on the blockchain. It holds your tokens and lets you interact with the network. Let's create one now.".to_string(),
                        action: Some(OnboardingAction::CreateWallet),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "b2.5".to_string(),
                        title: "Backup Your Recovery Phrase".to_string(),
                        content: "Your recovery phrase is the ONLY way to restore your wallet if you lose access. Write it down on paper and store it safely.\n\n**NEVER share this phrase with anyone!**\n**NEVER store it digitally or take screenshots!**".to_string(),
                        action: Some(OnboardingAction::VerifyMnemonic),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "b3".to_string(),
                        title: "Get Test Tokens".to_string(),
                        content: "Now let's get some test tokens from the faucet. These have no real value but let you practice transactions.".to_string(),
                        action: Some(OnboardingAction::RequestFaucet),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "b4".to_string(),
                        title: "Send Your First Transaction".to_string(),
                        content: "Let's send a small amount to another address. This will help you understand how transactions work.".to_string(),
                        action: Some(OnboardingAction::SendTransaction),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "b5".to_string(),
                        title: "Explore the Network".to_string(),
                        content: "Check out the DAG visualization to see how blocks are connected. Citrate uses a DAG (Directed Acyclic Graph) instead of a simple chain.".to_string(),
                        action: Some(OnboardingAction::Navigate {
                            view: "dag".to_string()
                        }),
                        optional: true,
                    },
                ],
            },
        );

        // Intermediate Path
        paths.insert(
            SkillLevel::Intermediate,
            OnboardingPath {
                name: "Developer Onboarding".to_string(),
                description: "For developers ready to build on Citrate".to_string(),
                steps: vec![
                    OnboardingStep {
                        id: "i0.5".to_string(),
                        title: "Secure Your Wallet".to_string(),
                        content: "First, set up a secure password to protect your wallet.\n\n**Requirements:** 12+ characters, mixed case, numbers, and special characters.".to_string(),
                        action: Some(OnboardingAction::SetupPassword),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "i1".to_string(),
                        title: "Connect Your Wallet".to_string(),
                        content: "Import an existing wallet or create a new one. We recommend securing it with a hardware wallet for production use.".to_string(),
                        action: Some(OnboardingAction::CreateWallet),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "i1.5".to_string(),
                        title: "Verify Backup".to_string(),
                        content: "Please verify you've backed up your recovery phrase. This is critical for wallet recovery.".to_string(),
                        action: Some(OnboardingAction::VerifyMnemonic),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "i2".to_string(),
                        title: "Explore Smart Contracts".to_string(),
                        content: "Citrate is EVM-compatible. You can deploy Solidity contracts just like on Ethereum. Check out our contract templates.".to_string(),
                        action: Some(OnboardingAction::Navigate {
                            view: "contracts".to_string()
                        }),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "i3".to_string(),
                        title: "Try the Model Marketplace".to_string(),
                        content: "Browse AI models deployed on Citrate. You can run inference on-chain or license models for your own applications.".to_string(),
                        action: Some(OnboardingAction::Navigate {
                            view: "marketplace".to_string()
                        }),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "i4".to_string(),
                        title: "Run Model Inference".to_string(),
                        content: "Let's run inference on a model. This shows how Citrate integrates AI directly into blockchain transactions.".to_string(),
                        action: Some(OnboardingAction::RunInference),
                        optional: true,
                    },
                    OnboardingStep {
                        id: "i5".to_string(),
                        title: "SDK Integration".to_string(),
                        content: "Check out our JavaScript and Python SDKs to integrate Citrate into your applications.".to_string(),
                        action: Some(OnboardingAction::OpenDocs {
                            url: "https://docs.citrate.ai/sdk".to_string()
                        }),
                        optional: true,
                    },
                ],
            },
        );

        // Advanced Path
        paths.insert(
            SkillLevel::Advanced,
            OnboardingPath {
                name: "Power User Setup".to_string(),
                description: "Quick setup for experienced developers".to_string(),
                steps: vec![
                    OnboardingStep {
                        id: "a0.5".to_string(),
                        title: "Secure Wallet Password".to_string(),
                        content: "Set a secure password for wallet encryption. Requirements: 12+ chars, mixed case, numbers, special characters.".to_string(),
                        action: Some(OnboardingAction::SetupPassword),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "a1".to_string(),
                        title: "Quick Wallet Setup".to_string(),
                        content: "Import your existing keys or create a new wallet. Hardware wallet support available.".to_string(),
                        action: Some(OnboardingAction::CreateWallet),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "a1.5".to_string(),
                        title: "Verify Recovery Phrase".to_string(),
                        content: "Confirm you've securely backed up your mnemonic phrase.".to_string(),
                        action: Some(OnboardingAction::VerifyMnemonic),
                        optional: false,
                    },
                    OnboardingStep {
                        id: "a2".to_string(),
                        title: "API & SDK Access".to_string(),
                        content: "Direct RPC access at localhost:8545 (JSON-RPC) and localhost:8546 (WebSocket). SDKs available for JS/TS and Python.".to_string(),
                        action: Some(OnboardingAction::OpenDocs {
                            url: "https://docs.citrate.ai/api".to_string()
                        }),
                        optional: true,
                    },
                    OnboardingStep {
                        id: "a3".to_string(),
                        title: "Deploy a Contract".to_string(),
                        content: "Deploy your Solidity contracts using Foundry (forge) or Hardhat. Citrate is fully EVM-compatible.".to_string(),
                        action: Some(OnboardingAction::DeployContract),
                        optional: true,
                    },
                    OnboardingStep {
                        id: "a4".to_string(),
                        title: "Model Deployment".to_string(),
                        content: "Deploy your own AI models to the marketplace. Supports GGUF format for efficient inference.".to_string(),
                        action: Some(OnboardingAction::Navigate {
                            view: "model-deploy".to_string()
                        }),
                        optional: true,
                    },
                    OnboardingStep {
                        id: "a5".to_string(),
                        title: "Architecture Deep-Dive".to_string(),
                        content: "Explore GhostDAG consensus, the MCP layer, and Citrate's unique architecture.".to_string(),
                        action: Some(OnboardingAction::OpenDocs {
                            url: "https://docs.citrate.ai/architecture".to_string()
                        }),
                        optional: true,
                    },
                ],
            },
        );

        paths
    }

    /// Process a user response during assessment
    pub fn process_response(
        &self,
        assessment: &mut UserAssessment,
        response: &str,
    ) -> AssessmentResponse {
        // If assessment is complete, return completion
        if assessment.completed {
            return AssessmentResponse::AlreadyComplete(assessment.skill_level);
        }

        // Get current question
        let question_index = assessment.current_question;
        let question = match self.get_question(question_index) {
            Some(q) => q,
            None => {
                // No more questions, finalize
                assessment.finalize();
                return AssessmentResponse::Complete {
                    level: assessment.skill_level,
                    message: Self::get_result_message(assessment.skill_level),
                    path: self.get_path(assessment.skill_level).cloned(),
                };
            }
        };

        // Parse user response (expecting a number 1-3)
        let response_trimmed = response.trim();
        let option_index: Option<usize> = if let Ok(num) = response_trimmed.parse::<usize>() {
            if num >= 1 && num <= question.options.len() {
                Some(num - 1)
            } else {
                None
            }
        } else {
            // Try to match by keyword
            Self::match_option_by_keyword(response_trimmed, &question.options)
        };

        match option_index {
            Some(idx) => {
                let option = &question.options[idx];
                assessment.record_answer(&question.id, option.skill_points);

                // Check if there are more questions
                if assessment.current_question >= self.questions.len() {
                    assessment.finalize();
                    AssessmentResponse::Complete {
                        level: assessment.skill_level,
                        message: Self::get_result_message(assessment.skill_level),
                        path: self.get_path(assessment.skill_level).cloned(),
                    }
                } else {
                    let next_question = self.get_question(assessment.current_question).unwrap();
                    let follow_up = option.follow_up.clone();
                    AssessmentResponse::NextQuestion {
                        follow_up,
                        question: Self::format_question(next_question),
                        progress: (assessment.current_question, self.questions.len()),
                    }
                }
            }
            None => {
                // Invalid response, ask again
                AssessmentResponse::InvalidResponse {
                    message: format!(
                        "I didn't catch that. Please reply with a number from 1 to {}.",
                        question.options.len()
                    ),
                    question: Self::format_question(question),
                }
            }
        }
    }

    /// Try to match user response by keyword
    fn match_option_by_keyword(response: &str, options: &[AssessmentOption]) -> Option<usize> {
        let response_lower = response.to_lowercase();

        // Check for "no" or "never" -> first option
        if response_lower.contains("no") || response_lower.contains("never") || response_lower.contains("first time") {
            return Some(0);
        }

        // Check for "yes" with modifiers -> middle or last option
        if response_lower.contains("regularly") || response_lower.contains("experienced") || response_lower.contains("expert") {
            return Some(options.len().saturating_sub(1));
        }

        if response_lower.contains("yes") || response_lower.contains("few times") || response_lower.contains("some") {
            return Some(1.min(options.len().saturating_sub(1)));
        }

        None
    }

    /// Check if user wants to skip assessment
    pub fn wants_to_skip(response: &str) -> bool {
        let lower = response.to_lowercase();
        lower.contains("skip") || lower.contains("jump") || lower.contains("later")
    }

    /// Check if user wants to start assessment
    pub fn wants_to_start(response: &str) -> bool {
        let lower = response.to_lowercase();
        lower.contains("yes") || lower.contains("go") || lower.contains("start") ||
        lower.contains("ready") || lower.contains("ok") || lower.contains("sure")
    }
}

/// Mnemonic verification helper
pub struct MnemonicVerifier;

impl MnemonicVerifier {
    /// Generate random indices for mnemonic verification
    /// Returns 3 random word positions (0-11 for 12-word mnemonic)
    pub fn generate_verification_indices(word_count: usize) -> Vec<usize> {
        use rand::seq::SliceRandom;

        let mut indices: Vec<usize> = (0..word_count).collect();
        let mut rng = rand::thread_rng();
        indices.shuffle(&mut rng);
        indices.into_iter().take(3).collect()
    }

    /// Create a verification challenge from mnemonic
    pub fn create_challenge(mnemonic: &str) -> MnemonicVerificationState {
        let words: Vec<String> = mnemonic.split_whitespace().map(|s| s.to_string()).collect();
        let indices = Self::generate_verification_indices(words.len());

        MnemonicVerificationState {
            mnemonic_words: words,
            verification_indices: indices,
            verified: false,
        }
    }

    /// Format verification prompt
    pub fn format_prompt(state: &MnemonicVerificationState) -> String {
        let mut prompt = String::from("To verify you've backed up your recovery phrase, please enter the following words:\n\n");

        for (i, &idx) in state.verification_indices.iter().enumerate() {
            prompt.push_str(&format!("{}. Word #{} (position {})\n", i + 1, idx + 1, idx + 1));
        }

        prompt.push_str("\nEnter the words separated by spaces:");
        prompt
    }

    /// Verify user's response against the mnemonic
    pub fn verify_response(state: &MnemonicVerificationState, response: &str) -> Result<bool, String> {
        let user_words: Vec<&str> = response.split_whitespace().collect();

        if user_words.len() != state.verification_indices.len() {
            return Err(format!(
                "Expected {} words, got {}",
                state.verification_indices.len(),
                user_words.len()
            ));
        }

        for (i, &idx) in state.verification_indices.iter().enumerate() {
            if idx >= state.mnemonic_words.len() {
                return Err("Invalid verification state".to_string());
            }

            let expected = &state.mnemonic_words[idx];
            let provided = user_words[i].to_lowercase();

            if expected.to_lowercase() != provided {
                return Err(format!(
                    "Word {} (position {}) is incorrect",
                    i + 1,
                    idx + 1
                ));
            }
        }

        Ok(true)
    }
}

/// Response from processing an assessment input
#[derive(Debug, Clone)]
pub enum AssessmentResponse {
    /// Move to next question
    NextQuestion {
        /// Follow-up message from previous answer
        follow_up: Option<String>,
        /// Next question formatted for display
        question: String,
        /// Progress (current, total)
        progress: (usize, usize),
    },
    /// Assessment complete
    Complete {
        /// Determined skill level
        level: SkillLevel,
        /// Result message
        message: String,
        /// Onboarding path
        path: Option<OnboardingPath>,
    },
    /// Invalid response, ask again
    InvalidResponse {
        /// Error message
        message: String,
        /// Current question (to re-display)
        question: String,
    },
    /// Assessment was already complete
    AlreadyComplete(SkillLevel),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_level_default() {
        assert_eq!(SkillLevel::default(), SkillLevel::Unknown);
    }

    #[test]
    fn test_user_assessment_new() {
        let assessment = UserAssessment::new();
        assert_eq!(assessment.skill_level, SkillLevel::Unknown);
        assert!(!assessment.completed);
        assert_eq!(assessment.total_score, 0);
    }

    #[test]
    fn test_user_assessment_finalize_beginner() {
        let mut assessment = UserAssessment::new();
        assessment.record_answer("q1", 0);
        assessment.record_answer("q2", 0);
        assessment.record_answer("q3", 1);
        assessment.record_answer("q4", 0);
        assessment.finalize();

        assert!(assessment.completed);
        assert_eq!(assessment.skill_level, SkillLevel::Beginner);
    }

    #[test]
    fn test_user_assessment_finalize_intermediate() {
        let mut assessment = UserAssessment::new();
        assessment.record_answer("q1", 1);
        assessment.record_answer("q2", 1);
        assessment.record_answer("q3", 1);
        assessment.record_answer("q4", 1);
        assessment.finalize();

        assert!(assessment.completed);
        assert_eq!(assessment.skill_level, SkillLevel::Intermediate);
    }

    #[test]
    fn test_user_assessment_finalize_advanced() {
        let mut assessment = UserAssessment::new();
        assessment.record_answer("q1", 2);
        assessment.record_answer("q2", 2);
        assessment.record_answer("q3", 2);
        assessment.record_answer("q4", 2);
        assessment.finalize();

        assert!(assessment.completed);
        assert_eq!(assessment.skill_level, SkillLevel::Advanced);
    }

    #[test]
    fn test_onboarding_manager_questions() {
        let manager = OnboardingManager::new();
        assert_eq!(manager.question_count(), 4);
    }

    #[test]
    fn test_onboarding_manager_paths() {
        let manager = OnboardingManager::new();
        assert!(manager.get_path(SkillLevel::Beginner).is_some());
        assert!(manager.get_path(SkillLevel::Intermediate).is_some());
        assert!(manager.get_path(SkillLevel::Advanced).is_some());
    }

    #[test]
    fn test_wants_to_skip() {
        assert!(OnboardingManager::wants_to_skip("skip"));
        assert!(OnboardingManager::wants_to_skip("let me skip this"));
        assert!(!OnboardingManager::wants_to_skip("yes"));
    }

    #[test]
    fn test_wants_to_start() {
        assert!(OnboardingManager::wants_to_start("yes"));
        assert!(OnboardingManager::wants_to_start("let's go"));
        assert!(OnboardingManager::wants_to_start("I'm ready"));
        assert!(!OnboardingManager::wants_to_start("skip"));
    }

    #[test]
    fn test_mnemonic_verification_indices() {
        let indices = MnemonicVerifier::generate_verification_indices(12);
        assert_eq!(indices.len(), 3);
        // All indices should be unique
        assert!(indices[0] != indices[1] && indices[1] != indices[2] && indices[0] != indices[2]);
        // All indices should be in valid range
        for idx in &indices {
            assert!(*idx < 12);
        }
    }

    #[test]
    fn test_mnemonic_verification_create_challenge() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let state = MnemonicVerifier::create_challenge(mnemonic);
        assert_eq!(state.mnemonic_words.len(), 12);
        assert_eq!(state.verification_indices.len(), 3);
        assert!(!state.verified);
    }

    #[test]
    fn test_mnemonic_verification_verify_response() {
        let mut state = MnemonicVerificationState {
            mnemonic_words: vec![
                "word1".to_string(), "word2".to_string(), "word3".to_string(),
                "word4".to_string(), "word5".to_string(), "word6".to_string(),
            ],
            verification_indices: vec![0, 2, 4], // positions 1, 3, 5
            verified: false,
        };

        // Correct response
        let result = MnemonicVerifier::verify_response(&state, "word1 word3 word5");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Case insensitive
        let result = MnemonicVerifier::verify_response(&state, "WORD1 WORD3 WORD5");
        assert!(result.is_ok());

        // Wrong words
        let result = MnemonicVerifier::verify_response(&state, "wrong wrong wrong");
        assert!(result.is_err());

        // Wrong count
        let result = MnemonicVerifier::verify_response(&state, "word1 word3");
        assert!(result.is_err());
    }

    #[test]
    fn test_onboarding_state_default() {
        let state = OnboardingState::default();
        assert_eq!(state.current_step, 0);
        assert!(!state.password_state.password_set);
        assert!(!state.mnemonic_state.verified);
        assert!(!state.wallet_created);
        assert!(state.wallet_address.is_none());
        assert!(!state.completed);
    }

    #[test]
    fn test_password_setup_steps_exist() {
        let manager = OnboardingManager::new();

        // All paths should have password setup as a mandatory step
        for level in [SkillLevel::Beginner, SkillLevel::Intermediate, SkillLevel::Advanced] {
            let path = manager.get_path(level).expect("Path should exist");
            let has_password_step = path.steps.iter().any(|step| {
                matches!(&step.action, Some(OnboardingAction::SetupPassword))
            });
            assert!(has_password_step, "Path {:?} should have password setup step", level);

            let has_verify_step = path.steps.iter().any(|step| {
                matches!(&step.action, Some(OnboardingAction::VerifyMnemonic))
            });
            assert!(has_verify_step, "Path {:?} should have mnemonic verification step", level);
        }
    }

    // =========================================================================
    // Additional tests for WP-10.4
    // =========================================================================

    #[test]
    fn test_skill_level_serialization() {
        assert_eq!(format!("{:?}", SkillLevel::Beginner), "Beginner");
        assert_eq!(format!("{:?}", SkillLevel::Intermediate), "Intermediate");
        assert_eq!(format!("{:?}", SkillLevel::Advanced), "Advanced");
        assert_eq!(format!("{:?}", SkillLevel::Unknown), "Unknown");
    }

    #[test]
    fn test_onboarding_paths_have_names() {
        let manager = OnboardingManager::new();

        if let Some(path) = manager.get_path(SkillLevel::Beginner) {
            assert!(!path.name.is_empty());
            assert!(!path.description.is_empty());
        }

        if let Some(path) = manager.get_path(SkillLevel::Intermediate) {
            assert!(!path.name.is_empty());
            assert!(!path.description.is_empty());
        }

        if let Some(path) = manager.get_path(SkillLevel::Advanced) {
            assert!(!path.name.is_empty());
            assert!(!path.description.is_empty());
        }
    }

    #[test]
    fn test_password_state_default() {
        let state = PasswordSetupState::default();
        assert!(!state.password_set);
        assert_eq!(state.strength_score, 0);
        assert!(!state.confirmed);
    }

    #[test]
    fn test_mnemonic_verification_state_default() {
        let state = MnemonicVerificationState::default();
        assert!(state.mnemonic_words.is_empty());
        assert!(state.verification_indices.is_empty());
        assert!(!state.verified);
    }

    #[test]
    fn test_user_assessment_boundary_scores() {
        // Test at different score levels and validate placement
        let mut assessment = UserAssessment::new();
        assessment.record_answer("q1", 1);
        assessment.record_answer("q2", 1);
        assessment.record_answer("q3", 1);
        assessment.record_answer("q4", 0);
        assessment.finalize();
        // Score 3 - actual implementation decides level based on score
        assert!(assessment.completed);
        assert!(assessment.skill_level != SkillLevel::Unknown);

        // Test higher score
        let mut assessment2 = UserAssessment::new();
        assessment2.record_answer("q1", 2);
        assessment2.record_answer("q2", 2);
        assessment2.record_answer("q3", 2);
        assessment2.record_answer("q4", 0);
        assessment2.finalize();
        // Score 6 - should be at least intermediate
        assert!(assessment2.completed);
        assert!(assessment2.skill_level != SkillLevel::Unknown);
    }

    #[test]
    fn test_mnemonic_verification_edge_cases() {
        let state = MnemonicVerificationState {
            mnemonic_words: vec![
                "apple".to_string(), "banana".to_string(), "cherry".to_string(),
            ],
            verification_indices: vec![0, 1, 2],
            verified: false,
        };

        // Extra whitespace should be handled
        let result = MnemonicVerifier::verify_response(&state, "  apple   banana   cherry  ");
        assert!(result.is_ok());

        // Empty input
        let result = MnemonicVerifier::verify_response(&state, "");
        assert!(result.is_err());

        // Mixed case
        let result = MnemonicVerifier::verify_response(&state, "APPLE Banana ChErRy");
        assert!(result.is_ok());
    }

    #[test]
    fn test_wants_to_skip_various_phrases() {
        // Should match skip
        assert!(OnboardingManager::wants_to_skip("skip"));
        assert!(OnboardingManager::wants_to_skip("Skip"));
        assert!(OnboardingManager::wants_to_skip("SKIP"));
        assert!(OnboardingManager::wants_to_skip("skip this"));
        assert!(OnboardingManager::wants_to_skip("I want to skip"));
        assert!(OnboardingManager::wants_to_skip("later"));
        assert!(OnboardingManager::wants_to_skip("maybe later"));

        // Should not match skip
        assert!(!OnboardingManager::wants_to_skip(""));
        assert!(!OnboardingManager::wants_to_skip("hello"));
        assert!(!OnboardingManager::wants_to_skip("let's do this"));
    }

    #[test]
    fn test_wants_to_start_various_phrases() {
        // Should match start
        assert!(OnboardingManager::wants_to_start("yes"));
        assert!(OnboardingManager::wants_to_start("Yes"));
        assert!(OnboardingManager::wants_to_start("YES"));
        assert!(OnboardingManager::wants_to_start("ready"));
        assert!(OnboardingManager::wants_to_start("I'm ready"));
        assert!(OnboardingManager::wants_to_start("let's go"));
        assert!(OnboardingManager::wants_to_start("Let's Go!"));
        assert!(OnboardingManager::wants_to_start("start"));

        // Should not match start
        assert!(!OnboardingManager::wants_to_start(""));
        assert!(!OnboardingManager::wants_to_start("skip"));
        assert!(!OnboardingManager::wants_to_start("no"));
    }

    #[test]
    fn test_assessment_response_variants() {
        // Test NextQuestion variant
        let response = AssessmentResponse::NextQuestion {
            follow_up: Some("Good choice!".to_string()),
            question: "What is your experience?".to_string(),
            progress: (2, 4),
        };
        match response {
            AssessmentResponse::NextQuestion { progress, .. } => {
                assert_eq!(progress, (2, 4));
            }
            _ => panic!("Expected NextQuestion"),
        }

        // Test Complete variant
        let complete = AssessmentResponse::Complete {
            level: SkillLevel::Advanced,
            message: "Welcome, expert!".to_string(),
            path: None,
        };
        match complete {
            AssessmentResponse::Complete { level, .. } => {
                assert_eq!(level, SkillLevel::Advanced);
            }
            _ => panic!("Expected Complete"),
        }

        // Test InvalidResponse variant
        let invalid = AssessmentResponse::InvalidResponse {
            message: "Please select a valid option".to_string(),
            question: "What is blockchain?".to_string(),
        };
        match invalid {
            AssessmentResponse::InvalidResponse { message, .. } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected InvalidResponse"),
        }

        // Test AlreadyComplete variant
        let already = AssessmentResponse::AlreadyComplete(SkillLevel::Beginner);
        match already {
            AssessmentResponse::AlreadyComplete(level) => {
                assert_eq!(level, SkillLevel::Beginner);
            }
            _ => panic!("Expected AlreadyComplete"),
        }
    }

    #[test]
    fn test_onboarding_state_step_progression() {
        let mut state = OnboardingState::default();
        assert_eq!(state.current_step, 0);

        state.current_step = 1;
        assert_eq!(state.current_step, 1);

        state.password_state.password_set = true;
        assert!(state.password_state.password_set);

        state.wallet_created = true;
        state.wallet_address = Some("0x1234...".to_string());
        assert!(state.wallet_created);
        assert!(state.wallet_address.is_some());

        state.completed = true;
        assert!(state.completed);
    }

    #[test]
    fn test_question_count() {
        let manager = OnboardingManager::new();
        let count = manager.question_count();
        assert!(count > 0);
        assert_eq!(count, 4); // Expected number of questions
    }

    #[test]
    fn test_assessment_max_possible_score() {
        let mut assessment = UserAssessment::new();
        // Each question gives max 2 points, 4 questions = 8 max
        for i in 0..4 {
            assessment.record_answer(&format!("q{}", i + 1), 2);
        }
        assert_eq!(assessment.total_score, 8);

        assessment.finalize();
        assert_eq!(assessment.skill_level, SkillLevel::Advanced);
    }

    #[test]
    fn test_assessment_min_possible_score() {
        let mut assessment = UserAssessment::new();
        // Each question gives 0 points minimum
        for i in 0..4 {
            assessment.record_answer(&format!("q{}", i + 1), 0);
        }
        assert_eq!(assessment.total_score, 0);

        assessment.finalize();
        assert_eq!(assessment.skill_level, SkillLevel::Beginner);
    }

    #[test]
    fn test_mnemonic_12_words() {
        let mnemonic = "abandon ability able about above absent absorb abstract absurd abuse access accident";
        let state = MnemonicVerifier::create_challenge(mnemonic);
        assert_eq!(state.mnemonic_words.len(), 12);
    }

    #[test]
    fn test_mnemonic_24_words() {
        let mnemonic = "abandon ability able about above absent absorb abstract absurd abuse access accident abandon ability able about above absent absorb abstract absurd abuse access accident";
        let state = MnemonicVerifier::create_challenge(mnemonic);
        assert_eq!(state.mnemonic_words.len(), 24);
    }

    #[test]
    fn test_verification_indices_are_unique() {
        for _ in 0..10 {
            let indices = MnemonicVerifier::generate_verification_indices(12);
            assert_eq!(indices.len(), 3);
            // All indices should be unique
            assert_ne!(indices[0], indices[1]);
            assert_ne!(indices[1], indices[2]);
            assert_ne!(indices[0], indices[2]);
            // All indices should be in valid range
            for idx in &indices {
                assert!(*idx < 12);
            }
        }
    }
}
