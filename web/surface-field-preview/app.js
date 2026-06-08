const canvas = document.querySelector("#viewport");
const ctx = canvas.getContext("2d");
const stats = document.querySelector("#stats");
const controls = {
  scalarLayer: document.querySelector("#scalar-layer"),
  live: document.querySelector("#toggle-live"),
  runtimeStatus: document.querySelector("#runtime-status"),
  play: document.querySelector("#toggle-play"),
  frameSlider: document.querySelector("#frame-slider"),
  framePosition: document.querySelector("#frame-position"),
  speed: document.querySelector("#playback-speed"),
  edges: document.querySelector("#toggle-edges"),
  tier2: document.querySelector("#toggle-tier2"),
  regions: document.querySelector("#toggle-regions"),
  polarity: document.querySelector("#toggle-polarity"),
  labels: document.querySelector("#toggle-labels"),
  reset: document.querySelector("#reset-view"),
};

const params = new URLSearchParams(window.location.search);
const frameUrl = params.get("frame");
const sequenceUrl = params.get("sequence")
  || (frameUrl ? null : "/fixtures/fields/surface_field_visual_sequence.json");
const payloadUrl = sequenceUrl || frameUrl || "/fixtures/fields/surface_field_visual_frame.json";
const wasmBaseUrl = params.get("wasm") || "/local-artifacts/matter_surface_field_wasm";

let sequence = null;
let frames = [];
let currentFrameIndex = 0;
let bounds = null;
let view = { zoom: 1, offsetX: 0, offsetY: 0 };
let drag = null;
let playing = true;
let lastAnimationTime = null;
let frameAccumulatorMs = 0;
let liveRuntime = null;
let liveTopology = null;
let liveFrame = null;
let liveStats = null;
let liveStepMs = 0;

fetch(payloadUrl)
  .then((response) => {
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    return response.json();
  })
  .then(async (payload) => {
    loadPayload(payload);
    await initLiveRuntime();
    requestAnimationFrame(animationLoop);
  })
  .catch((error) => {
    stats.textContent = `Frame load failed: ${error.message}`;
    setPlaying(false);
    draw();
  });

for (const input of [
  controls.scalarLayer,
  controls.edges,
  controls.tier2,
  controls.regions,
  controls.polarity,
  controls.labels,
]) {
  input.addEventListener("change", () => {
    updateStats();
    draw();
  });
}

controls.live.addEventListener("change", () => {
  if (controls.live.checked && !liveRuntime) {
    controls.live.checked = false;
    return;
  }
  if (isLiveMode()) {
    liveRuntime.reset();
    liveFrame = buildLiveFrame();
    bounds = computeBounds(liveFrame);
    setPlaying(true);
  } else if (frames.length > 0) {
    bounds = computeBounds(activeFrame());
    setPlaying(frames.length > 1);
  }
  fillLayerSelect();
  updateTimelineControls();
  updateStats();
  draw();
});

controls.play.addEventListener("click", () => {
  setPlaying(!playing);
});

controls.frameSlider.addEventListener("input", () => {
  if (isLiveMode()) {
    return;
  }
  currentFrameIndex = Number(controls.frameSlider.value);
  lastAnimationTime = null;
  frameAccumulatorMs = 0;
  updateTimelineControls();
  updateStats();
  draw();
});

controls.speed.addEventListener("change", () => {
  frameAccumulatorMs = 0;
});

controls.reset.addEventListener("click", () => {
  if (isLiveMode()) {
    liveRuntime.reset();
    liveFrame = buildLiveFrame();
  }
  resetView();
  updateTimelineControls();
  updateStats();
  draw();
});

canvas.addEventListener("pointerdown", (event) => {
  canvas.setPointerCapture(event.pointerId);
  drag = {
    x: event.clientX,
    y: event.clientY,
    offsetX: view.offsetX,
    offsetY: view.offsetY,
  };
});

canvas.addEventListener("pointermove", (event) => {
  if (!drag) {
    return;
  }
  const scale = window.devicePixelRatio || 1;
  view.offsetX = drag.offsetX + (event.clientX - drag.x) * scale;
  view.offsetY = drag.offsetY + (event.clientY - drag.y) * scale;
  draw();
});

canvas.addEventListener("pointerup", () => {
  drag = null;
});

canvas.addEventListener("wheel", (event) => {
  event.preventDefault();
  const nextZoom = view.zoom * (event.deltaY > 0 ? 0.92 : 1.08);
  view.zoom = clamp(nextZoom, 0.45, 4.0);
  draw();
}, { passive: false });

window.addEventListener("resize", draw);

function loadPayload(payload) {
  if (Array.isArray(payload.frames)) {
    sequence = payload;
    frames = payload.frames;
  } else {
    sequence = null;
    frames = [payload];
  }
  currentFrameIndex = 0;
  bounds = computeBounds(activeFrame());
  fillLayerSelect();
  resetView();
  setPlaying(frames.length > 1);
  updateTimelineControls();
  updateRuntimeStatus("sequence");
  updateStats();
  draw();
}

async function initLiveRuntime() {
  try {
    const module = await import(`${wasmBaseUrl}/rusty_matter_fields_wasm.js`);
    await module.default(`${wasmBaseUrl}/rusty_matter_fields_wasm_bg.wasm`);
    liveRuntime = new module.SurfaceFieldRealtimeRuntime();
    liveTopology = decodeLiveTopology(liveRuntime);
    liveFrame = buildLiveFrame();
    controls.live.disabled = false;
    controls.live.checked = true;
    bounds = computeBounds(liveFrame);
    fillLayerSelect();
    setPlaying(true);
    updateTimelineControls();
    updateRuntimeStatus("live");
    updateStats();
    draw();
  } catch (_error) {
    liveRuntime = null;
    liveTopology = null;
    liveFrame = null;
    controls.live.checked = false;
    controls.live.disabled = true;
    updateRuntimeStatus("sequence");
  }
}

function animationLoop(timestamp) {
  if (playing && canAnimate()) {
    if (lastAnimationTime !== null) {
      frameAccumulatorMs += timestamp - lastAnimationTime;
      const intervalMs = frameIntervalSeconds() * 1000 / playbackSpeed();
      if (isLiveMode()) {
        const steps = Math.min(8, Math.floor(frameAccumulatorMs / intervalMs));
        if (steps > 0) {
          frameAccumulatorMs -= steps * intervalMs;
          const started = performance.now();
          liveRuntime.step(steps);
          liveStepMs = performance.now() - started;
          liveFrame = buildLiveFrame();
          updateTimelineControls();
          updateStats();
          draw();
        }
      } else {
        while (frameAccumulatorMs >= intervalMs) {
          frameAccumulatorMs -= intervalMs;
          currentFrameIndex = (currentFrameIndex + 1) % frames.length;
        }
        updateTimelineControls();
        updateStats();
        draw();
      }
    }
    lastAnimationTime = timestamp;
  } else {
    lastAnimationTime = null;
  }
  requestAnimationFrame(animationLoop);
}

function setPlaying(nextPlaying) {
  playing = nextPlaying && canAnimate();
  controls.play.textContent = playing ? "Pause" : "Play";
  controls.play.disabled = !canAnimate();
}

function canAnimate() {
  return isLiveMode() || frames.length > 1;
}

function isLiveMode() {
  return Boolean(controls.live.checked && liveRuntime && liveFrame);
}

function fillLayerSelect() {
  const selected = controls.scalarLayer.value;
  controls.scalarLayer.innerHTML = "";
  const frame = activeFrame();
  for (const layer of frame.scalar_layers) {
    const option = document.createElement("option");
    option.value = layer.field_id;
    option.textContent = layer.field_id.replace("field.", "");
    controls.scalarLayer.append(option);
  }
  if (frame.scalar_layers.some((layer) => layer.field_id === selected)) {
    controls.scalarLayer.value = selected;
    return;
  }
  const wound = frame.scalar_layers.find((layer) => layer.field_id.includes("wound"));
  if (wound) {
    controls.scalarLayer.value = wound.field_id;
  }
}

function resetView() {
  view = { zoom: 1, offsetX: 0, offsetY: 0 };
}

function updateTimelineControls() {
  if (isLiveMode()) {
    controls.frameSlider.disabled = true;
    controls.frameSlider.max = "0";
    controls.frameSlider.value = "0";
    controls.framePosition.textContent = `live ${Math.trunc(liveStats.step)}`;
    return;
  }
  const frameCount = Math.max(1, frames.length);
  controls.frameSlider.max = String(frameCount - 1);
  controls.frameSlider.value = String(currentFrameIndex);
  controls.frameSlider.disabled = frameCount <= 1;
  controls.framePosition.textContent = `${currentFrameIndex + 1}/${frameCount}`;
}

function updateRuntimeStatus(mode) {
  controls.runtimeStatus.textContent = mode === "live" ? "Matter live" : "sequence";
}

function updateStats() {
  const frame = activeFrame();
  if (!frame) {
    return;
  }
  const layer = activeScalarLayer();
  const liveParts = isLiveMode()
    ? [`live step ${Math.trunc(liveStats.step)}`, `${formatNumber(liveStepMs)}ms`]
    : [`step ${frame.step_index}${sequence ? `/${sequence.step_count}` : ""}`];
  stats.textContent = [
    `${frame.nodes.length} nodes`,
    `${frame.edges.length} edges`,
    isLiveMode() ? "live Matter Wasm" : `${frames.length} frames`,
    ...liveParts,
    `t ${formatNumber(frame.time_seconds)}s`,
    `${frame.scalar_layers.length} scalar layers`,
    `${frame.vector_layers[0]?.arrows.length || 0} arrows`,
    layer ? `${layer.field_id}: ${formatNumber(layer.min_value)}..${formatNumber(layer.max_value)}` : null,
  ].filter(Boolean).join("  ");
}

function draw() {
  resizeCanvas();
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  drawBackground();
  if (!activeFrame()) {
    drawLoading();
    return;
  }
  if (controls.edges.checked) {
    drawEdges();
  }
  if (controls.regions.checked) {
    drawRegions();
  }
  drawScalarNodes();
  if (controls.polarity.checked) {
    drawVectorArrows();
  }
  if (controls.labels.checked) {
    drawLabels();
  }
}

function resizeCanvas() {
  const rect = canvas.getBoundingClientRect();
  const scale = window.devicePixelRatio || 1;
  const width = Math.max(1, Math.round(rect.width * scale));
  const height = Math.max(1, Math.round(rect.height * scale));
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }
}

function drawBackground() {
  const gradient = ctx.createLinearGradient(0, 0, 0, canvas.height);
  gradient.addColorStop(0, "#111820");
  gradient.addColorStop(1, "#080b10");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, canvas.width, canvas.height);
}

function drawLoading() {
  ctx.fillStyle = "#d9e0e8";
  ctx.font = `${14 * (window.devicePixelRatio || 1)}px sans-serif`;
  ctx.fillText("Loading surface field frame", 24, 36);
}

function drawEdges() {
  const frame = activeFrame();
  ctx.lineCap = "round";
  for (const edge of frame.edges) {
    if (edge.tier === 2 && !controls.tier2.checked) {
      continue;
    }
    const startNode = frame.nodes[edge.from];
    const endNode = frame.nodes[edge.to];
    if (!startNode || !endNode) {
      continue;
    }
    const start = project(startNode.position);
    const end = project(endNode.position);
    ctx.strokeStyle = rgba(edge.color);
    ctx.lineWidth = (edge.tier === 1 ? 1.2 : 0.65) * (window.devicePixelRatio || 1);
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(end.x, end.y);
    ctx.stroke();
  }
}

function drawRegions() {
  const frame = activeFrame();
  for (const region of frame.perturbation_regions) {
    ctx.fillStyle = rgba(region.color);
    ctx.strokeStyle = rgba({ ...region.color, a: Math.min(0.72, region.color.a * 1.8) });
    ctx.lineWidth = 1.1 * (window.devicePixelRatio || 1);
    for (const nodeIndex of region.node_indices) {
      const node = frame.nodes[nodeIndex];
      if (!node) {
        continue;
      }
      const point = project(node.position);
      const radius = nodeScreenRadius(node) * 2.15;
      ctx.beginPath();
      ctx.arc(point.x, point.y, radius, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
    }
  }
}

function drawScalarNodes() {
  const frame = activeFrame();
  const layer = activeScalarLayer();
  if (!layer) {
    return;
  }
  for (const sample of layer.samples) {
    const node = frame.nodes[sample.node_index];
    const point = project(node.position);
    ctx.fillStyle = rgba(sample.color);
    ctx.strokeStyle = "rgba(235, 241, 245, 0.78)";
    ctx.lineWidth = 1.05 * (window.devicePixelRatio || 1);
    ctx.beginPath();
    ctx.arc(point.x, point.y, nodeScreenRadius(node), 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
  }
}

function drawVectorArrows() {
  const frame = activeFrame();
  for (const layer of frame.vector_layers) {
    for (const arrow of layer.arrows) {
      const start = project(arrow.start);
      const end = project(arrow.end);
      ctx.strokeStyle = rgba(arrow.color);
      ctx.fillStyle = rgba(arrow.color);
      ctx.lineWidth = 1.5 * (window.devicePixelRatio || 1);
      ctx.beginPath();
      ctx.moveTo(start.x, start.y);
      ctx.lineTo(end.x, end.y);
      ctx.stroke();
      drawArrowHead(start, end, 6 * (window.devicePixelRatio || 1));
    }
  }
}

function drawLabels() {
  const frame = activeFrame();
  const scale = window.devicePixelRatio || 1;
  ctx.fillStyle = "rgba(226, 232, 238, 0.82)";
  ctx.font = `${11 * scale}px sans-serif`;
  for (const node of frame.nodes) {
    const point = project(node.position);
    ctx.fillText(String(node.node_index), point.x + nodeScreenRadius(node) * 0.9, point.y - 4 * scale);
  }
}

function drawArrowHead(start, end, size) {
  const dx = end.x - start.x;
  const dy = end.y - start.y;
  const length = Math.hypot(dx, dy);
  if (length <= 0.001) {
    return;
  }
  const ux = dx / length;
  const uy = dy / length;
  const left = {
    x: end.x - ux * size - uy * size * 0.55,
    y: end.y - uy * size + ux * size * 0.55,
  };
  const right = {
    x: end.x - ux * size + uy * size * 0.55,
    y: end.y - uy * size - ux * size * 0.55,
  };
  ctx.beginPath();
  ctx.moveTo(end.x, end.y);
  ctx.lineTo(left.x, left.y);
  ctx.lineTo(right.x, right.y);
  ctx.closePath();
  ctx.fill();
}

function activeFrame() {
  if (isLiveMode()) {
    return liveFrame;
  }
  return frames[currentFrameIndex] || frames[0] || null;
}

function activeScalarLayer() {
  const frame = activeFrame();
  return frame?.scalar_layers.find((layer) => layer.field_id === controls.scalarLayer.value)
    || frame?.scalar_layers[0]
    || null;
}

function decodeLiveTopology(runtime) {
  const nodeData = runtime.nodes();
  const nodeCount = nodeData.length / 6;
  const positions = [];
  for (let index = 0; index < nodeCount; index += 1) {
    const offset = index * 6;
    positions.push({
      x: nodeData[offset],
      y: nodeData[offset + 1],
      z: nodeData[offset + 2],
    });
  }
  const topologyBounds = computeBoundsFromPositions(positions);
  const radius = visualNodeRadius(topologyBounds);
  const nodes = positions.map((position, nodeIndex) => ({
    node_index: nodeIndex,
    node_id: `live.node.${String(nodeIndex).padStart(4, "0")}`,
    position,
    radius,
  }));

  const edgeData = runtime.edges();
  const edges = [];
  for (let offset = 0; offset < edgeData.length; offset += 3) {
    const tier = edgeData[offset + 2];
    edges.push({
      from: edgeData[offset],
      to: edgeData[offset + 1],
      tier,
      color: edgeColor(tier),
    });
  }

  const regionData = runtime.region_metadata();
  const regionNodes = runtime.region_nodes();
  const perturbation_regions = [];
  for (let offset = 0; offset < regionData.length; offset += 4) {
    const effectCode = regionData[offset];
    const targetCodeValue = regionData[offset + 1];
    const nodeOffset = regionData[offset + 2];
    const nodeLength = regionData[offset + 3];
    const node_indices = [];
    for (let index = 0; index < nodeLength; index += 1) {
      node_indices.push(regionNodes[nodeOffset + index]);
    }
    const effect_kind = effectKind(effectCode);
    perturbation_regions.push({
      perturbation_id: `live.region.${offset / 4}`,
      target_field_id: targetField(targetCodeValue),
      effect_kind,
      node_indices,
      color: perturbationColor(effect_kind),
    });
  }

  return { bounds: topologyBounds, nodes, edges, perturbation_regions };
}

function buildLiveFrame() {
  const snapshot = liveRuntime.snapshot();
  const nodeCount = liveTopology.nodes.length;
  const fields = {
    "field.vmem_like": new Array(nodeCount),
    "field.wound_signal": new Array(nodeCount),
    "field.morphogen": new Array(nodeCount),
  };
  const vectors = new Array(nodeCount);
  for (let nodeIndex = 0; nodeIndex < nodeCount; nodeIndex += 1) {
    const offset = nodeIndex * 6;
    fields["field.vmem_like"][nodeIndex] = snapshot[offset];
    fields["field.wound_signal"][nodeIndex] = snapshot[offset + 1];
    fields["field.morphogen"][nodeIndex] = snapshot[offset + 2];
    vectors[nodeIndex] = {
      x: snapshot[offset + 3],
      y: snapshot[offset + 4],
      z: snapshot[offset + 5],
    };
  }
  liveStats = readLiveStats();
  return {
    schema_id: "rusty.optics.fields.surface.visual_frame.v1",
    frame_id: `live.surface_field.frame.${Math.trunc(liveStats.step)}`,
    source_frame_id: "live.matter.surface_field",
    source_schema_id: "rusty.matter.fields.live_wasm.v1",
    substrate_id: "fields.substrate.wasm_unit_square_dynamic",
    surface_id: "mesh.unit_square_surface",
    step_index: Math.trunc(liveStats.step),
    time_seconds: liveStats.time_seconds,
    bounds_min: liveTopology.bounds.min,
    bounds_max: liveTopology.bounds.max,
    nodes: liveTopology.nodes,
    edges: liveTopology.edges,
    scalar_layers: [
      buildLiveScalarLayer("field.vmem_like", "VmemLike", fields["field.vmem_like"]),
      buildLiveScalarLayer("field.wound_signal", "WoundSignal", fields["field.wound_signal"]),
      buildLiveScalarLayer("field.morphogen", "Morphogen", fields["field.morphogen"]),
    ],
    vector_layers: [buildLiveVectorLayer("field.polarity", "Polarity", vectors)],
    perturbation_regions: liveTopology.perturbation_regions,
  };
}

function buildLiveScalarLayer(fieldId, kind, values) {
  const minValue = Math.min(...values);
  const maxValue = Math.max(...values);
  const range = Math.max(1.0e-6, maxValue - minValue);
  return {
    field_id: fieldId,
    kind,
    min_value: minValue,
    max_value: maxValue,
    samples: values.map((value, nodeIndex) => {
      const normalized_value = clamp((value - minValue) / range, 0, 1);
      return {
        node_index: nodeIndex,
        value,
        normalized_value,
        color: scalarColor(fieldId, kind, normalized_value),
      };
    }),
  };
}

function buildLiveVectorLayer(fieldId, kind, vectors) {
  const arrowScale = liveTopology.nodes[0].radius * 4.2;
  const arrows = [];
  for (let nodeIndex = 0; nodeIndex < vectors.length; nodeIndex += 1) {
    const vector = vectors[nodeIndex];
    const magnitude = vectorLength(vector);
    if (magnitude <= 1.0e-6) {
      continue;
    }
    const start = liveTopology.nodes[nodeIndex].position;
    const direction = {
      x: vector.x / magnitude,
      y: vector.y / magnitude,
      z: vector.z / magnitude,
    };
    const scale = arrowScale * clamp(magnitude, 0.25, 1.0);
    arrows.push({
      node_index: nodeIndex,
      start,
      end: {
        x: start.x + direction.x * scale,
        y: start.y + direction.y * scale,
        z: start.z + direction.z * scale,
      },
      magnitude,
      color: vectorColor(fieldId, kind, vector),
    });
  }
  return { field_id: fieldId, kind, arrows };
}

function readLiveStats() {
  const values = liveRuntime.stats();
  return {
    step: values[0],
    time_seconds: values[1],
    node_count: values[2],
    edge_count: values[3],
    scalar_fields: values[4],
    vector_fields: values[5],
    active_perturbations: values[6],
    neighbor_links_visited: values[7],
    clamped_scalars: values[8],
    clamped_vectors: values[9],
    fixed_step_seconds: values[10],
  };
}

function computeBounds(payload) {
  const min = payload.bounds_min;
  const max = payload.bounds_max;
  const size = {
    x: Math.max(0.001, max.x - min.x),
    y: Math.max(0.001, max.y - min.y),
    z: Math.max(0.001, max.z - min.z),
  };
  return {
    min,
    max,
    size,
    center: {
      x: (min.x + max.x) * 0.5,
      y: (min.y + max.y) * 0.5,
      z: (min.z + max.z) * 0.5,
    },
    radius: Math.max(size.x, size.y, size.z, 0.001),
  };
}

function computeBoundsFromPositions(positions) {
  let min = { ...positions[0] };
  let max = { ...positions[0] };
  for (const position of positions) {
    min = {
      x: Math.min(min.x, position.x),
      y: Math.min(min.y, position.y),
      z: Math.min(min.z, position.z),
    };
    max = {
      x: Math.max(max.x, position.x),
      y: Math.max(max.y, position.y),
      z: Math.max(max.z, position.z),
    };
  }
  return computeBounds({ bounds_min: min, bounds_max: max });
}

function visualNodeRadius(topologyBounds) {
  return Math.max(topologyBounds.size.x, topologyBounds.size.y, topologyBounds.size.z, 1.0) * 0.026;
}

function project(point) {
  const scale = screenScale();
  return {
    x: canvas.width * 0.5 + (point.x - bounds.center.x) * scale + view.offsetX,
    y: canvas.height * 0.52 - (point.y - bounds.center.y) * scale + view.offsetY,
  };
}

function screenScale() {
  return (Math.min(canvas.width, canvas.height) * 0.74 * view.zoom) / bounds.radius;
}

function nodeScreenRadius(node) {
  return Math.max(4.5, node.radius * screenScale());
}

function frameIntervalSeconds() {
  if (isLiveMode()) {
    return liveStats?.fixed_step_seconds || 1 / 30;
  }
  if (sequence) {
    return Math.max(0.01, sequence.fixed_step_seconds * sequence.frame_stride);
  }
  return 1 / 30;
}

function playbackSpeed() {
  return Number(controls.speed.value) || 1;
}

function edgeColor(tier) {
  return tier === 1
    ? { r: 0.56, g: 0.62, b: 0.66, a: 0.45 }
    : { r: 0.38, g: 0.43, b: 0.47, a: 0.24 };
}

function scalarColor(fieldId, kind, value) {
  const key = `${fieldId} ${kind}`.toLowerCase();
  if (key.includes("wound")) {
    return lerpColor(
      { r: 0.38, g: 0.18, b: 0.14, a: 0.82 },
      { r: 1.0, g: 0.36, b: 0.24, a: 0.96 },
      value,
    );
  }
  if (key.includes("morphogen")) {
    return lerpColor(
      { r: 0.13, g: 0.29, b: 0.20, a: 0.82 },
      { r: 0.44, g: 0.84, b: 0.48, a: 0.96 },
      value,
    );
  }
  return lerpColor(
    { r: 0.25, g: 0.24, b: 0.42, a: 0.80 },
    { r: 0.72, g: 0.55, b: 0.92, a: 0.95 },
    value,
  );
}

function vectorColor(fieldId, kind, vector) {
  const key = `${fieldId} ${kind}`.toLowerCase();
  if (key.includes("polarity") && vector.x < 0.0) {
    return { r: 1.0, g: 0.77, b: 0.25, a: 0.94 };
  }
  return { r: 0.86, g: 0.88, b: 0.74, a: 0.90 };
}

function perturbationColor(effectKindValue) {
  switch (effectKindValue) {
    case "wound_region":
      return { r: 1.0, g: 0.26, b: 0.18, a: 0.34 };
    case "polarity_inversion":
      return { r: 1.0, g: 0.72, b: 0.22, a: 0.34 };
    case "depolarize_region":
      return { r: 0.78, g: 0.50, b: 0.96, a: 0.30 };
    case "coupling_multiplier_change":
      return { r: 0.42, g: 0.72, b: 0.56, a: 0.24 };
    default:
      return { r: 0.86, g: 0.86, b: 0.72, a: 0.24 };
  }
}

function effectKind(code) {
  switch (code) {
    case 1:
      return "wound_region";
    case 2:
      return "depolarize_region";
    case 3:
      return "polarity_inversion";
    case 4:
      return "coupling_multiplier_change";
    case 5:
      return "normal_polarity";
    default:
      return "custom";
  }
}

function targetField(code) {
  switch (code) {
    case 1:
      return "field.wound_signal";
    case 2:
      return "field.vmem_like";
    case 3:
      return "field.polarity";
    case 4:
      return "field.morphogen";
    default:
      return null;
  }
}

function vectorLength(vector) {
  return Math.hypot(vector.x, vector.y, vector.z);
}

function rgba(color) {
  const r = Math.round(clamp(color.r, 0, 1) * 255);
  const g = Math.round(clamp(color.g, 0, 1) * 255);
  const b = Math.round(clamp(color.b, 0, 1) * 255);
  const a = clamp(color.a, 0, 1);
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

function lerpColor(a, b, t) {
  const clamped = clamp(t, 0, 1);
  return {
    r: a.r + (b.r - a.r) * clamped,
    g: a.g + (b.g - a.g) * clamped,
    b: a.b + (b.b - a.b) * clamped,
    a: a.a + (b.a - a.a) * clamped,
  };
}

function formatNumber(value) {
  return Number(value).toFixed(2);
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}
