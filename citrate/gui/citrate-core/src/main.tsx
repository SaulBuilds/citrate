import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

// Debug: Log when main.tsx starts executing
console.log('[Citrate] main.tsx loading...');

const rootElement = document.getElementById("root");

if (!rootElement) {
  console.error('[Citrate] ERROR: Root element not found!');
  document.body.innerHTML = '<div style="padding: 20px; color: red;">Error: Root element not found</div>';
} else {
  console.log('[Citrate] Root element found, creating React root...');

  try {
    const root = ReactDOM.createRoot(rootElement);
    console.log('[Citrate] React root created, rendering App...');

    root.render(
      <React.StrictMode>
        <App />
      </React.StrictMode>,
    );

    console.log('[Citrate] App rendered successfully');
  } catch (error) {
    console.error('[Citrate] ERROR rendering App:', error);
    rootElement.innerHTML = `
      <div style="padding: 20px; color: red; font-family: system-ui;">
        <h2>React Initialization Error</h2>
        <pre style="background: #fee; padding: 10px; border-radius: 4px; overflow: auto;">${error instanceof Error ? error.stack : String(error)}</pre>
      </div>
    `;
  }
}
