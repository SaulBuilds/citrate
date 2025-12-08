// OnboardingModal.tsx
//
// AI-first onboarding experience for Citrate
// Handles: wallet setup, bundled model setup, skill assessment, personalized guidance
// Enhanced with password setup, mnemonic verification, network selection, and HF connection

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Sparkles,
  Brain,
  Rocket,
  CheckCircle,
  ChevronRight,
  ChevronLeft,
  Settings,
  Bot,
  Zap,
  MessageSquare,
  X,
  Lock,
  Key,
  Shield,
  AlertTriangle,
  Eye,
  EyeOff,
  Copy,
  RefreshCw,
  Globe,
  Wifi,
  Download,
  ExternalLink,
  Loader2
} from 'lucide-react';

// ============================================================================
// Types
// ============================================================================

interface OnboardingQuestion {
  id: string;
  question: string;
  options: {
    text: string;
    score: number;
  }[];
}

interface OnboardingState {
  is_first_run: boolean;
  has_bundled_model: boolean;
  bundled_model_ready: boolean;
  onboarding_completed: boolean;
  has_any_provider: boolean;
  has_local_model?: boolean;
  has_cloud_provider?: boolean;
}

interface SkillAssessmentResult {
  skill_level: 'beginner' | 'intermediate' | 'advanced';
  recommended_path: string[];
  personalized_greeting: string;
}

interface WalletSetupResult {
  primary_address: string;
  mnemonic: string;
}

interface PasswordStrength {
  score: number;
  feedback: string[];
  is_valid: boolean;
}

interface OnboardingModalProps {
  onComplete?: () => void;
}

type OnboardingStep =
  | 'welcome'
  | 'network-select'
  | 'password-setup'
  | 'wallet-create'
  | 'mnemonic-backup'
  | 'mnemonic-verify'
  | 'model-setup'
  | 'huggingface-connect'
  | 'assessment'
  | 'results'
  | 'complete';

// ============================================================================
// Helper Functions
// ============================================================================

const validatePasswordStrength = (password: string): PasswordStrength => {
  const feedback: string[] = [];
  let score = 0;

  if (password.length >= 12) {
    score += 25;
  } else {
    feedback.push('At least 12 characters required');
  }

  if (/[a-z]/.test(password) && /[A-Z]/.test(password)) {
    score += 25;
  } else {
    feedback.push('Mix of uppercase and lowercase letters');
  }

  if (/\d/.test(password)) {
    score += 25;
  } else {
    feedback.push('At least one number');
  }

  if (/[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(password)) {
    score += 25;
  } else {
    feedback.push('At least one special character');
  }

  return {
    score,
    feedback,
    is_valid: score >= 100
  };
};

const getPasswordStrengthColor = (score: number): string => {
  if (score < 50) return 'bg-red-500';
  if (score < 75) return 'bg-yellow-500';
  if (score < 100) return 'bg-orange-500';
  return 'bg-green-500';
};

const getSkillLevelTitle = (level: string): string => {
  switch (level) {
    case 'beginner':
      return 'Welcome, Explorer!';
    case 'intermediate':
      return 'Welcome, Builder!';
    case 'advanced':
      return 'Welcome, Expert!';
    default:
      return 'Welcome to Citrate!';
  }
};

// ============================================================================
// Subcomponents
// ============================================================================

const FeatureItem: React.FC<{ icon: React.ReactNode; title: string; description: string }> = ({
  icon,
  title,
  description
}) => (
  <div className="flex items-start space-x-3 p-3 bg-gray-50 rounded-lg">
    <div className="w-8 h-8 bg-orange-100 rounded-lg flex items-center justify-center flex-shrink-0 text-orange-600">
      {icon}
    </div>
    <div>
      <h4 className="font-medium text-gray-900">{title}</h4>
      <p className="text-sm text-gray-600">{description}</p>
    </div>
  </div>
);

const StepIndicator: React.FC<{
  currentStep: number;
  totalSteps: number;
  stepLabels?: string[];
}> = ({ currentStep, totalSteps, stepLabels }) => (
  <div className="flex space-x-1 mb-6">
    {Array.from({ length: totalSteps }).map((_, idx) => (
      <div
        key={idx}
        className={`flex-1 h-1 rounded-full transition-colors ${
          idx < currentStep
            ? 'bg-orange-600'
            : idx === currentStep
            ? 'bg-orange-400'
            : 'bg-gray-200'
        }`}
        title={stepLabels?.[idx]}
      />
    ))}
  </div>
);

// ============================================================================
// Main Component
// ============================================================================

export const OnboardingModal: React.FC<OnboardingModalProps> = ({ onComplete }) => {
  // Visibility and navigation
  const [isVisible, setIsVisible] = useState(false);
  const [step, setStep] = useState<OnboardingStep>('welcome');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Onboarding state from backend
  const [onboardingState, setOnboardingState] = useState<OnboardingState | null>(null);

  // Network selection
  const [selectedNetwork, setSelectedNetwork] = useState<'devnet' | 'testnet' | 'mainnet'>('devnet');

  // Password setup
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [passwordStrength, setPasswordStrength] = useState<PasswordStrength>({ score: 0, feedback: [], is_valid: false });

  // Wallet setup
  const [walletResult, setWalletResult] = useState<WalletSetupResult | null>(null);
  const [mnemonicCopied, setMnemonicCopied] = useState(false);
  const [mnemonicConfirmed, setMnemonicConfirmed] = useState(false);

  // Mnemonic verification
  const [verificationWords, setVerificationWords] = useState<{ index: number; word: string }[]>([]);
  const [verificationInputs, setVerificationInputs] = useState<string[]>(['', '', '']);
  const [verificationError, setVerificationError] = useState<string | null>(null);

  // Model setup
  const [modelSetupStatus, setModelSetupStatus] = useState<'pending' | 'setting-up' | 'ready' | 'error' | 'not-found'>('pending');
  const [modelInfo, setModelInfo] = useState<{ name?: string; size_mb?: number; path?: string } | null>(null);

  // HuggingFace connection
  const [hfConnected, setHfConnected] = useState(false);
  const [skipHf, setSkipHf] = useState(false);

  // Assessment
  const [questions, setQuestions] = useState<OnboardingQuestion[]>([]);
  const [currentQuestionIndex, setCurrentQuestionIndex] = useState(0);
  const [answers, setAnswers] = useState<number[]>([]);
  const [assessmentResult, setAssessmentResult] = useState<SkillAssessmentResult | null>(null);

  // ============================================================================
  // Effects
  // ============================================================================

  // Check if we should show onboarding
  useEffect(() => {
    checkOnboardingStatus();
  }, []);

  // Update password strength on password change
  useEffect(() => {
    setPasswordStrength(validatePasswordStrength(password));
  }, [password]);

  // ============================================================================
  // API Calls
  // ============================================================================

  const checkOnboardingStatus = async () => {
    try {
      const result = await invoke<OnboardingState>('check_first_run');
      setOnboardingState(result);

      // Show onboarding if it's first run and hasn't been completed
      if (result.is_first_run && !result.onboarding_completed) {
        setIsVisible(true);

        // Load assessment questions
        const questionsResult = await invoke<{ questions: OnboardingQuestion[] }>('get_onboarding_questions');
        setQuestions(questionsResult.questions || []);
      }
    } catch (err) {
      console.error('Error checking onboarding status:', err);
    }
  };

  const performWalletSetup = async (): Promise<boolean> => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<WalletSetupResult>('perform_first_time_setup', {
        password
      });
      setWalletResult(result);

      // Generate verification indices (3 random words from 12)
      const words = result.mnemonic.split(' ');
      const indices = generateVerificationIndices(words.length);
      setVerificationWords(indices.map(i => ({ index: i, word: words[i] })));
      setVerificationInputs(['', '', '']);

      return true;
    } catch (err: any) {
      setError(err.toString());
      return false;
    } finally {
      setLoading(false);
    }
  };

  const setupBundledModel = useCallback(async () => {
    setModelSetupStatus('setting-up');
    try {
      const result = await invoke<{ found: boolean; path?: string; size_mb?: number; model_name?: string; error?: string }>('setup_bundled_model');
      if (result.found) {
        setModelSetupStatus('ready');
        setModelInfo({
          name: result.model_name,
          size_mb: result.size_mb,
          path: result.path
        });
        setOnboardingState(prev => prev ? { ...prev, bundled_model_ready: true } : null);
      } else {
        setModelSetupStatus('not-found');
      }
    } catch (err) {
      setModelSetupStatus('error');
      console.error('Error setting up bundled model:', err);
    }
  }, []);

  const handleAnswerSelect = async (score: number) => {
    const newAnswers = [...answers, score];
    setAnswers(newAnswers);

    if (currentQuestionIndex < questions.length - 1) {
      setCurrentQuestionIndex(currentQuestionIndex + 1);
    } else {
      // Process all answers
      setLoading(true);
      try {
        const result = await invoke<SkillAssessmentResult>('process_onboarding_answer', {
          answers: newAnswers
        });
        setAssessmentResult(result);
        setStep('results');
      } catch (err) {
        console.error('Error processing answers:', err);
        setAssessmentResult({
          skill_level: 'beginner',
          recommended_path: ['Explore the dashboard', 'Try the AI chat', 'Check your wallet'],
          personalized_greeting: "Welcome to Citrate! Let's get you started."
        });
        setStep('results');
      } finally {
        setLoading(false);
      }
    }
  };

  const handleSkipOnboarding = async () => {
    try {
      await invoke('skip_onboarding');
      setIsVisible(false);
      onComplete?.();
    } catch (err) {
      console.error('Error skipping onboarding:', err);
      setIsVisible(false);
      onComplete?.();
    }
  };

  const handleComplete = async () => {
    try {
      await invoke('complete_onboarding');
    } catch (err) {
      console.error('Error completing onboarding:', err);
    }
    setIsVisible(false);
    onComplete?.();
  };

  // ============================================================================
  // Helper Functions
  // ============================================================================

  const generateVerificationIndices = (wordCount: number): number[] => {
    const indices: number[] = [];
    while (indices.length < 3) {
      const rand = Math.floor(Math.random() * wordCount);
      if (!indices.includes(rand)) {
        indices.push(rand);
      }
    }
    return indices.sort((a, b) => a - b);
  };

  const verifyMnemonic = (): boolean => {
    for (let i = 0; i < verificationWords.length; i++) {
      if (verificationInputs[i].toLowerCase().trim() !== verificationWords[i].word.toLowerCase()) {
        setVerificationError(`Word #${verificationWords[i].index + 1} is incorrect`);
        return false;
      }
    }
    setVerificationError(null);
    return true;
  };

  const copyMnemonic = () => {
    if (walletResult?.mnemonic) {
      navigator.clipboard.writeText(walletResult.mnemonic);
      setMnemonicCopied(true);
      setTimeout(() => setMnemonicCopied(false), 2000);
    }
  };

  const getStepNumber = (): number => {
    const steps: OnboardingStep[] = [
      'welcome',
      'network-select',
      'password-setup',
      'wallet-create',
      'mnemonic-backup',
      'mnemonic-verify',
      'model-setup',
      'assessment',
      'results',
      'complete'
    ];
    return steps.indexOf(step);
  };

  const getTotalSteps = (): number => {
    // Simplified step count for progress indicator
    return 8; // welcome, network, password, wallet, mnemonic, model, assessment, complete
  };

  // ============================================================================
  // Navigation
  // ============================================================================

  const goToNextStep = async () => {
    setError(null);

    switch (step) {
      case 'welcome':
        setStep('network-select');
        break;
      case 'network-select':
        setStep('password-setup');
        break;
      case 'password-setup':
        if (!passwordStrength.is_valid) {
          setError('Password does not meet requirements');
          return;
        }
        if (password !== confirmPassword) {
          setError('Passwords do not match');
          return;
        }
        setStep('wallet-create');
        break;
      case 'wallet-create':
        const success = await performWalletSetup();
        if (success) {
          setStep('mnemonic-backup');
        }
        break;
      case 'mnemonic-backup':
        if (!mnemonicConfirmed) {
          setError('Please confirm you have backed up your recovery phrase');
          return;
        }
        setStep('mnemonic-verify');
        break;
      case 'mnemonic-verify':
        if (verifyMnemonic()) {
          setStep('model-setup');
        }
        break;
      case 'model-setup':
        if (questions.length > 0) {
          setStep('assessment');
        } else {
          setStep('complete');
        }
        break;
      case 'huggingface-connect':
        if (questions.length > 0) {
          setStep('assessment');
        } else {
          setStep('complete');
        }
        break;
      case 'results':
        setStep('complete');
        break;
      default:
        break;
    }
  };

  const goToPreviousStep = () => {
    switch (step) {
      case 'network-select':
        setStep('welcome');
        break;
      case 'password-setup':
        setStep('network-select');
        break;
      case 'wallet-create':
        setStep('password-setup');
        break;
      case 'mnemonic-backup':
        setStep('wallet-create');
        break;
      case 'mnemonic-verify':
        setStep('mnemonic-backup');
        break;
      case 'model-setup':
        setStep('mnemonic-verify');
        break;
      case 'huggingface-connect':
        setStep('model-setup');
        break;
      case 'assessment':
        setStep('model-setup');
        break;
      default:
        break;
    }
  };

  // ============================================================================
  // Render
  // ============================================================================

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black/90 flex items-center justify-center z-[9998] backdrop-blur-md" style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0 }}>
      <div className="bg-white rounded-2xl shadow-2xl max-w-lg w-full mx-4 overflow-hidden max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="relative bg-gradient-to-r from-orange-500 to-orange-600 px-6 py-4">
          <button
            onClick={handleSkipOnboarding}
            className="absolute top-4 right-4 text-white/70 hover:text-white transition-colors"
            aria-label="Skip onboarding"
          >
            <X className="w-5 h-5" />
          </button>
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 bg-white/20 rounded-full flex items-center justify-center">
              <Sparkles className="w-5 h-5 text-white" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">Welcome to Citrate</h2>
              <p className="text-white/80 text-sm">AI-Native Blockchain</p>
            </div>
          </div>
        </div>

        <div className="p-6">
          {/* Progress Indicator */}
          {step !== 'welcome' && step !== 'complete' && (
            <StepIndicator currentStep={getStepNumber()} totalSteps={getTotalSteps()} />
          )}

          {/* Error Display */}
          {error && (
            <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg flex items-start space-x-2">
              <AlertTriangle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
              <p className="text-sm text-red-700">{error}</p>
            </div>
          )}

          {/* ================================================================== */}
          {/* WELCOME STEP */}
          {/* ================================================================== */}
          {step === 'welcome' && (
            <div className="text-center">
              <div className="mx-auto w-20 h-20 bg-orange-100 rounded-full flex items-center justify-center mb-6">
                <Bot className="w-10 h-10 text-orange-600" />
              </div>
              <h3 className="text-2xl font-bold text-gray-900 mb-3">
                Your AI Assistant is Ready
              </h3>
              <p className="text-gray-600 mb-6">
                Citrate comes with a built-in AI assistant that can help you navigate blockchain
                operations, manage your wallet, and interact with smart contracts naturally.
              </p>

              <div className="space-y-3 mb-8 text-left">
                <FeatureItem
                  icon={<MessageSquare className="w-5 h-5" />}
                  title="Natural Language Interface"
                  description="Chat with your blockchain like you would with an expert"
                />
                <FeatureItem
                  icon={<Zap className="w-5 h-5" />}
                  title="Offline Capable"
                  description="Works locally with bundled AI model - no API key required"
                />
                <FeatureItem
                  icon={<Brain className="w-5 h-5" />}
                  title="Smart Assistance"
                  description="Get personalized help based on your experience level"
                />
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={handleSkipOnboarding}
                  className="flex-1 px-4 py-3 border border-gray-300 rounded-lg text-gray-700 font-medium hover:bg-gray-50 transition-colors"
                >
                  Skip for Now
                </button>
                <button
                  onClick={() => setStep('network-select')}
                  className="flex-1 px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2"
                >
                  <span>Get Started</span>
                  <ChevronRight className="w-5 h-5" />
                </button>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* NETWORK SELECTION STEP */}
          {/* ================================================================== */}
          {step === 'network-select' && (
            <div>
              <div className="text-center mb-6">
                <div className="mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mb-4">
                  <Globe className="w-8 h-8 text-blue-600" />
                </div>
                <h3 className="text-xl font-bold text-gray-900 mb-2">Choose Your Network</h3>
                <p className="text-gray-600 text-sm">
                  Select which network to connect to. You can change this later in settings.
                </p>
              </div>

              <div className="space-y-3 mb-6">
                <button
                  onClick={() => setSelectedNetwork('devnet')}
                  className={`w-full p-4 text-left border-2 rounded-lg transition-colors ${
                    selectedNetwork === 'devnet'
                      ? 'border-orange-500 bg-orange-50'
                      : 'border-gray-200 hover:border-gray-300'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-semibold text-gray-900">Local Devnet</h4>
                      <p className="text-sm text-gray-600">Run a local node for development and testing</p>
                    </div>
                    {selectedNetwork === 'devnet' && (
                      <CheckCircle className="w-5 h-5 text-orange-600" />
                    )}
                  </div>
                </button>

                <button
                  onClick={() => setSelectedNetwork('testnet')}
                  className={`w-full p-4 text-left border-2 rounded-lg transition-colors ${
                    selectedNetwork === 'testnet'
                      ? 'border-orange-500 bg-orange-50'
                      : 'border-gray-200 hover:border-gray-300'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-semibold text-gray-900">Public Testnet</h4>
                      <p className="text-sm text-gray-600">Connect to the Citrate test network</p>
                    </div>
                    {selectedNetwork === 'testnet' && (
                      <CheckCircle className="w-5 h-5 text-orange-600" />
                    )}
                  </div>
                </button>

                <button
                  onClick={() => setSelectedNetwork('mainnet')}
                  disabled
                  className="w-full p-4 text-left border-2 border-gray-200 rounded-lg opacity-50 cursor-not-allowed"
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-semibold text-gray-500">Mainnet</h4>
                      <p className="text-sm text-gray-400">Coming soon - production network</p>
                    </div>
                    <span className="text-xs bg-gray-200 text-gray-600 px-2 py-1 rounded">Coming Soon</span>
                  </div>
                </button>
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={goToPreviousStep}
                  className="px-4 py-3 border border-gray-300 rounded-lg text-gray-700 font-medium hover:bg-gray-50 transition-colors flex items-center space-x-2"
                >
                  <ChevronLeft className="w-5 h-5" />
                </button>
                <button
                  onClick={goToNextStep}
                  className="flex-1 px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2"
                >
                  <span>Continue</span>
                  <ChevronRight className="w-5 h-5" />
                </button>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* PASSWORD SETUP STEP */}
          {/* ================================================================== */}
          {step === 'password-setup' && (
            <div>
              <div className="text-center mb-6">
                <div className="mx-auto w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mb-4">
                  <Lock className="w-8 h-8 text-green-600" />
                </div>
                <h3 className="text-xl font-bold text-gray-900 mb-2">Secure Your Wallet</h3>
                <p className="text-gray-600 text-sm">
                  Create a strong password to encrypt your wallet and protect your funds.
                </p>
              </div>

              <div className="space-y-4 mb-6">
                {/* Password Input */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Password</label>
                  <div className="relative">
                    <input
                      type={showPassword ? 'text' : 'password'}
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 pr-10"
                      placeholder="Enter a strong password"
                    />
                    <button
                      type="button"
                      onClick={() => setShowPassword(!showPassword)}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-700"
                    >
                      {showPassword ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
                    </button>
                  </div>

                  {/* Strength Indicator */}
                  <div className="mt-2">
                    <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                      <div
                        className={`h-full transition-all ${getPasswordStrengthColor(passwordStrength.score)}`}
                        style={{ width: `${passwordStrength.score}%` }}
                      />
                    </div>
                    {passwordStrength.feedback.length > 0 && (
                      <ul className="mt-2 text-xs text-gray-600 space-y-1">
                        {passwordStrength.feedback.map((f, i) => (
                          <li key={i} className="flex items-center space-x-1">
                            <span className="text-red-500">*</span>
                            <span>{f}</span>
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>
                </div>

                {/* Confirm Password */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Confirm Password</label>
                  <input
                    type={showPassword ? 'text' : 'password'}
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    className={`w-full px-4 py-3 border rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 ${
                      confirmPassword && password !== confirmPassword
                        ? 'border-red-500'
                        : 'border-gray-300'
                    }`}
                    placeholder="Confirm your password"
                  />
                  {confirmPassword && password !== confirmPassword && (
                    <p className="mt-1 text-xs text-red-500">Passwords do not match</p>
                  )}
                </div>
              </div>

              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
                <div className="flex items-start space-x-2">
                  <AlertTriangle className="w-5 h-5 text-yellow-600 flex-shrink-0 mt-0.5" />
                  <div className="text-sm text-yellow-800">
                    <p className="font-medium">Important</p>
                    <p>This password cannot be recovered. Make sure to remember it or store it securely.</p>
                  </div>
                </div>
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={goToPreviousStep}
                  className="px-4 py-3 border border-gray-300 rounded-lg text-gray-700 font-medium hover:bg-gray-50 transition-colors flex items-center space-x-2"
                >
                  <ChevronLeft className="w-5 h-5" />
                </button>
                <button
                  onClick={goToNextStep}
                  disabled={!passwordStrength.is_valid || password !== confirmPassword}
                  className="flex-1 px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <span>Create Wallet</span>
                  <ChevronRight className="w-5 h-5" />
                </button>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* WALLET CREATE STEP */}
          {/* ================================================================== */}
          {step === 'wallet-create' && (
            <div className="text-center">
              <div className="mx-auto w-20 h-20 bg-purple-100 rounded-full flex items-center justify-center mb-6">
                {loading ? (
                  <Loader2 className="w-10 h-10 text-purple-600 animate-spin" />
                ) : (
                  <Key className="w-10 h-10 text-purple-600" />
                )}
              </div>
              <h3 className="text-2xl font-bold text-gray-900 mb-3">
                {loading ? 'Creating Your Wallet...' : 'Ready to Create Wallet'}
              </h3>
              <p className="text-gray-600 mb-6">
                {loading
                  ? 'Generating secure keys and encrypting your wallet...'
                  : 'Your wallet will be created with a 12-word recovery phrase.'}
              </p>

              {!loading && (
                <button
                  onClick={goToNextStep}
                  className="w-full px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2"
                >
                  <Shield className="w-5 h-5" />
                  <span>Create Secure Wallet</span>
                </button>
              )}
            </div>
          )}

          {/* ================================================================== */}
          {/* MNEMONIC BACKUP STEP */}
          {/* ================================================================== */}
          {step === 'mnemonic-backup' && walletResult && (
            <div>
              <div className="text-center mb-6">
                <div className="mx-auto w-16 h-16 bg-yellow-100 rounded-full flex items-center justify-center mb-4">
                  <Key className="w-8 h-8 text-yellow-600" />
                </div>
                <h3 className="text-xl font-bold text-gray-900 mb-2">Backup Recovery Phrase</h3>
                <p className="text-gray-600 text-sm">
                  Write down these 12 words in order. This is the ONLY way to recover your wallet.
                </p>
              </div>

              {/* Mnemonic Display */}
              <div className="bg-gray-50 rounded-lg p-4 mb-4">
                <div className="grid grid-cols-3 gap-2">
                  {walletResult.mnemonic.split(' ').map((word, idx) => (
                    <div key={idx} className="flex items-center space-x-2 bg-white rounded px-2 py-1.5 border border-gray-200">
                      <span className="text-xs text-gray-400 w-4">{idx + 1}.</span>
                      <span className="font-mono text-sm text-gray-900">{word}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Copy Button */}
              <button
                onClick={copyMnemonic}
                className="w-full mb-4 px-4 py-2 border border-gray-300 rounded-lg text-gray-700 font-medium hover:bg-gray-50 transition-colors flex items-center justify-center space-x-2"
              >
                {mnemonicCopied ? (
                  <>
                    <CheckCircle className="w-4 h-4 text-green-600" />
                    <span className="text-green-600">Copied!</span>
                  </>
                ) : (
                  <>
                    <Copy className="w-4 h-4" />
                    <span>Copy to Clipboard</span>
                  </>
                )}
              </button>

              {/* Warning */}
              <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4">
                <div className="flex items-start space-x-2">
                  <AlertTriangle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
                  <div className="text-sm text-red-800">
                    <p className="font-medium">NEVER share your recovery phrase!</p>
                    <ul className="mt-1 list-disc list-inside text-xs space-y-1">
                      <li>Anyone with these words can steal your funds</li>
                      <li>Store it offline in a secure location</li>
                      <li>Never enter it on websites or share it online</li>
                    </ul>
                  </div>
                </div>
              </div>

              {/* Confirmation Checkbox */}
              <label className="flex items-start space-x-3 mb-6 cursor-pointer">
                <input
                  type="checkbox"
                  checked={mnemonicConfirmed}
                  onChange={(e) => setMnemonicConfirmed(e.target.checked)}
                  className="mt-1 w-4 h-4 text-orange-600 border-gray-300 rounded focus:ring-orange-500"
                />
                <span className="text-sm text-gray-700">
                  I have securely backed up my recovery phrase and understand that losing it means losing access to my wallet forever.
                </span>
              </label>

              {/* Wallet Address Display */}
              <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-6">
                <p className="text-xs text-green-700 mb-1">Your Wallet Address</p>
                <p className="font-mono text-sm text-green-900 break-all">{walletResult.primary_address}</p>
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={goToPreviousStep}
                  className="px-4 py-3 border border-gray-300 rounded-lg text-gray-700 font-medium hover:bg-gray-50 transition-colors flex items-center space-x-2"
                >
                  <ChevronLeft className="w-5 h-5" />
                </button>
                <button
                  onClick={goToNextStep}
                  disabled={!mnemonicConfirmed}
                  className="flex-1 px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <span>Verify Backup</span>
                  <ChevronRight className="w-5 h-5" />
                </button>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* MNEMONIC VERIFY STEP */}
          {/* ================================================================== */}
          {step === 'mnemonic-verify' && (
            <div>
              <div className="text-center mb-6">
                <div className="mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mb-4">
                  <Shield className="w-8 h-8 text-blue-600" />
                </div>
                <h3 className="text-xl font-bold text-gray-900 mb-2">Verify Recovery Phrase</h3>
                <p className="text-gray-600 text-sm">
                  Enter the requested words from your recovery phrase to confirm you saved it.
                </p>
              </div>

              <div className="space-y-4 mb-6">
                {verificationWords.map((item, idx) => (
                  <div key={idx}>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Word #{item.index + 1}
                    </label>
                    <input
                      type="text"
                      value={verificationInputs[idx]}
                      onChange={(e) => {
                        const newInputs = [...verificationInputs];
                        newInputs[idx] = e.target.value;
                        setVerificationInputs(newInputs);
                        setVerificationError(null);
                      }}
                      className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 font-mono"
                      placeholder={`Enter word #${item.index + 1}`}
                    />
                  </div>
                ))}
              </div>

              {verificationError && (
                <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg flex items-start space-x-2">
                  <AlertTriangle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
                  <p className="text-sm text-red-700">{verificationError}</p>
                </div>
              )}

              <div className="flex space-x-3">
                <button
                  onClick={goToPreviousStep}
                  className="px-4 py-3 border border-gray-300 rounded-lg text-gray-700 font-medium hover:bg-gray-50 transition-colors flex items-center space-x-2"
                >
                  <ChevronLeft className="w-5 h-5" />
                </button>
                <button
                  onClick={goToNextStep}
                  className="flex-1 px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2"
                >
                  <span>Verify</span>
                  <ChevronRight className="w-5 h-5" />
                </button>
              </div>
            </div>
          )}

          {/* ================================================================== */}
          {/* MODEL SETUP STEP */}
          {/* ================================================================== */}
          {step === 'model-setup' && (
            <div className="text-center">
              <div className="mx-auto w-20 h-20 bg-blue-100 rounded-full flex items-center justify-center mb-6">
                {modelSetupStatus === 'setting-up' ? (
                  <Loader2 className="w-10 h-10 text-blue-600 animate-spin" />
                ) : modelSetupStatus === 'ready' ? (
                  <CheckCircle className="w-10 h-10 text-green-600" />
                ) : modelSetupStatus === 'error' ? (
                  <AlertTriangle className="w-10 h-10 text-red-600" />
                ) : (
                  <Brain className="w-10 h-10 text-blue-600" />
                )}
              </div>

              <h3 className="text-2xl font-bold text-gray-900 mb-3">
                {modelSetupStatus === 'ready' && 'AI Model Ready!'}
                {modelSetupStatus === 'setting-up' && 'Setting Up AI Model...'}
                {modelSetupStatus === 'pending' && 'Setup Local AI Model'}
                {modelSetupStatus === 'error' && 'Model Setup Issue'}
                {modelSetupStatus === 'not-found' && 'No Bundled Model'}
              </h3>

              <p className="text-gray-600 mb-6">
                {modelSetupStatus === 'pending' &&
                  "Citrate includes a bundled AI model that works offline. Let's set it up for you."}
                {modelSetupStatus === 'setting-up' &&
                  'Configuring your local AI model. This only takes a moment...'}
                {modelSetupStatus === 'ready' &&
                  `${modelInfo?.name || 'AI model'} is configured and ready to help you!`}
                {modelSetupStatus === 'error' &&
                  'There was an issue setting up the model, but you can still use cloud AI providers.'}
                {modelSetupStatus === 'not-found' &&
                  'No bundled model found. You can use cloud AI providers or download a model later.'}
              </p>

              {modelSetupStatus === 'ready' && modelInfo && (
                <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-6 text-left">
                  <div className="flex items-center space-x-2 mb-2">
                    <CheckCircle className="w-4 h-4 text-green-600" />
                    <span className="font-medium text-green-800">{modelInfo.name}</span>
                  </div>
                  <p className="text-sm text-green-700">
                    Size: {modelInfo.size_mb} MB
                  </p>
                </div>
              )}

              {modelSetupStatus === 'pending' && (
                <button
                  onClick={setupBundledModel}
                  className="w-full px-4 py-3 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition-colors flex items-center justify-center space-x-2"
                >
                  <Download className="w-5 h-5" />
                  <span>Set Up Local AI Model</span>
                </button>
              )}

              {(modelSetupStatus === 'ready' || modelSetupStatus === 'error' || modelSetupStatus === 'not-found') && (
                <button
                  onClick={goToNextStep}
                  className="w-full px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2"
                >
                  <span>Continue</span>
                  <ChevronRight className="w-5 h-5" />
                </button>
              )}
            </div>
          )}

          {/* ================================================================== */}
          {/* ASSESSMENT STEP */}
          {/* ================================================================== */}
          {step === 'assessment' && questions.length > 0 && (
            <div>
              {/* Progress indicator */}
              <div className="flex space-x-1 mb-6">
                {questions.map((_, idx) => (
                  <div
                    key={idx}
                    className={`flex-1 h-1 rounded-full ${
                      idx < currentQuestionIndex
                        ? 'bg-orange-600'
                        : idx === currentQuestionIndex
                        ? 'bg-orange-400'
                        : 'bg-gray-200'
                    }`}
                  />
                ))}
              </div>

              <div className="text-center mb-6">
                <p className="text-sm text-gray-500 mb-2">
                  Question {currentQuestionIndex + 1} of {questions.length}
                </p>
                <h3 className="text-xl font-bold text-gray-900">
                  {questions[currentQuestionIndex]?.question}
                </h3>
              </div>

              <div className="space-y-3">
                {questions[currentQuestionIndex]?.options.map((option, idx) => (
                  <button
                    key={idx}
                    onClick={() => handleAnswerSelect(option.score)}
                    disabled={loading}
                    className="w-full p-4 text-left border border-gray-200 rounded-lg hover:border-orange-400 hover:bg-orange-50 transition-colors disabled:opacity-50"
                  >
                    <span className="text-gray-800">{option.text}</span>
                  </button>
                ))}
              </div>

              {loading && (
                <div className="mt-6 text-center">
                  <div className="inline-flex items-center space-x-2 text-gray-600">
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <span>Analyzing your responses...</span>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* ================================================================== */}
          {/* RESULTS STEP */}
          {/* ================================================================== */}
          {step === 'results' && assessmentResult && (
            <div className="text-center">
              <div className="mx-auto w-20 h-20 bg-green-100 rounded-full flex items-center justify-center mb-6">
                <Rocket className="w-10 h-10 text-green-600" />
              </div>

              <h3 className="text-2xl font-bold text-gray-900 mb-2">
                {getSkillLevelTitle(assessmentResult.skill_level)}
              </h3>

              <p className="text-gray-600 mb-6">
                {assessmentResult.personalized_greeting}
              </p>

              <div className="bg-gray-50 rounded-lg p-4 mb-6 text-left">
                <h4 className="font-semibold text-gray-900 mb-3 flex items-center">
                  <Sparkles className="w-4 h-4 mr-2 text-orange-600" />
                  Recommended Next Steps
                </h4>
                <ul className="space-y-2">
                  {assessmentResult.recommended_path.map((step, idx) => (
                    <li key={idx} className="flex items-start space-x-2">
                      <CheckCircle className="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                      <span className="text-gray-700 text-sm">{step}</span>
                    </li>
                  ))}
                </ul>
              </div>

              <button
                onClick={() => setStep('complete')}
                className="w-full px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2"
              >
                <span>Let's Go!</span>
                <ChevronRight className="w-5 h-5" />
              </button>
            </div>
          )}

          {/* ================================================================== */}
          {/* COMPLETE STEP */}
          {/* ================================================================== */}
          {step === 'complete' && (
            <div className="text-center">
              <div className="mx-auto w-20 h-20 bg-orange-100 rounded-full flex items-center justify-center mb-6">
                <CheckCircle className="w-10 h-10 text-orange-600" />
              </div>

              <h3 className="text-2xl font-bold text-gray-900 mb-3">
                You're All Set!
              </h3>

              <p className="text-gray-600 mb-6">
                Your Citrate workspace is configured and ready. You can always adjust
                settings or connect additional AI providers in Settings.
              </p>

              <div className="bg-gray-50 rounded-lg p-4 mb-6 space-y-3">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-gray-600">Wallet</span>
                  <span className="text-green-600 font-medium flex items-center">
                    <CheckCircle className="w-4 h-4 mr-1" />
                    Created & Secured
                  </span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-gray-600">Network</span>
                  <span className="text-blue-600 font-medium capitalize">
                    {selectedNetwork}
                  </span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-gray-600">AI Model</span>
                  <span className={`font-medium flex items-center ${
                    modelSetupStatus === 'ready' ? 'text-green-600' : 'text-gray-500'
                  }`}>
                    {modelSetupStatus === 'ready' ? (
                      <>
                        <CheckCircle className="w-4 h-4 mr-1" />
                        Local Model Ready
                      </>
                    ) : (
                      'Use Cloud Providers'
                    )}
                  </span>
                </div>
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={() => {
                    handleComplete();
                    window.dispatchEvent(new CustomEvent('navigate', { detail: { tab: 'settings' } }));
                  }}
                  className="flex-1 px-4 py-3 border border-gray-300 rounded-lg text-gray-700 font-medium hover:bg-gray-50 transition-colors flex items-center justify-center space-x-2"
                >
                  <Settings className="w-4 h-4" />
                  <span>Configure AI</span>
                </button>
                <button
                  onClick={handleComplete}
                  className="flex-1 px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors"
                >
                  Start Exploring
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default OnboardingModal;
