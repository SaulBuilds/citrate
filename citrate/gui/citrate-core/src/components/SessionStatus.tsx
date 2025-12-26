/**
 * SessionStatus Component
 *
 * Displays the current wallet session status with countdown timer.
 * Allows users to see session state and manually lock the wallet.
 */

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Lock, Unlock, Clock, Shield } from 'lucide-react';

interface SessionStatusProps {
  address: string;
  onSessionExpired?: () => void;
  onLock?: () => void;
  compact?: boolean;
}

export const SessionStatus: React.FC<SessionStatusProps> = ({
  address,
  onSessionExpired,
  onLock,
  compact = false,
}) => {
  const [remaining, setRemaining] = useState<number>(0);
  const [isActive, setIsActive] = useState<boolean>(false);
  const [isLocking, setIsLocking] = useState<boolean>(false);

  const checkSession = useCallback(async () => {
    if (!address) return;

    try {
      const secs = await invoke<number>('get_session_remaining', { address });
      setRemaining(secs);
      const wasActive = isActive;
      const nowActive = secs > 0;
      setIsActive(nowActive);

      // Trigger callback if session just expired
      if (wasActive && !nowActive && onSessionExpired) {
        onSessionExpired();
      }
    } catch (e) {
      console.error('Failed to check session:', e);
      setIsActive(false);
      setRemaining(0);
    }
  }, [address, isActive, onSessionExpired]);

  useEffect(() => {
    checkSession();
    // Poll every 5 seconds
    const interval = setInterval(checkSession, 5000);
    return () => clearInterval(interval);
  }, [checkSession]);

  // Update countdown every second when active
  useEffect(() => {
    if (!isActive || remaining <= 0) return;

    const timer = setInterval(() => {
      setRemaining((prev) => {
        const newVal = prev - 1;
        if (newVal <= 0) {
          setIsActive(false);
          if (onSessionExpired) {
            onSessionExpired();
          }
          return 0;
        }
        return newVal;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [isActive, onSessionExpired]);

  const handleLock = async () => {
    if (isLocking) return;

    setIsLocking(true);
    try {
      await invoke('lock_wallet', { address });
      setIsActive(false);
      setRemaining(0);
      if (onLock) {
        onLock();
      }
    } catch (e) {
      console.error('Failed to lock wallet:', e);
    } finally {
      setIsLocking(false);
    }
  };

  const formatTime = (secs: number): string => {
    const mins = Math.floor(secs / 60);
    const s = secs % 60;
    return `${mins}:${s.toString().padStart(2, '0')}`;
  };

  // Compact mode for inline display
  if (compact) {
    if (!isActive) {
      return (
        <div className="flex items-center gap-1.5 text-xs text-gray-500">
          <Lock className="w-3 h-3" />
          <span>Locked</span>
        </div>
      );
    }

    return (
      <div className="flex items-center gap-2">
        <div className="flex items-center gap-1.5 text-xs text-green-600">
          <Unlock className="w-3 h-3" />
          <span>{formatTime(remaining)}</span>
        </div>
        <button
          onClick={handleLock}
          disabled={isLocking}
          className="text-xs text-gray-500 hover:text-red-500 transition-colors"
          title="Lock wallet"
        >
          <Lock className="w-3 h-3" />
        </button>
      </div>
    );
  }

  // Full display mode
  if (!isActive) {
    return (
      <div className="flex items-center gap-2 px-3 py-2 bg-gray-100 dark:bg-gray-800 rounded-lg text-sm">
        <Lock className="w-4 h-4 text-gray-400" />
        <span className="text-gray-500 dark:text-gray-400">Session inactive</span>
      </div>
    );
  }

  // Warning state when less than 60 seconds remaining
  const isWarning = remaining <= 60;

  return (
    <div
      className={`flex items-center justify-between gap-3 px-3 py-2 rounded-lg text-sm ${
        isWarning
          ? 'bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800'
          : 'bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800'
      }`}
    >
      <div className="flex items-center gap-2">
        <div className={`w-2 h-2 rounded-full ${isWarning ? 'bg-orange-500' : 'bg-green-500'} animate-pulse`} />
        <Shield className={`w-4 h-4 ${isWarning ? 'text-orange-600' : 'text-green-600'}`} />
        <span className={isWarning ? 'text-orange-700 dark:text-orange-300' : 'text-green-700 dark:text-green-300'}>
          Session active
        </span>
      </div>

      <div className="flex items-center gap-3">
        <div className={`flex items-center gap-1 ${isWarning ? 'text-orange-600' : 'text-green-600'}`}>
          <Clock className="w-4 h-4" />
          <span className="font-mono font-medium">{formatTime(remaining)}</span>
        </div>

        <button
          onClick={handleLock}
          disabled={isLocking}
          className={`px-2 py-1 text-xs font-medium rounded transition-colors ${
            isLocking
              ? 'bg-gray-200 text-gray-400 cursor-not-allowed'
              : 'bg-red-100 text-red-600 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-900/50'
          }`}
        >
          <div className="flex items-center gap-1">
            <Lock className="w-3 h-3" />
            {isLocking ? 'Locking...' : 'Lock'}
          </div>
        </button>
      </div>
    </div>
  );
};

export default SessionStatus;
