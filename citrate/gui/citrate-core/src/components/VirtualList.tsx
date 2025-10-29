/**
 * VirtualList Component
 *
 * Simple virtualized list implementation for efficient rendering of large lists.
 * Only renders visible items to improve performance with 1000+ items.
 *
 * Note: This is a stub implementation. For production use, install react-window:
 * npm install react-window @types/react-window
 */

import React, { useState, useRef, CSSProperties } from 'react';

/**
 * Props for FixedSizeVirtualList
 */
export interface FixedSizeVirtualListProps<T> {
  /** Array of items to render */
  items: T[];
  /** Height of each item in pixels */
  itemHeight: number;
  /** Total height of the list container */
  height: number;
  /** Optional width (defaults to '100%') */
  width?: string | number;
  /** Number of items to render outside visible area (default: 5) */
  overscanCount?: number;
  /** Render function for each item */
  renderItem: (item: T, index: number, style: React.CSSProperties) => React.ReactNode;
  /** Optional className for the list container */
  className?: string;
}

/**
 * FixedSizeVirtualList - For items with uniform height
 *
 * Use this when all items have the same height.
 * Most performant option for uniform lists.
 *
 * @example
 * ```tsx
 * <FixedSizeVirtualList
 *   items={transactions}
 *   itemHeight={80}
 *   height={600}
 *   renderItem={(tx, index, style) => (
 *     <div style={style} key={tx.hash}>
 *       {tx.hash} - {tx.amount}
 *     </div>
 *   )}
 * />
 * ```
 */
export function FixedSizeVirtualList<T>({
  items,
  itemHeight,
  height,
  width = '100%',
  overscanCount = 5,
  renderItem,
  className,
}: FixedSizeVirtualListProps<T>) {
  const [scrollTop, setScrollTop] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);

  // Calculate visible range
  const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscanCount);
  const endIndex = Math.min(
    items.length,
    Math.ceil((scrollTop + height) / itemHeight) + overscanCount
  );

  const visibleItems = items.slice(startIndex, endIndex);
  const totalHeight = items.length * itemHeight;

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  };

  return (
    <div
      ref={containerRef}
      className={className}
      onScroll={handleScroll}
      style={{
        height,
        width,
        overflow: 'auto',
        position: 'relative',
      }}
    >
      <div style={{ height: totalHeight, position: 'relative' }}>
        {visibleItems.map((item, i) => {
          const index = startIndex + i;
          const style: CSSProperties = {
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: itemHeight,
            transform: `translateY(${index * itemHeight}px)`,
          };
          return <div key={index}>{renderItem(item, index, style)}</div>;
        })}
      </div>
    </div>
  );
}

/**
 * Props for VariableSizeVirtualList
 */
export interface VariableSizeVirtualListProps<T> {
  /** Array of items to render */
  items: T[];
  /** Function that returns height for each item */
  getItemHeight: (index: number) => number;
  /** Total height of the list container */
  height: number;
  /** Optional width (defaults to '100%') */
  width?: string | number;
  /** Number of items to render outside visible area (default: 5) */
  overscanCount?: number;
  /** Render function for each item */
  renderItem: (item: T, index: number, style: React.CSSProperties) => React.ReactNode;
  /** Optional className for the list container */
  className?: string;
}

/**
 * VariableSizeVirtualList - For items with variable heights
 *
 * Use this when items have different heights.
 * Slightly less performant than FixedSizeList but more flexible.
 *
 * @example
 * ```tsx
 * <VariableSizeVirtualList
 *   items={messages}
 *   getItemHeight={(index) => messages[index].expanded ? 200 : 100}
 *   height={600}
 *   renderItem={(msg, index, style) => (
 *     <div style={style} key={msg.id}>
 *       {msg.content}
 *     </div>
 *   )}
 * />
 * ```
 */
export function VariableSizeVirtualList<T>({
  items,
  getItemHeight,
  height,
  width = '100%',
  overscanCount = 5,
  renderItem,
  className,
}: VariableSizeVirtualListProps<T>) {
  const [scrollTop, setScrollTop] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);

  // Calculate heights for all items (cached)
  const itemHeights = items.map((_, i) => getItemHeight(i));
  const itemOffsets = itemHeights.reduce<number[]>((acc, h) => {
    acc.push((acc[acc.length - 1] || 0) + h);
    return acc;
  }, []);

  // Find visible range
  const startIndex = Math.max(
    0,
    itemOffsets.findIndex((offset) => offset > scrollTop) - overscanCount
  );
  const endIndex = Math.min(
    items.length,
    itemOffsets.findIndex((offset) => offset > scrollTop + height) + overscanCount
  );

  const visibleItems = items.slice(startIndex, endIndex);
  const totalHeight = itemOffsets[itemOffsets.length - 1] || 0;

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  };

  return (
    <div
      ref={containerRef}
      className={className}
      onScroll={handleScroll}
      style={{
        height,
        width,
        overflow: 'auto',
        position: 'relative',
      }}
    >
      <div style={{ height: totalHeight, position: 'relative' }}>
        {visibleItems.map((item, i) => {
          const index = startIndex + i;
          const itemHeight = itemHeights[index];
          const offsetY = index > 0 ? itemOffsets[index - 1] : 0;
          const style: CSSProperties = {
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: itemHeight,
            transform: `translateY(${offsetY}px)`,
          };
          return <div key={index}>{renderItem(item, index, style)}</div>;
        })}
      </div>
    </div>
  );
}

/**
 * Default export for convenience
 */
export default FixedSizeVirtualList;
