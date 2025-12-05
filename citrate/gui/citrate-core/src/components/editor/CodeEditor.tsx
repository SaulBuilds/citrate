/**
 * Code Editor Component
 *
 * Sprint 5: Multi-Window & Terminal Integration
 *
 * Monaco-based code editor with syntax highlighting and AI suggestions.
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import Editor, { OnMount, OnChange } from '@monaco-editor/react';
import type { editor as MonacoEditor } from 'monaco-editor';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface EditorFile {
  path: string;
  content: string;
  language: string;
  isDirty: boolean;
}

interface EditorSuggestion {
  path: string;
  range: {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  };
  suggestion: string;
  description?: string;
}

interface CodeEditorProps {
  /** Initial file to open */
  initialFile?: {
    path: string;
    content: string;
    language?: string;
  };
  /** Callback when content changes */
  onChange?: (content: string, isDirty: boolean) => void;
  /** Callback when file is saved */
  onSave?: (path: string, content: string) => void;
  /** Custom CSS class */
  className?: string;
}

// Detect language from file extension
function detectLanguage(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase();
  const languageMap: Record<string, string> = {
    js: 'javascript',
    jsx: 'javascript',
    ts: 'typescript',
    tsx: 'typescript',
    py: 'python',
    rs: 'rust',
    sol: 'solidity',
    json: 'json',
    md: 'markdown',
    html: 'html',
    css: 'css',
    scss: 'scss',
    yaml: 'yaml',
    yml: 'yaml',
    toml: 'toml',
    sh: 'shell',
    bash: 'shell',
    sql: 'sql',
    go: 'go',
    java: 'java',
    c: 'c',
    cpp: 'cpp',
    h: 'c',
    hpp: 'cpp',
  };
  return languageMap[ext || ''] || 'plaintext';
}

export function CodeEditor({
  initialFile,
  onChange,
  onSave,
  className = '',
}: CodeEditorProps) {
  const editorRef = useRef<MonacoEditor.IStandaloneCodeEditor | null>(null);
  const [files, setFiles] = useState<EditorFile[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);
  const [pendingSuggestion, setPendingSuggestion] = useState<EditorSuggestion | null>(null);

  // Initialize with initial file
  useEffect(() => {
    if (initialFile) {
      const file: EditorFile = {
        path: initialFile.path,
        content: initialFile.content,
        language: initialFile.language || detectLanguage(initialFile.path),
        isDirty: false,
      };
      setFiles([file]);
    }
  }, [initialFile]);

  // Handle Monaco editor mount
  const handleEditorMount: OnMount = useCallback((editor) => {
    editorRef.current = editor;

    // Add keybinding for save
    editor.addCommand(
      // Monaco KeyMod.CtrlCmd | Monaco KeyCode.KeyS
      2048 | 49, // Ctrl/Cmd + S
      () => {
        const currentFile = files[activeIndex];
        if (currentFile) {
          handleSave();
        }
      }
    );
  }, [files, activeIndex]);

  // Handle content change
  const handleChange: OnChange = useCallback((value) => {
    if (value !== undefined) {
      setFiles((prev) => {
        const updated = [...prev];
        if (updated[activeIndex]) {
          const originalContent = initialFile?.content || '';
          updated[activeIndex] = {
            ...updated[activeIndex],
            content: value,
            isDirty: value !== originalContent,
          };
        }
        return updated;
      });

      const isDirty = value !== (initialFile?.content || '');
      onChange?.(value, isDirty);
    }
  }, [activeIndex, initialFile, onChange]);

  // Save file
  const handleSave = useCallback(async () => {
    const currentFile = files[activeIndex];
    if (!currentFile) return;

    try {
      // Try to write file via Tauri
      await invoke('write_file', {
        path: currentFile.path,
        content: currentFile.content,
      }).catch(() => {
        // Fallback: just notify via callback
        console.log('File save via invoke not available, using callback');
      });

      // Mark as saved
      setFiles((prev) => {
        const updated = [...prev];
        if (updated[activeIndex]) {
          updated[activeIndex] = {
            ...updated[activeIndex],
            isDirty: false,
          };
        }
        return updated;
      });

      onSave?.(currentFile.path, currentFile.content);
    } catch (err) {
      console.error('Failed to save file:', err);
    }
  }, [files, activeIndex, onSave]);

  // Open file
  const openFile = useCallback((path: string, content: string, language?: string) => {
    // Check if file is already open
    const existingIndex = files.findIndex((f) => f.path === path);
    if (existingIndex >= 0) {
      setActiveIndex(existingIndex);
      return;
    }

    // Add new file
    const file: EditorFile = {
      path,
      content,
      language: language || detectLanguage(path),
      isDirty: false,
    };

    setFiles((prev) => [...prev, file]);
    setActiveIndex(files.length);
  }, [files]);

  // Close file
  const closeFile = useCallback((index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
    if (activeIndex >= index && activeIndex > 0) {
      setActiveIndex((prev) => prev - 1);
    }
  }, [activeIndex]);

  // Apply suggestion
  const applySuggestion = useCallback(() => {
    if (!pendingSuggestion || !editorRef.current) return;

    const editor = editorRef.current;
    const model = editor.getModel();
    if (!model) return;

    const { range, suggestion } = pendingSuggestion;

    // Apply edit
    editor.executeEdits('ai-suggestion', [
      {
        range: {
          startLineNumber: range.startLine,
          startColumn: range.startColumn,
          endLineNumber: range.endLine,
          endColumn: range.endColumn,
        },
        text: suggestion,
      },
    ]);

    setPendingSuggestion(null);
  }, [pendingSuggestion]);

  // Reject suggestion
  const rejectSuggestion = useCallback(() => {
    setPendingSuggestion(null);
  }, []);

  // Listen for open file events
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setup = async () => {
      unlisten = await listen<{ path: string; content?: string; language?: string }>(
        'editor:open',
        async (event) => {
          const { path, content, language } = event.payload;

          // If content not provided, try to read file
          let fileContent = content;
          if (!fileContent) {
            try {
              fileContent = await invoke<string>('read_file', { path });
            } catch {
              fileContent = '';
            }
          }

          openFile(path, fileContent, language);
        }
      );
    };

    setup();

    return () => {
      unlisten?.();
    };
  }, [openFile]);

  // Listen for suggestion events
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setup = async () => {
      unlisten = await listen<EditorSuggestion>('editor:suggest', (event) => {
        // Only show suggestion if it's for the active file
        const currentFile = files[activeIndex];
        if (currentFile && event.payload.path === currentFile.path) {
          setPendingSuggestion(event.payload);
        }
      });
    };

    setup();

    return () => {
      unlisten?.();
    };
  }, [files, activeIndex]);

  const currentFile = files[activeIndex];

  return (
    <div
      className={`editor-container ${className}`}
      style={{
        display: 'flex',
        flexDirection: 'column',
        width: '100%',
        height: '100%',
        backgroundColor: '#1e1e2e',
      }}
    >
      {/* Tab Bar */}
      <div
        style={{
          display: 'flex',
          backgroundColor: '#181825',
          borderBottom: '1px solid #313244',
          overflowX: 'auto',
        }}
      >
        {files.map((file, index) => (
          <div
            key={file.path}
            onClick={() => setActiveIndex(index)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              padding: '8px 16px',
              backgroundColor: index === activeIndex ? '#1e1e2e' : 'transparent',
              borderRight: '1px solid #313244',
              cursor: 'pointer',
              color: index === activeIndex ? '#cdd6f4' : '#a6adc8',
              fontSize: '13px',
            }}
          >
            <span>
              {file.isDirty && '● '}
              {file.path.split('/').pop()}
            </span>
            <button
              onClick={(e) => {
                e.stopPropagation();
                closeFile(index);
              }}
              style={{
                background: 'none',
                border: 'none',
                color: '#6c7086',
                cursor: 'pointer',
                padding: '2px',
                fontSize: '14px',
              }}
            >
              ×
            </button>
          </div>
        ))}
      </div>

      {/* Editor */}
      <div style={{ flex: 1, position: 'relative' }}>
        {currentFile ? (
          <Editor
            height="100%"
            language={currentFile.language}
            value={currentFile.content}
            theme="vs-dark"
            onMount={handleEditorMount}
            onChange={handleChange}
            options={{
              minimap: { enabled: true },
              fontSize: 14,
              fontFamily: '"Fira Code", "Cascadia Code", Consolas, monospace',
              fontLigatures: true,
              lineNumbers: 'on',
              renderWhitespace: 'selection',
              tabSize: 2,
              wordWrap: 'on',
              automaticLayout: true,
              scrollBeyondLastLine: false,
              bracketPairColorization: { enabled: true },
            }}
          />
        ) : (
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              color: '#6c7086',
            }}
          >
            <div style={{ fontSize: '48px', marginBottom: '16px' }}>📝</div>
            <div style={{ fontSize: '16px' }}>No file open</div>
            <div style={{ fontSize: '13px', marginTop: '8px' }}>
              Open a file to start editing
            </div>
          </div>
        )}

        {/* AI Suggestion Banner */}
        {pendingSuggestion && (
          <div
            style={{
              position: 'absolute',
              bottom: '16px',
              left: '50%',
              transform: 'translateX(-50%)',
              backgroundColor: '#313244',
              border: '1px solid #89b4fa',
              borderRadius: '8px',
              padding: '12px 16px',
              boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
              maxWidth: '500px',
            }}
          >
            <div style={{ color: '#89b4fa', fontSize: '12px', marginBottom: '8px' }}>
              AI Suggestion
            </div>
            {pendingSuggestion.description && (
              <div style={{ color: '#cdd6f4', fontSize: '13px', marginBottom: '12px' }}>
                {pendingSuggestion.description}
              </div>
            )}
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                onClick={applySuggestion}
                style={{
                  padding: '6px 16px',
                  backgroundColor: '#a6e3a1',
                  border: 'none',
                  borderRadius: '4px',
                  color: '#1e1e2e',
                  cursor: 'pointer',
                  fontSize: '13px',
                }}
              >
                Apply
              </button>
              <button
                onClick={rejectSuggestion}
                style={{
                  padding: '6px 16px',
                  backgroundColor: '#45475a',
                  border: 'none',
                  borderRadius: '4px',
                  color: '#cdd6f4',
                  cursor: 'pointer',
                  fontSize: '13px',
                }}
              >
                Dismiss
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Status Bar */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: '4px 12px',
          backgroundColor: '#313244',
          borderTop: '1px solid #45475a',
          color: '#a6adc8',
          fontSize: '12px',
        }}
      >
        <div>
          {currentFile?.path || 'No file'}
        </div>
        <div style={{ display: 'flex', gap: '16px' }}>
          <span>{currentFile?.language || 'Plain Text'}</span>
          {currentFile?.isDirty && <span style={{ color: '#f9e2af' }}>Modified</span>}
        </div>
      </div>
    </div>
  );
}

export default CodeEditor;
