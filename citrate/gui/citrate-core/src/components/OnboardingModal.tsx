// OnboardingModal.tsx
//
// AI-first onboarding experience for Citrate
// Handles: bundled model setup, skill assessment, personalized guidance

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Sparkles,
  Brain,
  Rocket,
  CheckCircle,
  ChevronRight,
  Settings,
  Bot,
  Zap,
  MessageSquare,
  X
} from 'lucide-react';

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
}

interface SkillAssessmentResult {
  skill_level: 'beginner' | 'intermediate' | 'advanced';
  recommended_path: string[];
  personalized_greeting: string;
}

interface OnboardingModalProps {
  onComplete?: () => void;
}

export const OnboardingModal: React.FC<OnboardingModalProps> = ({ onComplete }) => {
  const [isVisible, setIsVisible] = useState(false);
  const [step, setStep] = useState<'welcome' | 'model-setup' | 'assessment' | 'results' | 'complete'>('welcome');
  const [loading, setLoading] = useState(false);
  const [onboardingState, setOnboardingState] = useState<OnboardingState | null>(null);
  const [questions, setQuestions] = useState<OnboardingQuestion[]>([]);
  const [currentQuestionIndex, setCurrentQuestionIndex] = useState(0);
  const [answers, setAnswers] = useState<number[]>([]);
  const [assessmentResult, setAssessmentResult] = useState<SkillAssessmentResult | null>(null);
  const [modelSetupStatus, setModelSetupStatus] = useState<'pending' | 'setting-up' | 'ready' | 'error'>('pending');

  // Check if we should show onboarding
  useEffect(() => {
    checkOnboardingStatus();
  }, []);

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
    } catch (error) {
      console.error('Error checking onboarding status:', error);
    }
  };

  const setupBundledModel = useCallback(async () => {
    setModelSetupStatus('setting-up');
    try {
      const result = await invoke<{ success: boolean; path?: string; error?: string }>('setup_bundled_model');
      if (result.success) {
        setModelSetupStatus('ready');
        // Update onboarding state
        setOnboardingState(prev => prev ? { ...prev, bundled_model_ready: true } : null);
      } else {
        setModelSetupStatus('error');
        console.error('Model setup failed:', result.error);
      }
    } catch (error) {
      setModelSetupStatus('error');
      console.error('Error setting up bundled model:', error);
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
      } catch (error) {
        console.error('Error processing answers:', error);
        // Still show results with default values
        setAssessmentResult({
          skill_level: 'beginner',
          recommended_path: ['Explore the dashboard', 'Try the AI chat', 'Check your wallet'],
          personalized_greeting: 'Welcome to Citrate! Let\'s get you started.'
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
    } catch (error) {
      console.error('Error skipping onboarding:', error);
      setIsVisible(false);
      onComplete?.();
    }
  };

  const handleComplete = async () => {
    try {
      await invoke('complete_onboarding');
    } catch (error) {
      console.error('Error completing onboarding:', error);
    }
    setIsVisible(false);
    onComplete?.();
  };

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black/90 flex items-center justify-center z-[9998] backdrop-blur-md" style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0 }}>
      <div className="bg-white rounded-2xl shadow-2xl max-w-lg w-full mx-4 overflow-hidden">
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
          {/* Welcome Step */}
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
                  onClick={() => {
                    if (onboardingState?.has_bundled_model && !onboardingState?.bundled_model_ready) {
                      setStep('model-setup');
                    } else if (questions.length > 0) {
                      setStep('assessment');
                    } else {
                      setStep('complete');
                    }
                  }}
                  className="flex-1 px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2"
                >
                  <span>Get Started</span>
                  <ChevronRight className="w-5 h-5" />
                </button>
              </div>
            </div>
          )}

          {/* Model Setup Step */}
          {step === 'model-setup' && (
            <div className="text-center">
              <div className="mx-auto w-20 h-20 bg-blue-100 rounded-full flex items-center justify-center mb-6">
                {modelSetupStatus === 'setting-up' ? (
                  <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-blue-600" />
                ) : modelSetupStatus === 'ready' ? (
                  <CheckCircle className="w-10 h-10 text-green-600" />
                ) : (
                  <Brain className="w-10 h-10 text-blue-600" />
                )}
              </div>

              <h3 className="text-2xl font-bold text-gray-900 mb-3">
                {modelSetupStatus === 'ready' ? 'AI Model Ready!' : 'Setting Up AI Model'}
              </h3>

              <p className="text-gray-600 mb-6">
                {modelSetupStatus === 'pending' &&
                  'Citrate includes a bundled AI model that works offline. Let\'s set it up.'}
                {modelSetupStatus === 'setting-up' &&
                  'Configuring your local AI model. This only takes a moment...'}
                {modelSetupStatus === 'ready' &&
                  'Your AI assistant is configured and ready to help you!'}
                {modelSetupStatus === 'error' &&
                  'There was an issue setting up the model, but you can still use cloud providers.'}
              </p>

              {modelSetupStatus === 'pending' && (
                <button
                  onClick={setupBundledModel}
                  className="w-full px-4 py-3 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition-colors"
                >
                  Set Up Local AI Model
                </button>
              )}

              {(modelSetupStatus === 'ready' || modelSetupStatus === 'error') && (
                <button
                  onClick={() => questions.length > 0 ? setStep('assessment') : setStep('complete')}
                  className="w-full px-4 py-3 bg-orange-600 text-white rounded-lg font-medium hover:bg-orange-700 transition-colors flex items-center justify-center space-x-2"
                >
                  <span>Continue</span>
                  <ChevronRight className="w-5 h-5" />
                </button>
              )}
            </div>
          )}

          {/* Assessment Step */}
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
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-orange-600" />
                    <span>Analyzing your responses...</span>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Results Step */}
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

          {/* Complete Step */}
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

              <div className="bg-gray-50 rounded-lg p-4 mb-6">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-gray-600">AI Model</span>
                  <span className="text-green-600 font-medium flex items-center">
                    <CheckCircle className="w-4 h-4 mr-1" />
                    {onboardingState?.bundled_model_ready ? 'Local Model Ready' : 'Available'}
                  </span>
                </div>
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={() => {
                    handleComplete();
                    // Navigate to settings if they want to configure providers
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

// Helper component for feature items
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

// Helper function for skill level titles
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

export default OnboardingModal;
