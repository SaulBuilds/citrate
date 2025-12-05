/**
 * App Preview Component
 *
 * Sprint 5: Multi-Window & Terminal Integration
 *
 * Embedded webview for previewing running applications.
 */

import React, { useState, useCallback, useEffect, useRef } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

interface AppPreviewProps {
  /** Initial URL to load */
  url?: string;
  /** Window title */
  title?: string;
  /** Callback when URL changes */
  onNavigate?: (url: string) => void;
  /** Callback when preview is ready */
  onReady?: () => void;
  /** Custom CSS class */
  className?: string;
}

interface NavigationState {
  url: string;
  canGoBack: boolean;
  canGoForward: boolean;
  isLoading: boolean;
  error?: string;
}

export function AppPreview({
  url: initialUrl = 'http://localhost:3000',
  title: initialTitle = 'App Preview',
  onNavigate,
  onReady,
  className = '',
}: AppPreviewProps) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [navigationState, setNavigationState] = useState<NavigationState>({
    url: initialUrl,
    canGoBack: false,
    canGoForward: false,
    isLoading: true,
  });
  const [inputUrl, setInputUrl] = useState(initialUrl);
  const [title, setTitle] = useState(initialTitle);
  const historyRef = useRef<string[]>([initialUrl]);
  const historyIndexRef = useRef(0);

  // Handle navigation
  const navigate = useCallback((url: string, addToHistory = true) => {
    // Validate URL
    try {
      new URL(url);
    } catch {
      // Try adding protocol
      if (!url.startsWith('http://') && !url.startsWith('https://')) {
        url = `http://${url}`;
      }
    }

    setNavigationState((prev) => ({
      ...prev,
      url,
      isLoading: true,
      error: undefined,
    }));
    setInputUrl(url);

    if (addToHistory) {
      // Truncate forward history
      historyRef.current = historyRef.current.slice(0, historyIndexRef.current + 1);
      historyRef.current.push(url);
      historyIndexRef.current = historyRef.current.length - 1;
    }

    // Update navigation buttons
    setNavigationState((prev) => ({
      ...prev,
      canGoBack: historyIndexRef.current > 0,
      canGoForward: historyIndexRef.current < historyRef.current.length - 1,
    }));

    onNavigate?.(url);
  }, [onNavigate]);

  // Go back in history
  const goBack = useCallback(() => {
    if (historyIndexRef.current > 0) {
      historyIndexRef.current--;
      const url = historyRef.current[historyIndexRef.current];
      navigate(url, false);
    }
  }, [navigate]);

  // Go forward in history
  const goForward = useCallback(() => {
    if (historyIndexRef.current < historyRef.current.length - 1) {
      historyIndexRef.current++;
      const url = historyRef.current[historyIndexRef.current];
      navigate(url, false);
    }
  }, [navigate]);

  // Reload current page
  const reload = useCallback(() => {
    if (iframeRef.current) {
      setNavigationState((prev) => ({ ...prev, isLoading: true }));
      iframeRef.current.src = navigationState.url;
    }
  }, [navigationState.url]);

  // Handle URL input submit
  const handleUrlSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault();
    navigate(inputUrl);
  }, [inputUrl, navigate]);

  // Handle iframe load
  const handleLoad = useCallback(() => {
    setNavigationState((prev) => ({ ...prev, isLoading: false }));
    onReady?.();

    // Try to get the title from the iframe
    try {
      const doc = iframeRef.current?.contentDocument;
      if (doc?.title) {
        setTitle(doc.title);
      }
    } catch {
      // Cross-origin restriction - can't access document
    }
  }, [onReady]);

  // Handle iframe error
  const handleError = useCallback(() => {
    setNavigationState((prev) => ({
      ...prev,
      isLoading: false,
      error: 'Failed to load page',
    }));
  }, []);

  // Listen for external navigation commands
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setup = async () => {
      unlisten = await listen<{ url: string; title?: string }>('preview:navigate', (event) => {
        if (event.payload.title) {
          setTitle(event.payload.title);
        }
        navigate(event.payload.url);
      });
    };

    setup();

    return () => {
      unlisten?.();
    };
  }, [navigate]);

  // Listen for reload commands
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setup = async () => {
      unlisten = await listen('preview:reload', () => {
        reload();
      });
    };

    setup();

    return () => {
      unlisten?.();
    };
  }, [reload]);

  return (
    <div
      className={`preview-container ${className}`}
      style={{
        display: 'flex',
        flexDirection: 'column',
        width: '100%',
        height: '100%',
        backgroundColor: '#1e1e2e',
      }}
    >
      {/* Navigation Bar */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          padding: '8px 12px',
          backgroundColor: '#313244',
          borderBottom: '1px solid #45475a',
        }}
      >
        {/* Navigation Buttons */}
        <button
          onClick={goBack}
          disabled={!navigationState.canGoBack}
          style={{
            padding: '4px 8px',
            backgroundColor: 'transparent',
            border: '1px solid #45475a',
            borderRadius: '4px',
            color: navigationState.canGoBack ? '#cdd6f4' : '#6c7086',
            cursor: navigationState.canGoBack ? 'pointer' : 'not-allowed',
            fontSize: '14px',
          }}
          title="Go Back"
        >
          ←
        </button>

        <button
          onClick={goForward}
          disabled={!navigationState.canGoForward}
          style={{
            padding: '4px 8px',
            backgroundColor: 'transparent',
            border: '1px solid #45475a',
            borderRadius: '4px',
            color: navigationState.canGoForward ? '#cdd6f4' : '#6c7086',
            cursor: navigationState.canGoForward ? 'pointer' : 'not-allowed',
            fontSize: '14px',
          }}
          title="Go Forward"
        >
          →
        </button>

        <button
          onClick={reload}
          style={{
            padding: '4px 8px',
            backgroundColor: 'transparent',
            border: '1px solid #45475a',
            borderRadius: '4px',
            color: '#cdd6f4',
            cursor: 'pointer',
            fontSize: '14px',
          }}
          title="Reload"
        >
          ↻
        </button>

        {/* URL Input */}
        <form onSubmit={handleUrlSubmit} style={{ flex: 1 }}>
          <input
            type="text"
            value={inputUrl}
            onChange={(e) => setInputUrl(e.target.value)}
            style={{
              width: '100%',
              padding: '6px 12px',
              backgroundColor: '#1e1e2e',
              border: '1px solid #45475a',
              borderRadius: '4px',
              color: '#cdd6f4',
              fontSize: '13px',
              outline: 'none',
            }}
            placeholder="Enter URL..."
          />
        </form>

        {/* Loading Indicator */}
        {navigationState.isLoading && (
          <div
            style={{
              width: '16px',
              height: '16px',
              border: '2px solid #45475a',
              borderTopColor: '#89b4fa',
              borderRadius: '50%',
              animation: 'spin 1s linear infinite',
            }}
          />
        )}
      </div>

      {/* Title Bar (optional) */}
      <div
        style={{
          padding: '4px 12px',
          backgroundColor: '#1e1e2e',
          borderBottom: '1px solid #45475a',
          color: '#a6adc8',
          fontSize: '12px',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
        }}
      >
        {title}
      </div>

      {/* Content Area */}
      <div style={{ flex: 1, position: 'relative' }}>
        {navigationState.error ? (
          <div
            style={{
              position: 'absolute',
              top: '50%',
              left: '50%',
              transform: 'translate(-50%, -50%)',
              textAlign: 'center',
              color: '#f38ba8',
            }}
          >
            <div style={{ fontSize: '48px', marginBottom: '16px' }}>⚠</div>
            <div style={{ fontSize: '18px', marginBottom: '8px' }}>{navigationState.error}</div>
            <div style={{ fontSize: '14px', color: '#6c7086' }}>
              {navigationState.url}
            </div>
            <button
              onClick={reload}
              style={{
                marginTop: '16px',
                padding: '8px 16px',
                backgroundColor: '#89b4fa',
                border: 'none',
                borderRadius: '4px',
                color: '#1e1e2e',
                cursor: 'pointer',
              }}
            >
              Retry
            </button>
          </div>
        ) : (
          <iframe
            ref={iframeRef}
            src={navigationState.url}
            onLoad={handleLoad}
            onError={handleError}
            style={{
              width: '100%',
              height: '100%',
              border: 'none',
              backgroundColor: '#fff',
            }}
            title={title}
            sandbox="allow-same-origin allow-scripts allow-forms allow-popups allow-modals"
          />
        )}
      </div>

      {/* Inline styles for animation */}
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

export default AppPreview;
