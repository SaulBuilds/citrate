/**
 * ErrorBoundary Test Component
 *
 * This component is used to test the ErrorBoundary functionality.
 * It provides buttons to trigger different types of errors.
 *
 * Usage:
 * 1. Import this component into App.tsx or any component
 * 2. Add it to the render tree inside ErrorBoundary
 * 3. Click the test buttons to verify error boundary catches errors
 * 4. Remove or comment out after testing
 */

import { useState } from 'react';
import { AlertTriangle, Bomb, Bug } from 'lucide-react';

export function ErrorBoundaryTest() {
  const [shouldThrow, setShouldThrow] = useState(false);

  // This will throw an error during render
  if (shouldThrow) {
    throw new Error('Test error: This is an intentional error to test ErrorBoundary');
  }

  // Function to throw error in event handler
  const throwEventError = () => {
    throw new Error('Test error: Error thrown from event handler');
  };

  // Function to throw error asynchronously
  const throwAsyncError = () => {
    setTimeout(() => {
      throw new Error('Test error: Async error (this will NOT be caught by ErrorBoundary)');
    }, 100);
  };

  return (
    <div style={{
      position: 'fixed',
      bottom: '20px',
      right: '20px',
      background: '#fff3cd',
      border: '2px solid #ffc107',
      borderRadius: '8px',
      padding: '1rem',
      boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
      zIndex: 9999,
      maxWidth: '300px'
    }}>
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
        marginBottom: '0.75rem'
      }}>
        <AlertTriangle size={20} color="#ff6b00" />
        <strong style={{ color: '#856404' }}>ErrorBoundary Test</strong>
      </div>

      <p style={{
        fontSize: '0.875rem',
        color: '#856404',
        margin: '0 0 1rem 0'
      }}>
        Click buttons to test error handling
      </p>

      <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: '0.5rem'
      }}>
        <button
          onClick={() => setShouldThrow(true)}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            padding: '0.5rem 1rem',
            background: '#dc3545',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
            fontWeight: '600'
          }}
        >
          <Bomb size={16} />
          Throw Render Error
        </button>

        <button
          onClick={throwEventError}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            padding: '0.5rem 1rem',
            background: '#fd7e14',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
            fontWeight: '600'
          }}
        >
          <Bug size={16} />
          Throw Event Error
        </button>

        <button
          onClick={throwAsyncError}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            padding: '0.5rem 1rem',
            background: '#6c757d',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '0.875rem',
            fontWeight: '600'
          }}
        >
          <AlertTriangle size={16} />
          Throw Async Error
        </button>
      </div>

      <p style={{
        fontSize: '0.75rem',
        color: '#856404',
        margin: '0.75rem 0 0 0',
        fontStyle: 'italic'
      }}>
        Note: Async errors won't be caught by ErrorBoundary
      </p>
    </div>
  );
}

export default ErrorBoundaryTest;
