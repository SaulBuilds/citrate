import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Shield,
  Copy,
  Eye,
  EyeOff,
  CheckCircle,
  AlertTriangle,
  Wallet,
  Key
} from 'lucide-react';
import { validateMnemonic } from '../utils/validation';

interface FirstTimeSetupResult {
  primary_address: string;
  mnemonic: string;
  warning_message: string;
}

interface FirstTimeSetupProps {
  onSetupComplete: () => void;
}

export const FirstTimeSetup: React.FC<FirstTimeSetupProps> = ({ onSetupComplete }) => {
  const [isVisible, setIsVisible] = useState(false);
  const [step, setStep] = useState<'welcome' | 'setup' | 'reveal' | 'confirm' | 'complete'>('welcome');
  const [setupResult, setSetupResult] = useState<FirstTimeSetupResult | null>(null);
  const [showMnemonic, setShowMnemonic] = useState(false);
  const [copied, setCopied] = useState(false);
  const [confirmed, setConfirmed] = useState(false);
  const [loading, setLoading] = useState(false);

  // Password management state
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [passwordError, setPasswordError] = useState('');
  const [passwordStrength, setPasswordStrength] = useState(0);

  // Mnemonic validation state
  const [mnemonicWarning, setMnemonicWarning] = useState('');

  useEffect(() => {
    checkFirstTimeSetup();
  }, []);

  const checkFirstTimeSetup = async () => {
    try {
      const isFirstTime = await invoke<boolean>('is_first_time_setup');
      if (isFirstTime) {
        setIsVisible(true);
      }
    } catch (error) {
      console.error('Error checking first-time setup:', error);
    }
  };

  const calculatePasswordStrength = (pwd: string): number => {
    let strength = 0;
    if (pwd.length >= 8) strength++;
    if (pwd.length >= 12) strength++;
    if (/[a-z]/.test(pwd) && /[A-Z]/.test(pwd)) strength++;
    if (/\d/.test(pwd)) strength++;
    if (/[^a-zA-Z0-9]/.test(pwd)) strength++;
    return Math.min(strength, 4);
  };

  const validatePasswords = (): boolean => {
    if (password.length < 8) {
      setPasswordError('Password must be at least 8 characters');
      return false;
    }
    if (password !== confirmPassword) {
      setPasswordError('Passwords do not match');
      return false;
    }
    setPasswordError('');
    return true;
  };

  const handleSetupWallet = async () => {
    // Validate passwords before proceeding
    if (!validatePasswords()) {
      return;
    }

    setLoading(true);
    setPasswordError(''); // Clear any previous errors

    try {
      const result = await invoke<FirstTimeSetupResult>('perform_first_time_setup', {
        password: password  // Use user-provided password instead of hardcoded value
      });

      // Validate the received mnemonic
      const mnemonicValidation = validateMnemonic(result.mnemonic);
      if (!mnemonicValidation.isValid) {
        setMnemonicWarning(`Warning: Generated mnemonic validation failed: ${mnemonicValidation.error}`);
      } else {
        setMnemonicWarning('');
      }

      setSetupResult(result);
      setStep('reveal');
    } catch (error) {
      console.error('Error setting up wallet:', error);
      setPasswordError('Failed to create wallet. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  };

  const handleComplete = () => {
    setIsVisible(false);
    onSetupComplete();
  };

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50 backdrop-blur-sm">
      <div className="bg-white rounded-xl shadow-2xl max-w-md w-full mx-4 p-6">

        {/* Welcome Step */}
        {step === 'welcome' && (
          <div className="text-center">
            <div className="mx-auto w-16 h-16 bg-orange-100 rounded-full flex items-center justify-center mb-4">
              <Wallet className="w-8 h-8" style={{color: 'var(--brand-primary)'}} />
            </div>
            <h2 className="text-2xl font-bold text-gray-900 mb-4">
              Welcome to Citrate!
            </h2>
            <p className="text-gray-600 mb-6">
              This is your first time using Citrate. Let's set up your secure wallet
              and start earning rewards by contributing to the network.
            </p>
            <div className="space-y-3 mb-6">
              <div className="flex items-center text-sm text-gray-700">
                <Shield className="w-4 h-4 mr-2" style={{color: '#10b981'}} />
                Secure wallet with 12-word recovery phrase
              </div>
              <div className="flex items-center text-sm text-gray-700">
                <Key className="w-4 h-4 mr-2" style={{color: 'var(--brand-primary)'}} />
                Automatic reward address configuration
              </div>
              <div className="flex items-center text-sm text-gray-700">
                <CheckCircle className="w-4 h-4 mr-2" style={{color: 'var(--brand-primary)'}} />
                Start earning tokens immediately
              </div>
            </div>
            <button
              onClick={() => setStep('setup')}
              className="w-full text-white py-3 rounded-lg font-medium transition-colors"
              style={{backgroundColor: 'var(--brand-primary)'}}
              onMouseEnter={(e) => e.currentTarget.style.backgroundColor = 'var(--brand-hover)'}
              onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'var(--brand-primary)'}
            >
              Set Up My Wallet
            </button>
          </div>
        )}

        {/* Setup Step */}
        {step === 'setup' && (
          <div>
            <div className="text-center mb-6">
              <div className="mx-auto w-16 h-16 bg-orange-100 rounded-full flex items-center justify-center mb-4">
                <Shield className="w-8 h-8" style={{color: 'var(--brand-primary)'}} />
              </div>
              <h2 className="text-2xl font-bold text-gray-900 mb-2">
                Create Your Secure Wallet
              </h2>
              <p className="text-gray-600 text-sm">
                Choose a strong password to protect your wallet
              </p>
            </div>

            {/* Password Input */}
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Password
              </label>
              <div className="relative">
                <input
                  type={showPassword ? 'text' : 'password'}
                  value={password}
                  onChange={(e) => {
                    setPassword(e.target.value);
                    setPasswordStrength(calculatePasswordStrength(e.target.value));
                    setPasswordError('');
                  }}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg pr-10 focus:ring-2 focus:ring-orange-500 focus:border-transparent"
                  placeholder="Enter password"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-2 top-2 p-1 text-gray-500 hover:text-gray-700"
                >
                  {showPassword ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
                </button>
              </div>
            </div>

            {/* Confirm Password Input */}
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Confirm Password
              </label>
              <input
                type={showPassword ? 'text' : 'password'}
                value={confirmPassword}
                onChange={(e) => {
                  setConfirmPassword(e.target.value);
                  setPasswordError('');
                }}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-transparent"
                placeholder="Re-enter password"
              />
            </div>

            {/* Password Strength Indicator */}
            {password && (
              <div className="mb-4">
                <div className="flex items-center justify-between mb-1">
                  <span className="text-sm text-gray-600">Password Strength</span>
                  <span className={`text-sm font-medium ${
                    passwordStrength === 0 ? 'text-red-600' :
                    passwordStrength <= 2 ? 'text-yellow-600' :
                    passwordStrength === 3 ? 'text-orange-500' :
                    'text-green-600'
                  }`}>
                    {passwordStrength === 0 ? 'Too Weak' :
                     passwordStrength <= 2 ? 'Weak' :
                     passwordStrength === 3 ? 'Good' :
                     'Strong'}
                  </span>
                </div>
                <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                  <div
                    className={`h-full transition-all duration-300 ${
                      passwordStrength === 0 ? 'bg-red-500 w-1/4' :
                      passwordStrength <= 2 ? 'bg-yellow-500 w-2/4' :
                      passwordStrength === 3 ? 'bg-orange-500 w-3/4' :
                      'bg-green-500 w-full'
                    }`}
                  />
                </div>
              </div>
            )}

            {/* Password Requirements */}
            <div className="text-xs text-gray-600 mb-4 bg-gray-50 rounded-lg p-3">
              <p className="font-medium mb-1">Password should contain:</p>
              <ul className="space-y-1">
                <li className={password.length >= 8 ? 'text-green-600' : ''}>
                  • At least 8 characters {password.length >= 8 && '✓'}
                </li>
                <li className={/[A-Z]/.test(password) && /[a-z]/.test(password) ? 'text-green-600' : ''}>
                  • Upper and lowercase letters {/[A-Z]/.test(password) && /[a-z]/.test(password) && '✓'}
                </li>
                <li className={/\d/.test(password) ? 'text-green-600' : ''}>
                  • At least one number {/\d/.test(password) && '✓'}
                </li>
                <li className={/[^a-zA-Z0-9]/.test(password) ? 'text-green-600' : ''}>
                  • At least one special character {/[^a-zA-Z0-9]/.test(password) && '✓'}
                </li>
              </ul>
            </div>

            {/* Error Message */}
            {passwordError && (
              <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                <p className="text-red-600 text-sm">{passwordError}</p>
              </div>
            )}

            {/* Warning */}
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
              <div className="flex items-start">
                <AlertTriangle className="w-5 h-5 text-yellow-600 mr-2 flex-shrink-0 mt-0.5" />
                <div className="text-sm text-yellow-800">
                  <strong>Important:</strong> Your password protects your wallet and recovery phrase.
                  Make sure it's strong and keep it secure.
                </div>
              </div>
            </div>

            {/* Create Wallet Button */}
            <button
              onClick={handleSetupWallet}
              disabled={loading || !password || !confirmPassword}
              className="w-full disabled:opacity-50 disabled:cursor-not-allowed text-white py-3 rounded-lg font-medium transition-colors"
              style={{backgroundColor: (loading || !password || !confirmPassword) ? '#9ca3af' : 'var(--brand-primary)'}}
              onMouseEnter={(e) => (!loading && password && confirmPassword) && (e.currentTarget.style.backgroundColor = 'var(--brand-hover)')}
              onMouseLeave={(e) => (!loading && password && confirmPassword) && (e.currentTarget.style.backgroundColor = 'var(--brand-primary)')}
            >
              {loading ? 'Creating Wallet...' : 'Create Secure Wallet'}
            </button>
          </div>
        )}

        {/* Reveal Step */}
        {step === 'reveal' && setupResult && (
          <div>
            <div className="text-center mb-6">
              <div className="mx-auto w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mb-4">
                <CheckCircle className="w-8 h-8 text-green-600" />
              </div>
              <h2 className="text-2xl font-bold text-gray-900 mb-2">
                Wallet Created Successfully!
              </h2>
              <p className="text-sm text-gray-600">
                Your address: <span className="font-mono bg-gray-100 px-2 py-1 rounded">
                  {setupResult.primary_address.slice(0, 8)}...{setupResult.primary_address.slice(-6)}
                </span>
              </p>
            </div>

            <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4">
              <div className="flex items-start">
                <AlertTriangle className="w-5 h-5 text-red-600 mr-2 flex-shrink-0 mt-0.5" />
                <div className="text-sm text-red-800">
                  {setupResult.warning_message}
                </div>
              </div>
            </div>

            {mnemonicWarning && (
              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
                <div className="flex items-start">
                  <AlertTriangle className="w-5 h-5 text-yellow-600 mr-2 flex-shrink-0 mt-0.5" />
                  <div className="text-sm text-yellow-800">
                    {mnemonicWarning}
                  </div>
                </div>
              </div>
            )}

            <div className="mb-6">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Recovery Phrase (Keep this secure!)
              </label>
              <div className="relative">
                <div className="bg-gray-100 border rounded-lg p-4 font-mono text-sm">
                  {showMnemonic ? (
                    <div className="grid grid-cols-3 gap-2">
                      {setupResult.mnemonic.split(' ').map((word, index) => (
                        <div key={index} className="bg-white px-2 py-1 rounded border text-center">
                          <span className="text-xs text-gray-500">{index + 1}</span>
                          <div className="font-medium">{word}</div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 text-gray-500">
                      Click the eye icon to reveal your recovery phrase
                    </div>
                  )}
                </div>
                <button
                  onClick={() => setShowMnemonic(!showMnemonic)}
                  className="absolute top-2 right-2 p-2 text-gray-500 hover:text-gray-700"
                >
                  {showMnemonic ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              </div>
            </div>

            {showMnemonic && (
              <div className="space-y-3 mb-6">
                <button
                  onClick={() => copyToClipboard(setupResult.mnemonic)}
                  className="w-full flex items-center justify-center space-x-2 bg-gray-100 hover:bg-gray-200 text-gray-700 py-2 rounded-lg transition-colors"
                >
                  <Copy className="w-4 h-4" />
                  <span>{copied ? 'Copied!' : 'Copy Recovery Phrase'}</span>
                </button>

                <button
                  onClick={() => copyToClipboard(setupResult.primary_address)}
                  className="w-full flex items-center justify-center space-x-2 py-2 rounded-lg transition-colors text-gray-700"
                  style={{backgroundColor: '#f3f4f6'}}
                  onMouseEnter={(e) => e.currentTarget.style.backgroundColor = '#e5e7eb'}
                  onMouseLeave={(e) => e.currentTarget.style.backgroundColor = '#f3f4f6'}
                >
                  <Wallet className="w-4 h-4" />
                  <span>Copy Wallet Address</span>
                </button>
              </div>
            )}

            <div className="flex items-center mb-6">
              <input
                type="checkbox"
                id="confirm-backup"
                checked={confirmed}
                onChange={(e) => setConfirmed(e.target.checked)}
                className="mr-2"
              />
              <label htmlFor="confirm-backup" className="text-sm text-gray-700">
                I have securely saved my recovery phrase
              </label>
            </div>

            <button
              onClick={handleComplete}
              disabled={!confirmed}
              className="w-full disabled:bg-gray-400 text-white py-3 rounded-lg font-medium transition-colors"
              style={{backgroundColor: confirmed ? 'var(--brand-primary)' : '#9ca3af'}}
              onMouseEnter={(e) => confirmed && (e.currentTarget.style.backgroundColor = 'var(--brand-hover)')}
              onMouseLeave={(e) => confirmed && (e.currentTarget.style.backgroundColor = 'var(--brand-primary)')}
            >
              Complete Setup & Start Earning
            </button>
          </div>
        )}
      </div>
    </div>
  );
};