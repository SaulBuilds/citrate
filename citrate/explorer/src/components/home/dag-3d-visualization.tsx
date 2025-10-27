'use client';

import { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';

interface DagNode {
  id: string;
  number: string;
  blueScore: number;
  isBlue: boolean;
  txCount: number;
  timestamp: string;
  position?: THREE.Vector3;
}

interface DagLink {
  source: string;
  target: string;
  type: 'parent' | 'selected' | 'merge';
}

export function Dag3DVisualization() {
  const mountRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const nodesRef = useRef<Map<string, THREE.Mesh>>(new Map());
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [hoveredNode, setHoveredNode] = useState<DagNode | null>(null);

  const { data, isLoading } = useQuery({
    queryKey: ['dag-3d'],
    queryFn: async () => {
      const response = await axios.get('/api/dag?depth=50');
      return response.data;
    },
    refetchInterval: 15000
  });

  useEffect(() => {
    if (!mountRef.current) return;

    // Scene setup
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0a0a0a);
    scene.fog = new THREE.Fog(0x0a0a0a, 100, 1000);
    sceneRef.current = scene;

    // Camera setup
    const camera = new THREE.PerspectiveCamera(
      75,
      mountRef.current.clientWidth / mountRef.current.clientHeight,
      0.1,
      1000
    );
    camera.position.set(0, 0, 100);
    cameraRef.current = camera;

    // Renderer setup
    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setSize(mountRef.current.clientWidth, mountRef.current.clientHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    mountRef.current.appendChild(renderer.domElement);
    rendererRef.current = renderer;

    // Controls
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.05;
    controls.rotateSpeed = 0.5;
    controls.zoomSpeed = 1;
    controlsRef.current = controls;

    // Lighting
    const ambientLight = new THREE.AmbientLight(0x404040, 1.5);
    scene.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 1);
    directionalLight.position.set(50, 50, 50);
    directionalLight.castShadow = true;
    scene.add(directionalLight);

    const pointLight1 = new THREE.PointLight(0x3b82f6, 2, 100);
    pointLight1.position.set(30, 30, 30);
    scene.add(pointLight1);

    const pointLight2 = new THREE.PointLight(0x8b5cf6, 2, 100);
    pointLight2.position.set(-30, -30, -30);
    scene.add(pointLight2);

    // Grid helper
    const gridHelper = new THREE.GridHelper(200, 20, 0x2a2a2a, 0x1a1a1a);
    gridHelper.rotation.x = Math.PI / 2;
    scene.add(gridHelper);

    // Animation loop
    const animate = () => {
      requestAnimationFrame(animate);
      
      if (controlsRef.current) {
        controlsRef.current.update();
      }

      // Rotate nodes slightly
      nodesRef.current.forEach((mesh) => {
        mesh.rotation.x += 0.001;
        mesh.rotation.y += 0.002;
      });

      if (rendererRef.current && sceneRef.current && cameraRef.current) {
        rendererRef.current.render(sceneRef.current, cameraRef.current);
      }
    };

    animate();

    // Handle resize
    const handleResize = () => {
      if (!mountRef.current || !cameraRef.current || !rendererRef.current) return;
      
      cameraRef.current.aspect = mountRef.current.clientWidth / mountRef.current.clientHeight;
      cameraRef.current.updateProjectionMatrix();
      rendererRef.current.setSize(mountRef.current.clientWidth, mountRef.current.clientHeight);
    };

    window.addEventListener('resize', handleResize);

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize);
      if (mountRef.current && rendererRef.current) {
        mountRef.current.removeChild(rendererRef.current.domElement);
      }
      rendererRef.current?.dispose();
    };
  }, []);

  useEffect(() => {
    if (!data || !sceneRef.current) return;

    // Clear existing nodes
    nodesRef.current.forEach(mesh => {
      sceneRef.current?.remove(mesh);
    });
    nodesRef.current.clear();

    // Calculate positions using blue score for vector space positioning
    const nodes: DagNode[] = data.nodes.map((node: any, index: number) => {
      const blueScore = parseFloat(node.blueScore);
      const blockNum = parseFloat(node.number);
      
      // Position nodes in 3D space based on blue score and block number
      // Higher blue scores get positioned higher in Y axis
      // Block number determines X axis spread
      // Add some randomness for Z axis depth
      const position = new THREE.Vector3(
        (blockNum % 20 - 10) * 8, // X: spread based on block number
        blueScore * 0.5 - 20,      // Y: height based on blue score
        (Math.random() - 0.5) * 60 // Z: random depth
      );

      return {
        ...node,
        blueScore,
        position
      };
    });

    // Create node meshes
    nodes.forEach((node) => {
      // Geometry based on node type
      const geometry = node.isBlue
        ? new THREE.OctahedronGeometry(2 + node.txCount * 0.1)
        : new THREE.TetrahedronGeometry(1.5 + node.txCount * 0.1);

      // Material with emissive glow
      const material = new THREE.MeshPhongMaterial({
        color: node.isBlue ? 0x3b82f6 : 0xef4444,
        emissive: node.isBlue ? 0x1e40af : 0x991b1b,
        emissiveIntensity: 0.5,
        transparent: true,
        opacity: 0.9,
        shininess: 100
      });

      const mesh = new THREE.Mesh(geometry, material);
      mesh.position.copy(node.position!);
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      mesh.userData = node;

      // Add glow effect
      const glowGeometry = node.isBlue
        ? new THREE.OctahedronGeometry(2.5 + node.txCount * 0.1)
        : new THREE.TetrahedronGeometry(2 + node.txCount * 0.1);
      
      const glowMaterial = new THREE.MeshBasicMaterial({
        color: node.isBlue ? 0x60a5fa : 0xf87171,
        transparent: true,
        opacity: 0.2
      });

      const glowMesh = new THREE.Mesh(glowGeometry, glowMaterial);
      mesh.add(glowMesh);

      sceneRef.current!.add(mesh);
      nodesRef.current.set(node.id, mesh);
    });

    // Create links
    data.links.forEach((link: DagLink) => {
      const sourceNode = nodesRef.current.get(link.source);
      const targetNode = nodesRef.current.get(link.target);

      if (sourceNode && targetNode) {
        const points = [];
        points.push(sourceNode.position);
        
        // Add curve for better visualization
        const midPoint = new THREE.Vector3()
          .addVectors(sourceNode.position, targetNode.position)
          .multiplyScalar(0.5);
        midPoint.y += 5; // Curve upward
        points.push(midPoint);
        
        points.push(targetNode.position);

        const curve = new THREE.CatmullRomCurve3(points);
        const curvePoints = curve.getPoints(50);
        const geometry = new THREE.BufferGeometry().setFromPoints(curvePoints);

        const material = new THREE.LineBasicMaterial({
          color: link.type === 'selected' ? 0x3b82f6 :
                 link.type === 'merge' ? 0x8b5cf6 : 0x4b5563,
          opacity: 0.6,
          transparent: true,
          linewidth: link.type === 'selected' ? 2 : 1
        });

        const line = new THREE.Line(geometry, material);
        sceneRef.current!.add(line);
      }
    });

    // Add particle system for ambiance
    const particlesGeometry = new THREE.BufferGeometry();
    const particlesCount = 1000;
    const posArray = new Float32Array(particlesCount * 3);

    for (let i = 0; i < particlesCount * 3; i++) {
      posArray[i] = (Math.random() - 0.5) * 200;
    }

    particlesGeometry.setAttribute('position', new THREE.BufferAttribute(posArray, 3));
    
    const particlesMaterial = new THREE.PointsMaterial({
      size: 0.5,
      color: 0x3b82f6,
      transparent: true,
      opacity: 0.3
    });

    const particlesMesh = new THREE.Points(particlesGeometry, particlesMaterial);
    sceneRef.current!.add(particlesMesh);

  }, [data]);

  // Mouse interaction
  useEffect(() => {
    if (!rendererRef.current || !cameraRef.current) return;

    const raycaster = new THREE.Raycaster();
    const mouse = new THREE.Vector2();

    const handleMouseMove = (event: MouseEvent) => {
      if (!mountRef.current) return;
      
      const rect = mountRef.current.getBoundingClientRect();
      mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

      raycaster.setFromCamera(mouse, cameraRef.current!);
      
      const intersects = raycaster.intersectObjects(Array.from(nodesRef.current.values()));
      
      if (intersects.length > 0) {
        const node = intersects[0].object.userData as DagNode;
        setHoveredNode(node);
        document.body.style.cursor = 'pointer';
      } else {
        setHoveredNode(null);
        document.body.style.cursor = 'default';
      }
    };

    const handleClick = (event: MouseEvent) => {
      if (!mountRef.current) return;
      
      const rect = mountRef.current.getBoundingClientRect();
      mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

      raycaster.setFromCamera(mouse, cameraRef.current!);
      
      const intersects = raycaster.intersectObjects(Array.from(nodesRef.current.values()));
      
      if (intersects.length > 0) {
        const node = intersects[0].object.userData as DagNode;
        setSelectedNode(node.id);
        window.location.href = `/block/${node.id}`;
      }
    };

    rendererRef.current.domElement.addEventListener('mousemove', handleMouseMove);
    rendererRef.current.domElement.addEventListener('click', handleClick);

    return () => {
      rendererRef.current?.domElement.removeEventListener('mousemove', handleMouseMove);
      rendererRef.current?.domElement.removeEventListener('click', handleClick);
    };
  }, []);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[600px] bg-gray-900 rounded-xl">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-400">Loading DAG visualization...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="relative bg-gray-900 rounded-xl overflow-hidden">
      <div ref={mountRef} className="w-full h-[600px]" />
      
      {/* HUD Overlay */}
      <div className="absolute top-4 left-4 bg-black/50 backdrop-blur-sm rounded-lg p-4 text-white">
        <h3 className="font-bold mb-2">DAG Statistics</h3>
        <div className="space-y-1 text-sm">
          <div>Nodes: {data?.stats.totalNodes || 0}</div>
          <div className="text-blue-400">Blue: {data?.stats.blueNodes || 0}</div>
          <div className="text-red-400">Red: {data?.stats.redNodes || 0}</div>
        </div>
      </div>

      {/* Node Info */}
      {hoveredNode && (
        <div className="absolute bottom-4 left-4 bg-black/50 backdrop-blur-sm rounded-lg p-4 text-white max-w-sm">
          <div className="font-mono text-sm mb-2">Block #{hoveredNode.number}</div>
          <div className="text-xs space-y-1">
            <div>Blue Score: {hoveredNode.blueScore}</div>
            <div>Transactions: {hoveredNode.txCount}</div>
            <div>Time: {new Date(hoveredNode.timestamp).toLocaleString()}</div>
          </div>
        </div>
      )}

      {/* Controls */}
      <div className="absolute top-4 right-4 bg-black/50 backdrop-blur-sm rounded-lg p-3 text-white text-xs">
        <div>🖱️ Drag to rotate</div>
        <div>📜 Scroll to zoom</div>
        <div>👆 Click to explore</div>
      </div>
    </div>
  );
}