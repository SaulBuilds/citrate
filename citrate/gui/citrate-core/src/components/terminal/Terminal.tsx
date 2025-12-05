/**
 * Terminal Component
 *
 * Sprint 5: Multi-Window & Terminal Integration
 *
 * xterm.js-based terminal that connects to PTY backend.
 */

import { useEffect, useRef, useCallback, useState } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import '@xterm/xterm/css/xterm.css';

interface TerminalOutput {
  session_id: string;
  data: string; // Base64 encoded
  timestamp: number;
}

interface TerminalInfo {
  session_id: string;
  shell: string;
  cwd: string;
  cols: number;
  rows: number;
  created_at: number;
  is_active: boolean;
}

interface TerminalProps {
  /** Optional session ID to reconnect to */
  sessionId?: string;
  /** Initial working directory */
  cwd?: string;
  /** Custom shell to use */
  shell?: string;
  /** Callback when terminal is ready */
  onReady?: (sessionId: string) => void;
  /** Callback when terminal is closed */
  onClose?: () => void;
  /** Custom CSS class */
  className?: string;
}

export function Terminal({
  sessionId: initialSessionId,
  cwd,
  shell,
  onReady,
  onClose,
  className = '',
}: TerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(initialSessionId || null);
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Initialize terminal
  useEffect(() => {
    if (!containerRef.current) return;

    // Create terminal instance
    const term = new XTerm({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: '"Fira Code", "Cascadia Code", "JetBrains Mono", Consolas, monospace',
      theme: {
        background: '#1e1e2e',
        foreground: '#cdd6f4',
        cursor: '#f5e0dc',
        cursorAccent: '#1e1e2e',
        selectionBackground: '#585b70',
        black: '#45475a',
        red: '#f38ba8',
        green: '#a6e3a1',
        yellow: '#f9e2af',
        blue: '#89b4fa',
        magenta: '#f5c2e7',
        cyan: '#94e2d5',
        white: '#bac2de',
        brightBlack: '#585b70',
        brightRed: '#f38ba8',
        brightGreen: '#a6e3a1',
        brightYellow: '#f9e2af',
        brightBlue: '#89b4fa',
        brightMagenta: '#f5c2e7',
        brightCyan: '#94e2d5',
        brightWhite: '#a6adc8',
      },
      allowProposedApi: true,
    });

    // Add addons
    const fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon();

    term.loadAddon(fitAddon);
    term.loadAddon(webLinksAddon);

    // Open terminal in container
    term.open(containerRef.current);
    fitAddon.fit();

    terminalRef.current = term;
    fitAddonRef.current = fitAddon;

    // Handle resize
    const handleResize = () => {
      if (fitAddonRef.current) {
        fitAddonRef.current.fit();
      }
    };

    const resizeObserver = new ResizeObserver(handleResize);
    resizeObserver.observe(containerRef.current);

    window.addEventListener('resize', handleResize);

    // Cleanup
    return () => {
      resizeObserver.disconnect();
      window.removeEventListener('resize', handleResize);
      term.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, []);

  // Create or connect to session
  useEffect(() => {
    if (!terminalRef.current) return;

    let cleanup: (() => void) | undefined;
    let outputUnlisten: UnlistenFn | undefined;
    let closedUnlisten: UnlistenFn | undefined;

    const connectSession = async () => {
      try {
        let sid = sessionId;

        // Create new session if we don't have one
        if (!sid) {
          const info = await invoke<TerminalInfo>('terminal_create', {
            args: {
              shell,
              cwd,
              cols: terminalRef.current?.cols || 80,
              rows: terminalRef.current?.rows || 24,
            },
          });
          sid = info.session_id;
          setSessionId(sid);
        }

        // Listen for output
        outputUnlisten = await listen<TerminalOutput>('terminal-output', (event) => {
          if (event.payload.session_id === sid && terminalRef.current) {
            // Decode base64 data
            const decoded = atob(event.payload.data);
            terminalRef.current.write(decoded);
          }
        });

        // Listen for close events
        closedUnlisten = await listen<{ session_id: string }>('terminal-closed', (event) => {
          if (event.payload.session_id === sid) {
            setIsConnected(false);
            if (terminalRef.current) {
              terminalRef.current.write('\r\n\x1b[31m[Session closed]\x1b[0m\r\n');
            }
            onClose?.();
          }
        });

        // Handle user input
        const inputDisposable = terminalRef.current!.onData(async (data) => {
          if (sid) {
            try {
              await invoke('terminal_write', { sessionId: sid, data });
            } catch (err) {
              console.error('Failed to write to terminal:', err);
            }
          }
        });

        // Handle resize
        const resizeDisposable = terminalRef.current!.onResize(async ({ cols, rows }) => {
          if (sid) {
            try {
              await invoke('terminal_resize', { sessionId: sid, cols, rows });
            } catch (err) {
              console.error('Failed to resize terminal:', err);
            }
          }
        });

        setIsConnected(true);
        setError(null);
        onReady?.(sid);

        cleanup = () => {
          inputDisposable.dispose();
          resizeDisposable.dispose();
        };
      } catch (err) {
        console.error('Failed to create terminal session:', err);
        setError(String(err));
        if (terminalRef.current) {
          terminalRef.current.write(`\r\n\x1b[31mError: ${err}\x1b[0m\r\n`);
        }
      }
    };

    connectSession();

    // Cleanup on unmount
    return () => {
      cleanup?.();
      outputUnlisten?.();
      closedUnlisten?.();

      // Close session when component unmounts
      if (sessionId) {
        invoke('terminal_close', { sessionId }).catch(console.error);
      }
    };
  }, [sessionId, cwd, shell, onReady, onClose]);

  // Focus terminal on click
  const handleClick = useCallback(() => {
    terminalRef.current?.focus();
  }, []);

  return (
    <div
      className={`terminal-container ${className}`}
      style={{
        width: '100%',
        height: '100%',
        backgroundColor: '#1e1e2e',
        position: 'relative',
      }}
      onClick={handleClick}
    >
      <div
        ref={containerRef}
        style={{
          width: '100%',
          height: '100%',
          padding: '4px',
        }}
      />
      {error && (
        <div
          style={{
            position: 'absolute',
            bottom: '8px',
            left: '8px',
            right: '8px',
            padding: '8px',
            backgroundColor: 'rgba(243, 139, 168, 0.2)',
            borderRadius: '4px',
            color: '#f38ba8',
            fontSize: '12px',
          }}
        >
          {error}
        </div>
      )}
      {!isConnected && !error && (
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            color: '#cdd6f4',
            fontSize: '14px',
          }}
        >
          Connecting...
        </div>
      )}
    </div>
  );
}

export default Terminal;
