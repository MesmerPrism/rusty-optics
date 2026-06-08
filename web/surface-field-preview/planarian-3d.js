const NODE_STRIDE = 9;
const SNAPSHOT_STRIDE = 4;
const EDGE_STRIDE = 4;
const CONDUCTANCE_STRIDE = 6;
const PICK_SELECTION_SCHEMA_ID = "rusty.optics.fields.planarian_bioelectric.pick_selection.v1";

export async function createPlanarianBioelectric3DView(options) {
  const three = await import(options.threeModuleUrl);
  const view = new PlanarianBioelectric3DView(three, options);
  view.initialize();
  return view;
}

class PlanarianBioelectric3DView {
  constructor(three, options) {
    this.THREE = three;
    this.container = options.container;
    this.runtime = options.runtime;
    this.visualId = options.visualId || "fields.visual.planarian3d.live";
    this.surfaceId = options.surfaceId || "mesh.planarian_ap.sketchfab_educational_surface";
    this.substrateId = options.substrateId || "fields.substrate.planarian_ap.sketchfab_educational";
    this.getViewRevision = options.getViewRevision || (() => null);
    this.onSelectNode = options.onSelectNode || (() => {});
    this.nodes = [];
    this.edges = [];
    this.edgeGroups = [];
    this.selectedNodeIndex = null;
    this.pointer = null;
    this.yaw = -0.42;
    this.pitch = 0.42;
    this.distance = 1;
    this.target = new this.THREE.Vector3();
    this.mouse = new this.THREE.Vector2();
    this.raycaster = new this.THREE.Raycaster();
    this.nodeColor = new this.THREE.Color();
    this.pickCounter = 0;
  }

  initialize() {
    this.readRuntimeGeometry();
    this.createScene();
    this.createBodySurface();
    this.createConductanceEdges();
    this.createNodes();
    this.createSelectedNodeMarker();
    this.installControls();
    this.updateSnapshot(this.runtime.snapshot(), this.runtime.conductance_values(), "circuit.voltage");
    this.render();
  }

  readRuntimeGeometry() {
    const bodyVertices = this.runtime.body_vertices();
    const bodyTriangles = this.runtime.body_triangles();
    this.bodyVertices = bodyVertices;
    this.bodyTriangles = bodyTriangles;

    const nodeData = this.runtime.nodes();
    const nodeCount = nodeData.length / NODE_STRIDE;
    this.nodes = [];
    for (let nodeIndex = 0; nodeIndex < nodeCount; nodeIndex += 1) {
      const offset = nodeIndex * NODE_STRIDE;
      const position = new this.THREE.Vector3(
        nodeData[offset],
        nodeData[offset + 1],
        nodeData[offset + 2],
      );
      this.nodes.push({
        nodeIndex,
        position,
        normal: new this.THREE.Vector3(
          nodeData[offset + 3],
          nodeData[offset + 4],
          nodeData[offset + 5],
        ),
        regionCode: nodeData[offset + 6],
        ap: nodeData[offset + 7],
        lateral: nodeData[offset + 8],
      });
    }

    const edgeData = this.runtime.conductance_edges();
    this.edges = [];
    for (let offset = 0; offset < edgeData.length; offset += EDGE_STRIDE) {
      this.edges.push({
        from: edgeData[offset],
        to: edgeData[offset + 1],
        tier: edgeData[offset + 2],
        hasGate: edgeData[offset + 3] > 0,
      });
    }

    this.bounds = computeBounds(this.THREE, bodyVertices, this.nodes);
    this.target.copy(this.bounds.center);
    this.distance = Math.max(0.001, this.bounds.radius * 1.55);
    this.nodeRadius = Math.max(0.008, this.bounds.radius * 0.026);
  }

  createScene() {
    const THREE = this.THREE;
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x0c0f14);
    this.camera = new THREE.PerspectiveCamera(42, 1, 0.001, this.bounds.radius * 18);
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.setClearColor(0x0c0f14, 1);
    this.container.innerHTML = "";
    this.container.append(this.renderer.domElement);

    this.scene.add(new THREE.AmbientLight(0xd7e5ef, 0.86));
    const key = new THREE.DirectionalLight(0xffffff, 1.2);
    key.position.set(0.7, 1.4, 0.9);
    this.scene.add(key);
    const fill = new THREE.DirectionalLight(0x7db8ff, 0.45);
    fill.position.set(-1.0, 0.3, -0.8);
    this.scene.add(fill);
  }

  createBodySurface() {
    const THREE = this.THREE;
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.BufferAttribute(this.bodyVertices, 3));
    geometry.setIndex(new THREE.BufferAttribute(this.bodyTriangles, 1));
    geometry.computeVertexNormals();

    const colors = new Float32Array(this.bodyVertices.length);
    for (let offset = 0; offset < this.bodyVertices.length; offset += 3) {
      const z = this.bodyVertices[offset + 2];
      const color = bodyColorForZ(THREE, z, this.bounds.min.z, this.bounds.max.z);
      colors[offset] = color.r;
      colors[offset + 1] = color.g;
      colors[offset + 2] = color.b;
    }
    geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));

    const material = new THREE.MeshStandardMaterial({
      color: 0xffffff,
      vertexColors: true,
      roughness: 0.82,
      metalness: 0.02,
      transparent: true,
      opacity: 0.42,
      side: THREE.DoubleSide,
    });
    this.bodyMesh = new THREE.Mesh(geometry, material);
    this.scene.add(this.bodyMesh);

    const wire = new THREE.WireframeGeometry(geometry);
    const wireMaterial = new THREE.LineBasicMaterial({
      color: 0xb7c4cf,
      transparent: true,
      opacity: 0.22,
    });
    this.bodyWire = new THREE.LineSegments(wire, wireMaterial);
    this.scene.add(this.bodyWire);
  }

  createConductanceEdges() {
    const THREE = this.THREE;
    const groups = new Map();
    groups.set(1, { tier: 1, positions: [], colors: [], edgeIndices: [] });
    groups.set(2, { tier: 2, positions: [], colors: [], edgeIndices: [] });

    for (let edgeIndex = 0; edgeIndex < this.edges.length; edgeIndex += 1) {
      const edge = this.edges[edgeIndex];
      const group = groups.get(edge.tier) || groups.get(2);
      const start = this.nodes[edge.from]?.position;
      const end = this.nodes[edge.to]?.position;
      if (!start || !end) {
        continue;
      }
      group.edgeIndices.push(edgeIndex);
      group.positions.push(start.x, start.y, start.z, end.x, end.y, end.z);
      group.colors.push(0.35, 0.45, 0.50, 0.35, 0.45, 0.50);
    }

    this.edgeGroups = [...groups.values()].map((group) => {
      const geometry = new THREE.BufferGeometry();
      const colorArray = new Float32Array(group.colors);
      geometry.setAttribute("position", new THREE.Float32BufferAttribute(group.positions, 3));
      geometry.setAttribute("color", new THREE.BufferAttribute(colorArray, 3));
      const material = new THREE.LineBasicMaterial({
        vertexColors: true,
        transparent: true,
        opacity: group.tier === 1 ? 0.78 : 0.42,
        depthTest: false,
        depthWrite: false,
      });
      const lines = new THREE.LineSegments(geometry, material);
      this.scene.add(lines);
      return { ...group, colorArray, geometry, lines };
    });
  }

  createNodes() {
    const THREE = this.THREE;
    const positions = new Float32Array(this.nodes.length * 3);
    const colors = new Float32Array(this.nodes.length * 3);
    for (const node of this.nodes) {
      const renderPosition = node.position.clone().addScaledVector(node.normal, this.nodeRadius * 1.7);
      node.renderPosition = renderPosition;
      const offset = node.nodeIndex * 3;
      positions[offset] = renderPosition.x;
      positions[offset + 1] = renderPosition.y;
      positions[offset + 2] = renderPosition.z;
      colors[offset] = 0.86;
      colors[offset + 1] = 0.90;
      colors[offset + 2] = 0.94;
    }
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    const material = new THREE.PointsMaterial({
      alphaTest: 0.08,
      depthTest: false,
      map: createNodePointTexture(THREE),
      size: this.nodeRadius * 1.75,
      sizeAttenuation: true,
      transparent: true,
      opacity: 0.94,
      vertexColors: true,
    });
    this.nodeColors = colors;
    this.nodeGeometry = geometry;
    this.nodePoints = new THREE.Points(geometry, material);
    this.scene.add(this.nodePoints);
  }

  createSelectedNodeMarker() {
    const THREE = this.THREE;
    const geometry = new THREE.SphereGeometry(this.nodeRadius * 0.82, 24, 14);
    const material = new THREE.MeshBasicMaterial({
      color: 0xffffff,
      depthTest: false,
      wireframe: true,
      transparent: true,
      opacity: 0.92,
    });
    this.selectedMarker = new THREE.Mesh(geometry, material);
    this.selectedMarker.visible = false;
    this.scene.add(this.selectedMarker);
  }

  installControls() {
    const element = this.renderer.domElement;
    element.addEventListener("pointerdown", (event) => {
      element.setPointerCapture(event.pointerId);
      this.pointer = {
        x: event.clientX,
        y: event.clientY,
        yaw: this.yaw,
        pitch: this.pitch,
        moved: false,
      };
    });
    element.addEventListener("pointermove", (event) => {
      if (!this.pointer) {
        return;
      }
      const dx = event.clientX - this.pointer.x;
      const dy = event.clientY - this.pointer.y;
      if (Math.hypot(dx, dy) > 3) {
        this.pointer.moved = true;
      }
      this.yaw = this.pointer.yaw - dx * 0.006;
      this.pitch = clamp(this.pointer.pitch - dy * 0.005, -1.12, 1.12);
      this.render();
    });
    element.addEventListener("pointerup", (event) => {
      if (this.pointer && !this.pointer.moved) {
        this.pickNode(event);
      }
      this.pointer = null;
    });
    element.addEventListener("wheel", (event) => {
      event.preventDefault();
      this.distance = clamp(
        this.distance * (event.deltaY > 0 ? 1.08 : 0.92),
        this.bounds.radius * 0.85,
        this.bounds.radius * 8,
      );
      this.render();
    }, { passive: false });
  }

  updateSnapshot(snapshot, conductanceValues, layer) {
    this.snapshot = snapshot;
    this.updateNodeColors(layer);
    this.updateConductanceColors(conductanceValues);
  }

  updateNodeColors(layer) {
    if (!this.snapshot || !this.nodeGeometry) {
      return;
    }
    for (let nodeIndex = 0; nodeIndex < this.nodes.length; nodeIndex += 1) {
      const value = snapshotValue(this.snapshot, nodeIndex, layer);
      colorForLayer(this.THREE, this.nodeColor, layer, value);
      const offset = nodeIndex * 3;
      this.nodeColors[offset] = this.nodeColor.r;
      this.nodeColors[offset + 1] = this.nodeColor.g;
      this.nodeColors[offset + 2] = this.nodeColor.b;
    }
    this.nodeGeometry.attributes.color.needsUpdate = true;
  }

  updateConductanceColors(values) {
    if (!values) {
      return;
    }
    let maxConductance = 1.0e-6;
    for (let offset = 0; offset < values.length; offset += CONDUCTANCE_STRIDE) {
      maxConductance = Math.max(maxConductance, values[offset + 1]);
    }
    for (const group of this.edgeGroups) {
      for (let localIndex = 0; localIndex < group.edgeIndices.length; localIndex += 1) {
        const edgeIndex = group.edgeIndices[localIndex];
        const conductance = values[edgeIndex * CONDUCTANCE_STRIDE + 1];
        const normalized = clamp(conductance / maxConductance, 0, 1);
        const color = conductanceColor(this.THREE, normalized, group.tier);
        const colorOffset = localIndex * 6;
        group.colorArray[colorOffset] = color.r;
        group.colorArray[colorOffset + 1] = color.g;
        group.colorArray[colorOffset + 2] = color.b;
        group.colorArray[colorOffset + 3] = color.r;
        group.colorArray[colorOffset + 4] = color.g;
        group.colorArray[colorOffset + 5] = color.b;
      }
      group.geometry.attributes.color.needsUpdate = true;
    }
  }

  setVisibility(showEdges, showTier2) {
    for (const group of this.edgeGroups) {
      group.lines.visible = showEdges && (showTier2 || group.tier !== 2);
    }
  }

  selectNode(nodeIndex) {
    this.selectedNodeIndex = Number.isInteger(nodeIndex) ? nodeIndex : null;
    if (this.selectedNodeIndex === null) {
      this.selectedMarker.visible = false;
      this.render();
      return;
    }
    const node = this.nodes[this.selectedNodeIndex];
    if (!node) {
      this.selectedMarker.visible = false;
      this.render();
      return;
    }
    this.selectedMarker.position.copy(node.renderPosition || node.position);
    this.selectedMarker.visible = true;
    this.render();
  }

  pickNode(event) {
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
    this.raycaster.params.Points.threshold = this.nodeRadius * 3.2;
    this.raycaster.setFromCamera(this.mouse, this.camera);
    const hits = this.raycaster.intersectObject(this.nodePoints, false);
    const hit = hits.find((entry) => Number.isInteger(entry.index));
    if (hit) {
      this.onSelectNode(this.selectionForNode(hit.index, hit.distance));
    }
  }

  selectionForNode(nodeIndex, distance) {
    const node = this.nodes[nodeIndex];
    const revision = this.getViewRevision();
    this.pickCounter += 1;
    return {
      schema_id: PICK_SELECTION_SCHEMA_ID,
      selection_id: [
        this.visualId,
        "pick",
        `node_${String(nodeIndex).padStart(4, "0")}`,
        revision === null ? "runknown" : `r${Math.trunc(revision)}`,
        this.pickCounter,
      ].join("."),
      visual_id: this.visualId,
      surface_id: this.surfaceId,
      substrate_id: this.substrateId,
      target: {
        SurfaceNode: {
          node_index: nodeIndex,
          node_id: `${this.substrateId}.node.${String(nodeIndex).padStart(4, "0")}`,
          region_id: regionIdForCode(node?.regionCode),
          ap_coordinate: node?.ap ?? 0,
          lateral_coordinate: node?.lateral ?? 0,
        },
      },
      normalized_pointer: { x: this.mouse.x, y: this.mouse.y },
      distance,
      view_revision: revision,
    };
  }

  render() {
    if (!this.renderer) {
      return;
    }
    this.resize();
    this.updateCamera();
    this.renderer.render(this.scene, this.camera);
  }

  resize() {
    const rect = this.container.getBoundingClientRect();
    const width = Math.max(1, Math.floor(rect.width));
    const height = Math.max(1, Math.floor(rect.height));
    if (this.renderer.domElement.width !== Math.floor(width * this.renderer.getPixelRatio())
      || this.renderer.domElement.height !== Math.floor(height * this.renderer.getPixelRatio())) {
      this.renderer.setSize(width, height, false);
      this.camera.aspect = width / height;
      this.camera.updateProjectionMatrix();
    }
  }

  updateCamera() {
    const cosPitch = Math.cos(this.pitch);
    this.camera.position.set(
      this.target.x + Math.sin(this.yaw) * cosPitch * this.distance,
      this.target.y + Math.sin(this.pitch) * this.distance,
      this.target.z + Math.cos(this.yaw) * cosPitch * this.distance,
    );
    this.camera.lookAt(this.target);
  }
}

function computeBounds(THREE, vertices, nodes) {
  const min = new THREE.Vector3(Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY);
  const max = new THREE.Vector3(Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY);
  for (let offset = 0; offset < vertices.length; offset += 3) {
    min.x = Math.min(min.x, vertices[offset]);
    min.y = Math.min(min.y, vertices[offset + 1]);
    min.z = Math.min(min.z, vertices[offset + 2]);
    max.x = Math.max(max.x, vertices[offset]);
    max.y = Math.max(max.y, vertices[offset + 1]);
    max.z = Math.max(max.z, vertices[offset + 2]);
  }
  for (const node of nodes) {
    min.min(node.position);
    max.max(node.position);
  }
  const size = new THREE.Vector3().subVectors(max, min);
  const center = new THREE.Vector3().addVectors(min, max).multiplyScalar(0.5);
  return {
    min,
    max,
    size,
    center,
    radius: Math.max(size.x, size.y, size.z, 0.001),
  };
}

function bodyColorForZ(THREE, z, minZ, maxZ) {
  const t = clamp((z - minZ) / Math.max(1.0e-6, maxZ - minZ), 0, 1);
  const color = new THREE.Color();
  if (t < 0.5) {
    color.setRGB(0.24 + t * 0.28, 0.34 + t * 0.34, 0.40 + t * 0.28);
  } else {
    const u = (t - 0.5) * 2;
    color.setRGB(0.38 + u * 0.28, 0.50 + u * 0.20, 0.54 - u * 0.04);
  }
  return color;
}

function snapshotValue(snapshot, nodeIndex, layer) {
  const offset = nodeIndex * SNAPSHOT_STRIDE;
  if (layer === "circuit.memory") {
    return snapshot[offset + 1];
  }
  if (layer.includes("head_identity")) {
    return snapshot[offset + 2];
  }
  if (layer.includes("tail_identity")) {
    return snapshot[offset + 3];
  }
  return snapshot[offset];
}

function colorForLayer(THREE, color, layer, value) {
  if (layer === "circuit.memory") {
    const t = clamp(value, 0, 1);
    color.setRGB(0.12 + t * 0.30, 0.34 + t * 0.62, 0.30 + t * 0.42);
    return color;
  }
  if (layer.includes("head_identity")) {
    const t = clamp(value, 0, 1);
    color.setRGB(0.16 + t * 0.12, 0.38 + t * 0.56, 0.54 + t * 0.42);
    return color;
  }
  if (layer.includes("tail_identity")) {
    const t = clamp(value, 0, 1);
    color.setRGB(0.36 + t * 0.62, 0.28 + t * 0.44, 0.14 + t * 0.14);
    return color;
  }
  const t = clamp((value + 1) * 0.5, 0, 1);
  if (t < 0.5) {
    const u = t * 2;
    color.setRGB(0.18 + u * 0.54, 0.42 + u * 0.42, 0.86 + u * 0.08);
  } else {
    const u = (t - 0.5) * 2;
    color.setRGB(0.72 + u * 0.26, 0.84 - u * 0.28, 0.94 - u * 0.68);
  }
  return color;
}

function conductanceColor(THREE, normalized, tier) {
  const t = clamp(normalized, 0, 1);
  const color = new THREE.Color();
  if (tier === 1) {
    color.setRGB(0.28 + t * 0.38, 0.42 + t * 0.44, 0.52 + t * 0.34);
  } else {
    color.setRGB(0.20 + t * 0.24, 0.28 + t * 0.30, 0.34 + t * 0.22);
  }
  return color;
}

function regionIdForCode(regionCode) {
  switch (Math.trunc(regionCode || 0)) {
    case 1:
      return "region_tail";
    case 2:
      return "region_postpharyngeal_trunk";
    case 3:
      return "region_pharyngeal_trunk";
    case 4:
      return "region_prepharyngeal_trunk";
    case 5:
      return "region_head";
    default:
      return "region_unknown";
  }
}

function createNodePointTexture(THREE) {
  const canvas = document.createElement("canvas");
  canvas.width = 64;
  canvas.height = 64;
  const context = canvas.getContext("2d");
  const gradient = context.createRadialGradient(32, 32, 3, 32, 32, 31);
  gradient.addColorStop(0, "rgba(255,255,255,1)");
  gradient.addColorStop(0.62, "rgba(255,255,255,0.94)");
  gradient.addColorStop(1, "rgba(255,255,255,0)");
  context.fillStyle = gradient;
  context.fillRect(0, 0, 64, 64);
  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}
