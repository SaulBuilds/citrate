/**
 * Window Types for Multi-Window Management
 *
 * Sprint 5: Multi-Window & Terminal Integration
 */

/** Window type identifiers */
export type WindowType = 'main' | 'terminal' | 'preview' | 'editor';

/** Window state */
export interface WindowState {
  /** Unique window identifier */
  id: string;
  /** Type of window */
  type: WindowType;
  /** Window title */
  title: string;
  /** Whether window is currently open */
  isOpen: boolean;
  /** Whether window has focus */
  isFocused: boolean;
  /** Window position (if known) */
  position?: WindowPosition;
  /** Window size (if known) */
  size?: WindowSize;
  /** Window-specific data */
  data?: WindowData;
  /** Creation timestamp */
  createdAt: number;
}

/** Window position */
export interface WindowPosition {
  x: number;
  y: number;
}

/** Window size */
export interface WindowSize {
  width: number;
  height: number;
}

/** Window-specific data by type */
export type WindowData =
  | TerminalWindowData
  | PreviewWindowData
  | EditorWindowData
  | MainWindowData;

/** Terminal window data */
export interface TerminalWindowData {
  type: 'terminal';
  sessionId: string;
  cwd: string;
  shell?: string;
}

/** Preview window data */
export interface PreviewWindowData {
  type: 'preview';
  url: string;
  canGoBack: boolean;
  canGoForward: boolean;
}

/** Editor window data */
export interface EditorWindowData {
  type: 'editor';
  files: EditorFile[];
  activeFile: string | null;
}

/** Editor file */
export interface EditorFile {
  path: string;
  content: string;
  language: string;
  isDirty: boolean;
}

/** Main window data */
export interface MainWindowData {
  type: 'main';
}

/** Window creation options */
export interface CreateWindowOptions {
  /** Window type */
  type: WindowType;
  /** Optional title (auto-generated if not provided) */
  title?: string;
  /** Initial position */
  position?: WindowPosition;
  /** Initial size */
  size?: WindowSize;
  /** Window-specific data */
  data?: Partial<WindowData>;
}

/** Default window sizes by type */
export const DEFAULT_WINDOW_SIZES: Record<WindowType, WindowSize> = {
  main: { width: 1200, height: 800 },
  terminal: { width: 800, height: 500 },
  preview: { width: 1024, height: 768 },
  editor: { width: 1000, height: 700 },
};

/** Default window titles by type */
export const DEFAULT_WINDOW_TITLES: Record<WindowType, string> = {
  main: 'Citrate',
  terminal: 'Terminal',
  preview: 'App Preview',
  editor: 'Code Editor',
};

/** Window event types */
export type WindowEventType =
  | 'window:opened'
  | 'window:closed'
  | 'window:focused'
  | 'window:blurred'
  | 'window:moved'
  | 'window:resized';

/** Window event payload */
export interface WindowEvent {
  type: WindowEventType;
  windowId: string;
  timestamp: number;
  data?: unknown;
}

/** Window manager state */
export interface WindowManagerState {
  windows: WindowState[];
  activeWindowId: string | null;
  mainWindowId: string;
}

/** Window context type */
export interface WindowContextType {
  /** Current window manager state */
  state: WindowManagerState;

  /** Open a new window */
  openWindow: (options: CreateWindowOptions) => Promise<string>;

  /** Close a window */
  closeWindow: (id: string) => Promise<void>;

  /** Focus a window */
  focusWindow: (id: string) => Promise<void>;

  /** Get window by ID */
  getWindow: (id: string) => WindowState | undefined;

  /** Get windows by type */
  getWindowsByType: (type: WindowType) => WindowState[];

  /** Update window data */
  updateWindowData: (id: string, data: Partial<WindowData>) => void;

  /** Check if a window type is open */
  hasOpenWindow: (type: WindowType) => boolean;
}
