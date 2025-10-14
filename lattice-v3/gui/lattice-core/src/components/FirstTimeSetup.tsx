import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
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

  const handleSetupWallet = async () => {
    setLoading(true);
    try {
      const result = await invoke<FirstTimeSetupResult>('perform_first_time_setup', {
        password: 'user_secure_password'
      });

      setSetupResult(result);
      setStep('reveal');
      setLoading(false);
    } catch (error) {
      console.error('Error setting up wallet:', error);
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
            <div className="mx-auto w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mb-4">
              <Wallet className="w-8 h-8 text-blue-600" />
            </div>
            <h2 className="text-2xl font-bold text-gray-900 mb-4">
              Welcome to Lattice!
            </h2>
            <p className="text-gray-600 mb-6">
              This is your first time using Lattice. Let's set up your secure wallet
              and start earning rewards by contributing to the network.
            </p>
            <div className="space-y-3 mb-6">
              <div className="flex items-center text-sm text-gray-700">
                <Shield className="w-4 h-4 text-green-600 mr-2" />
                Secure wallet with 12-word recovery phrase
              </div>
              <div className="flex items-center text-sm text-gray-700">
                <Key className="w-4 h-4 text-blue-600 mr-2" />
                Automatic reward address configuration
              </div>
              <div className="flex items-center text-sm text-gray-700">
                <CheckCircle className="w-4 h-4 text-purple-600 mr-2" />
                Start earning tokens immediately
              </div>
            </div>
            <button
              onClick={() => setStep('setup')}
              className="w-full bg-blue-600 hover:bg-blue-700 text-white py-3 rounded-lg font-medium transition-colors"
            >
              Set Up My Wallet
            </button>
          </div>
        )}

        {/* Setup Step */}
        {step === 'setup' && (
          <div className="text-center">
            <div className="mx-auto w-16 h-16 bg-purple-100 rounded-full flex items-center justify-center mb-4">
              <Shield className="w-8 h-8 text-purple-600" />
            </div>
            <h2 className="text-2xl font-bold text-gray-900 mb-4">
              Creating Your Secure Wallet
            </h2>
            <p className="text-gray-600 mb-6">
              We'll generate a unique wallet with a 12-word recovery phrase.
              This will be automatically configured as your reward address.
            </p>
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
              <div className="flex items-start">
                <AlertTriangle className="w-5 h-5 text-yellow-600 mr-2 flex-shrink-0 mt-0.5" />
                <div className="text-sm text-yellow-800">
                  <strong>Important:</strong> Your recovery phrase is the only way to restore your wallet.
                  Keep it secure and never share it with anyone.
                </div>
              </div>
            </div>
            <button
              onClick={handleSetupWallet}
              disabled={loading}
              className="w-full bg-purple-600 hover:bg-purple-700 disabled:bg-gray-400 text-white py-3 rounded-lg font-medium transition-colors"
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
                  className="w-full flex items-center justify-center space-x-2 bg-blue-100 hover:bg-blue-200 text-blue-700 py-2 rounded-lg transition-colors"
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
              className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white py-3 rounded-lg font-medium transition-colors"
            >
              Complete Setup & Start Earning
            </button>
          </div>
        )}
      </div>
    </div>
  );
};