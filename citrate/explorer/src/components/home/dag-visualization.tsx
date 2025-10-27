'use client';

import { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';

interface DagNode {
  id: string;
  number: string;
  timestamp: string;
  blueScore: string;
  isBlue: boolean;
  txCount: number;
}

interface DagLink {
  source: string;
  target: string;
  type: 'parent' | 'selected' | 'merge';
}

export function DagVisualization() {
  const svgRef = useRef<SVGSVGElement>(null);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });

  const { data, isLoading } = useQuery({
    queryKey: ['dag'],
    queryFn: async () => {
      const response = await axios.get('/api/dag');
      return response.data;
    },
    refetchInterval: 10000
  });

  useEffect(() => {
    const handleResize = () => {
      if (svgRef.current?.parentElement) {
        const { width } = svgRef.current.parentElement.getBoundingClientRect();
        setDimensions({ width, height: 600 });
      }
    };

    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  useEffect(() => {
    if (!data || !svgRef.current) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    const { width, height } = dimensions;
    const margin = { top: 20, right: 20, bottom: 20, left: 20 };
    const innerWidth = width - margin.left - margin.right;
    const innerHeight = height - margin.top - margin.bottom;

    // Create container
    const g = svg
      .append('g')
      .attr('transform', `translate(${margin.left},${margin.top})`);

    // Create zoom behavior
    const zoom = d3.zoom()
      .scaleExtent([0.5, 3])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
      });

    svg.call(zoom as any);

    // Create force simulation
    const simulation = d3.forceSimulation(data.nodes)
      .force('link', d3.forceLink(data.links)
        .id((d: any) => d.id)
        .distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(innerWidth / 2, innerHeight / 2))
      .force('collision', d3.forceCollide().radius(30));

    // Create arrows for directed edges
    svg.append('defs').selectAll('marker')
      .data(['parent', 'selected', 'merge'])
      .enter().append('marker')
      .attr('id', d => `arrow-${d}`)
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 20)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', d => {
        switch (d) {
          case 'selected': return '#3b82f6';
          case 'merge': return '#8b5cf6';
          default: return '#6b7280';
        }
      });

    // Create links
    const link = g.append('g')
      .selectAll('line')
      .data(data.links)
      .enter().append('line')
      .attr('class', 'dag-link')
      .attr('stroke', (d: DagLink) => {
        switch (d.type) {
          case 'selected': return '#3b82f6';
          case 'merge': return '#8b5cf6';
          default: return '#6b7280';
        }
      })
      .attr('stroke-width', (d: DagLink) => d.type === 'selected' ? 2 : 1)
      .attr('marker-end', (d: DagLink) => `url(#arrow-${d.type})`);

    // Create nodes
    const node = g.append('g')
      .selectAll('g')
      .data(data.nodes)
      .enter().append('g')
      .attr('class', 'dag-node')
      .call(d3.drag()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended) as any);

    // Add circles for nodes
    node.append('circle')
      .attr('r', (d: DagNode) => 10 + Math.min(d.txCount * 2, 20))
      .attr('fill', (d: DagNode) => d.isBlue ? '#3b82f6' : '#ef4444')
      .attr('stroke', '#fff')
      .attr('stroke-width', 2);

    // Add labels
    node.append('text')
      .text((d: DagNode) => `#${d.number}`)
      .attr('x', 0)
      .attr('y', -15)
      .attr('text-anchor', 'middle')
      .attr('font-size', '12px')
      .attr('fill', '#374151');

    // Add tooltip
    node.append('title')
      .text((d: DagNode) => 
        `Block #${d.number}\n` +
        `Blue Score: ${d.blueScore}\n` +
        `Transactions: ${d.txCount}\n` +
        `Time: ${new Date(d.timestamp).toLocaleString()}`
      );

    // Handle click
    node.on('click', (event, d: any) => {
      setSelectedNode(d.id);
      // Navigate to block page
      window.location.href = `/block/${d.id}`;
    });

    // Update positions on tick
    simulation.on('tick', () => {
      link
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y);

      node.attr('transform', (d: any) => `translate(${d.x},${d.y})`);
    });

    function dragstarted(event: any, d: any) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      d.fx = d.x;
      d.fy = d.y;
    }

    function dragged(event: any, d: any) {
      d.fx = event.x;
      d.fy = event.y;
    }

    function dragended(event: any, d: any) {
      if (!event.active) simulation.alphaTarget(0);
      d.fx = null;
      d.fy = null;
    }
  }, [data, dimensions]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[600px]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="relative">
      <div className="absolute top-4 right-4 bg-white dark:bg-gray-700 rounded-lg p-3 shadow-md">
        <div className="text-sm space-y-1">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 bg-blue-500 rounded-full"></div>
            <span>Blue Blocks ({data?.stats.blueNodes || 0})</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 bg-red-500 rounded-full"></div>
            <span>Red Blocks ({data?.stats.redNodes || 0})</span>
          </div>
        </div>
      </div>
      
      <svg
        ref={svgRef}
        width={dimensions.width}
        height={dimensions.height}
        className="w-full"
        style={{ cursor: 'grab' }}
      />
    </div>
  );
}