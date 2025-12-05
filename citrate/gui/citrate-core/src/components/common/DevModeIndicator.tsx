/**
 * Dev Mode Indicator Component
 *
 * Displays a visual indicator when the app is running in development mode.
 * This helps developers quickly identify if they're using a dev or prod build.
 *
 * In production builds, this component renders nothing (null).
 */

import React, { useState } from 'react';
import { isDevMode, getBuildInfo, DEV_FEATURES } from '../../utils/devMode';

// ============================================================================
// Styles
// ============================================================================

const styles = {
  container: {
    position: 'fixed' as const,
    bottom: '16px',
    left: '16px',
    zIndex: 9998,
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
  },
  badge: {
    backgroundColor: '#f59e0b',
    color: '#1e1e1e',
    padding: '4px 12px',
    borderRadius: '4px',
    fontSize: '11px',
    fontWeight: 700,
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    cursor: 'pointer',
    boxShadow: '0 2px 8px rgba(245, 158, 11, 0.4)',
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    transition: 'transform 0.2s, box-shadow 0.2s',
  },
  badgeHover: {
    transform: 'scale(1.05)',
    boxShadow: '0 4px 12px rgba(245, 158, 11, 0.5)',
  },
  dot: {
    width: '6px',
    height: '6px',
    borderRadius: '50%',
    backgroundColor: '#1e1e1e',
    animation: 'pulse 2s infinite',
  },
  popup: {
    backgroundColor: '#1e1e2e',
    border: '1px solid #333',
    borderRadius: '8px',
    padding: '12px 16px',
    boxShadow: '0 4px 16px rgba(0, 0, 0, 0.3)',
    minWidth: '200px',
  },
  popupTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#fff',
    marginBottom: '8px',
    paddingBottom: '8px',
    borderBottom: '1px solid #333',
  },
  popupRow: {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '11px',
    color: '#a0a0a0',
    marginBottom: '4px',
  },
  popupLabel: {
    color: '#666',
  },
  popupValue: {
    color: '#fff',
    fontFamily: 'monospace',
  },
  featureList: {
    marginTop: '8px',
    paddingTop: '8px',
    borderTop: '1px solid #333',
  },
  featureTitle: {
    fontSize: '11px',
    fontWeight: 600,
    color: '#666',
    marginBottom: '6px',
  },
  featureItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '10px',
    color: '#a0a0a0',
    marginBottom: '2px',
  },
  featureEnabled: {
    color: '#22c55e',
  },
  featureDisabled: {
    color: '#666',
  },
};

// ============================================================================
// Component
// ============================================================================

interface DevModeIndicatorProps {
  /** Show expanded info by default */
  expanded?: boolean;
  /** Position override */
  position?: 'bottom-left' | 'bottom-right' | 'top-left' | 'top-right';
}

export function DevModeIndicator({
  expanded = false,
  position = 'bottom-left',
}: DevModeIndicatorProps) {
  const [showInfo, setShowInfo] = useState(expanded);
  const [isHovered, setIsHovered] = useState(false);

  // Don't render anything in production
  if (!isDevMode()) {
    return null;
  }

  const buildInfo = getBuildInfo();

  // Calculate position styles
  const positionStyles: React.CSSProperties = {
    ...styles.container,
    ...(position === 'bottom-right' && { left: 'auto', right: '16px' }),
    ...(position === 'top-left' && { bottom: 'auto', top: '16px' }),
    ...(position === 'top-right' && { bottom: 'auto', top: '16px', left: 'auto', right: '16px' }),
  };

  return (
    <div style={positionStyles}>
      <style>
        {`
          @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
          }
        `}
      </style>

      {/* Expandable Info Popup */}
      {showInfo && (
        <div style={styles.popup}>
          <div style={styles.popupTitle}>Development Build</div>

          <div style={styles.popupRow}>
            <span style={styles.popupLabel}>Version:</span>
            <span style={styles.popupValue}>{buildInfo.version}</span>
          </div>

          <div style={styles.popupRow}>
            <span style={styles.popupLabel}>Built:</span>
            <span style={styles.popupValue}>
              {new Date(buildInfo.buildTime).toLocaleDateString()}
            </span>
          </div>

          <div style={styles.popupRow}>
            <span style={styles.popupLabel}>Mode:</span>
            <span style={{ ...styles.popupValue, color: '#f59e0b' }}>
              {buildInfo.environment}
            </span>
          </div>

          <div style={styles.featureList}>
            <div style={styles.featureTitle}>Dev Features</div>
            {Object.entries(DEV_FEATURES).map(([feature, enabled]) => (
              <div key={feature} style={styles.featureItem}>
                <span style={enabled ? styles.featureEnabled : styles.featureDisabled}>
                  {enabled ? '●' : '○'}
                </span>
                <span>{feature.replace(/_/g, ' ')}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Badge */}
      <div
        style={{
          ...styles.badge,
          ...(isHovered ? styles.badgeHover : {}),
        }}
        onClick={() => setShowInfo(!showInfo)}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => e.key === 'Enter' && setShowInfo(!showInfo)}
        aria-label="Development mode indicator"
        aria-expanded={showInfo}
      >
        <span style={styles.dot} />
        DEV MODE
      </div>
    </div>
  );
}

export default DevModeIndicator;
