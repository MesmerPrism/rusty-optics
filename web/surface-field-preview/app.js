const canvas = document.querySelector("#viewport");
const ctx = canvas.getContext("2d");
const viewport3d = document.querySelector("#viewport-3d");
const stats = document.querySelector("#stats");
const controls = {
  viewMode: document.querySelector("#view-mode"),
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
  planarian3dControls: document.querySelector("#planarian-3d-controls"),
  selectedNode: document.querySelector("#selected-node"),
  voltageDelta: document.querySelector("#voltage-delta"),
  voltageDeltaValue: document.querySelector("#voltage-delta-value"),
  applyVoltage: document.querySelector("#apply-voltage"),
  pulseCurrent: document.querySelector("#pulse-current"),
  setMemory: document.querySelector("#set-memory"),
  scaleGate: document.querySelector("#scale-gate"),
  editStatus: document.querySelector("#edit-status"),
  reset: document.querySelector("#reset-view"),
};

const params = new URLSearchParams(window.location.search);
const frameUrl = params.get("frame");
const sequenceUrl = params.get("sequence")
  || (frameUrl ? null : "/fixtures/fields/surface_field_visual_sequence.json");
const payloadUrl = sequenceUrl || frameUrl || "/fixtures/fields/surface_field_visual_frame.json";
const circuitUrl = params.get("circuit") || "/fixtures/fields/bioelectric_circuit_visual_frame.json";
const planarianUrl = params.get("planarian") || "/fixtures/fields/planarian_bioelectric_visual_sequence.json";
const initialView = params.get("view");
const wasmBaseUrl = params.get("wasm") || "/local-artifacts/matter_surface_field_wasm";
const threeModuleUrl = params.get("three")
  || new URL("/local-artifacts/web3d/three.module.js", window.location.href).href;
const PLANARIAN_EDIT_INTENT_SCHEMA_ID = "rusty.optics.fields.planarian_bioelectric.edit_intent.v1";
const PLANARIAN_3D_VISUAL_ID = "fields.visual.planarian3d.live";
const PLANARIAN_3D_SURFACE_ID = "mesh.planarian_ap.sketchfab_educational_surface";
const PLANARIAN_3D_SUBSTRATE_ID = "fields.substrate.planarian_ap.sketchfab_educational";

let sequence = null;
let frames = [];
let circuitFrame = null;
let planarianSequence = null;
let planarianFrames = [];
let currentFrameIndex = 0;
let currentPlanarianIndex = 0;
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
let matterWasmModule = null;
let planarian3dRuntime = null;
let planarian3dView = null;
let planarian3dStats = null;
let planarian3dStepMs = 0;
let selectedPlanarianNode = null;
let selectedPlanarianPick = null;
let lastPlanarianIntent = null;
let lastPlanarianEdit = null;
let planarian3dError = null;

updateVoltageDeltaLabel();

Promise.all([
  fetchJson(payloadUrl),
  fetchJson(circuitUrl).catch(() => null),
  fetchJson(planarianUrl).catch(() => null),
])
  .then(async ([payload, circuitPayload, planarianPayload]) => {
    loadPayload(payload);
    if (circuitPayload) {
      loadCircuitPayload(circuitPayload);
    }
    if (planarianPayload) {
      loadPlanarianPayload(planarianPayload);
    }
    applyInitialViewMode();
    await initLiveRuntime();
    requestAnimationFrame(animationLoop);
  })
  .catch((error) => {
    stats.textContent = `Frame load failed: ${error.message}`;
    setPlaying(false);
    draw();
  });

controls.viewMode.addEventListener("change", () => {
  if (isCircuitMode() && !circuitFrame) {
    controls.viewMode.value = "surface";
    return;
  }
  if (isPlanarian3DRequested() && !planarian3dView) {
    controls.viewMode.value = "planarian";
    return;
  }
  if (isPlanarianRequested() && !planarianSequence) {
    controls.viewMode.value = "surface";
    return;
  }
  if (isCircuitMode()) {
    controls.live.checked = false;
    bounds = computeBounds(circuitFrame);
    setPlaying(false);
  } else if (isPlanarianMode()) {
    controls.live.checked = false;
    bounds = computeBounds(activeFrame());
    setPlaying(planarianFrames.length > 1);
  } else if (isPlanarian3DMode()) {
    controls.live.checked = false;
    setPlaying(true);
  } else {
    if (liveRuntime) {
      controls.live.checked = true;
    }
    bounds = computeBounds(activeFrame());
    setPlaying(canAnimate());
  }
  fillLayerSelect();
  updateControlAvailability();
  updateTimelineControls();
  updateRuntimeStatus(isLiveMode() ? "live" : "sequence");
  updateStats();
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
    if (isPlanarian3DMode()) {
      updatePlanarian3DView();
    }
    updateStats();
    draw();
  });
}

controls.live.addEventListener("change", () => {
  if (isCircuitMode() || isPlanarianMode() || isPlanarian3DMode()) {
    controls.live.checked = false;
    return;
  }
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
  if (isPlanarianMode()) {
    currentPlanarianIndex = Number(controls.frameSlider.value);
    lastAnimationTime = null;
    frameAccumulatorMs = 0;
    updateTimelineControls();
    updateStats();
    draw();
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
  } else if (isPlanarian3DMode()) {
    resetPlanarian3D();
  }
  resetView();
  updateTimelineControls();
  updateStats();
  draw();
});

controls.voltageDelta.addEventListener("input", () => {
  updateVoltageDeltaLabel();
});

controls.applyVoltage.addEventListener("click", () => {
  const delta = Number(controls.voltageDelta.value) || 0;
  applyPlanarian3DEdit(
    { AddNodeVoltage: { delta } },
    (runtime, nodeIndex) => runtime.add_node_voltage(nodeIndex, delta),
  );
});

controls.pulseCurrent.addEventListener("click", () => {
  applyPlanarian3DEdit(
    { AddTransientCurrent: { current: 0.42, duration_steps: 72 } },
    (runtime, nodeIndex) => runtime.add_transient_current(nodeIndex, 0.42, 72),
  );
});

controls.setMemory.addEventListener("click", () => {
  applyPlanarian3DEdit(
    { SetNodeMemory: { memory_value: 1.0 } },
    (runtime, nodeIndex) => runtime.set_node_memory(nodeIndex, 1.0),
  );
});

controls.scaleGate.addEventListener("click", () => {
  applyPlanarian3DEdit(
    { ScaleIncidentConductance: { scale: 0.5 } },
    (runtime, nodeIndex) => runtime.scale_incident_conductance(nodeIndex, 0.5),
  );
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

function fetchJson(url) {
  return fetch(url).then((response) => {
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    return response.json();
  });
}

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
  updateControlAvailability();
  updateTimelineControls();
  updateRuntimeStatus("sequence");
  updateStats();
  draw();
}

function loadCircuitPayload(payload) {
  circuitFrame = payload;
  if (!circuitFrame?.schema_id?.includes("bioelectric_circuit.visual_frame")) {
    circuitFrame = null;
    return;
  }
  updateControlAvailability();
}

function loadPlanarianPayload(payload) {
  if (!payload?.schema_id?.includes("planarian_bioelectric.visual_sequence") || !Array.isArray(payload.frames)) {
    planarianSequence = null;
    planarianFrames = [];
    return;
  }
  planarianSequence = payload;
  planarianFrames = payload.frames;
  currentPlanarianIndex = 0;
  updateControlAvailability();
}

function applyInitialViewMode() {
  if (initialView === "planarian3d" && planarian3dView) {
    controls.viewMode.value = "planarian3d";
    controls.live.checked = false;
    setPlaying(true);
    fillLayerSelect();
    updateControlAvailability();
    updateTimelineControls();
    updateRuntimeStatus("planarian3d");
    updateStats();
  } else if (initialView === "planarian" && planarianSequence) {
    controls.viewMode.value = "planarian";
    controls.live.checked = false;
    bounds = computeBounds(activeFrame());
    setPlaying(planarianFrames.length > 1);
    fillLayerSelect();
    updateControlAvailability();
    updateTimelineControls();
    updateRuntimeStatus("planarian");
    updateStats();
  } else if (initialView === "circuit" && circuitFrame) {
    controls.viewMode.value = "circuit";
    controls.live.checked = false;
    bounds = computeBounds(circuitFrame);
    setPlaying(false);
    fillLayerSelect();
    updateControlAvailability();
    updateTimelineControls();
    updateRuntimeStatus("circuit");
    updateStats();
  }
}

async function initLiveRuntime() {
  try {
    const module = await import(`${wasmBaseUrl}/rusty_matter_fields_wasm.js`);
    await module.default(`${wasmBaseUrl}/rusty_matter_fields_wasm_bg.wasm`);
    matterWasmModule = module;
    liveRuntime = new module.SurfaceFieldRealtimeRuntime();
    liveTopology = decodeLiveTopology(liveRuntime);
    liveFrame = buildLiveFrame();
    await initPlanarian3D();
    if (initialView) {
      applyInitialViewMode();
    }
    controls.live.checked = !isCircuitMode() && !isPlanarianMode() && !isPlanarian3DMode();
    bounds = computeBounds(activeFrame());
    fillLayerSelect();
    setPlaying(canAnimate());
    updateControlAvailability();
    updateTimelineControls();
    updateRuntimeStatus("live");
    updateStats();
    draw();
  } catch (_error) {
    matterWasmModule = null;
    liveRuntime = null;
    liveTopology = null;
    liveFrame = null;
    controls.live.checked = false;
    controls.live.disabled = true;
    updateRuntimeStatus("sequence");
  }
}

async function initPlanarian3D() {
  if (!matterWasmModule?.PlanarianBioelectricRealtimeRuntime) {
    planarian3dError = "missing Matter planarian runtime";
    return;
  }
  try {
    const module = await import("./planarian-3d.js");
    planarian3dRuntime = new matterWasmModule.PlanarianBioelectricRealtimeRuntime();
    planarian3dView = await module.createPlanarianBioelectric3DView({
      container: viewport3d,
      runtime: planarian3dRuntime,
      threeModuleUrl,
      visualId: PLANARIAN_3D_VISUAL_ID,
      surfaceId: PLANARIAN_3D_SURFACE_ID,
      substrateId: PLANARIAN_3D_SUBSTRATE_ID,
      getViewRevision: () => planarian3dStats?.revision ?? null,
      onSelectNode: (selection) => {
        selectedPlanarianPick = selection;
        selectedPlanarianNode = selection?.target?.SurfaceNode?.node_index ?? null;
        lastPlanarianIntent = null;
        updatePlanarian3DSelection();
        updateStats();
      },
    });
    planarian3dStats = readPlanarian3DStats();
    updatePlanarian3DView();
    planarian3dError = null;
  } catch (error) {
    planarian3dRuntime = null;
    planarian3dView = null;
    planarian3dError = error.message;
    console.warn(`Planarian 3D unavailable: ${planarian3dError}`);
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
      } else if (isPlanarian3DMode()) {
        const steps = Math.min(8, Math.floor(frameAccumulatorMs / intervalMs));
        if (steps > 0) {
          frameAccumulatorMs -= steps * intervalMs;
          const started = performance.now();
          planarian3dRuntime.step(steps);
          planarian3dStepMs = performance.now() - started;
          updatePlanarian3DView();
          updateTimelineControls();
          updateStats();
          draw();
        }
      } else if (isPlanarianMode()) {
        while (frameAccumulatorMs >= intervalMs) {
          frameAccumulatorMs -= intervalMs;
          currentPlanarianIndex = (currentPlanarianIndex + 1) % planarianFrames.length;
        }
        updateTimelineControls();
        updateStats();
        draw();
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
  if (isPlanarian3DMode()) {
    return Boolean(planarian3dRuntime && planarian3dView);
  }
  if (isPlanarianMode()) {
    return planarianFrames.length > 1;
  }
  return !isCircuitMode() && (isLiveMode() || frames.length > 1);
}

function isLiveMode() {
  return Boolean(!isCircuitMode() && controls.live.checked && liveRuntime && liveFrame);
}

function isCircuitMode() {
  return controls.viewMode.value === "circuit" && circuitFrame;
}

function isPlanarianRequested() {
  return controls.viewMode.value === "planarian";
}

function isPlanarianMode() {
  return isPlanarianRequested() && planarianSequence && planarianFrames.length > 0;
}

function isPlanarian3DRequested() {
  return controls.viewMode.value === "planarian3d";
}

function isPlanarian3DMode() {
  return isPlanarian3DRequested() && planarian3dRuntime && planarian3dView;
}

function updateControlAvailability() {
  const circuit = isCircuitMode();
  const planarian = isPlanarianMode();
  const planarian3d = isPlanarian3DMode();
  controls.viewMode.querySelector('option[value="circuit"]').disabled = !circuitFrame;
  controls.viewMode.querySelector('option[value="planarian"]').disabled = !planarianSequence;
  controls.viewMode.querySelector('option[value="planarian3d"]').disabled = !planarian3dView;
  controls.live.disabled = circuit || planarian || planarian3d || !liveRuntime;
  controls.polarity.disabled = circuit || planarian || planarian3d;
  controls.planarian3dControls.hidden = !planarian3d;
  controls.applyVoltage.disabled = !planarian3d || selectedPlanarianNode === null;
  controls.pulseCurrent.disabled = controls.applyVoltage.disabled;
  controls.setMemory.disabled = controls.applyVoltage.disabled;
  controls.scaleGate.disabled = controls.applyVoltage.disabled;
}

function fillLayerSelect() {
  const selected = controls.scalarLayer.value;
  controls.scalarLayer.innerHTML = "";
  if (isPlanarian3DMode()) {
    addLayerOption("circuit.voltage", "voltage");
    addLayerOption("circuit.memory", "memory");
    addLayerOption("readout:readout.planarian_ap.head_identity", "planarian_ap.head_identity");
    addLayerOption("readout:readout.planarian_ap.tail_identity", "planarian_ap.tail_identity");
    if ([...controls.scalarLayer.options].some((option) => option.value === selected)) {
      controls.scalarLayer.value = selected;
    }
    updatePlanarian3DView();
    return;
  }
  if (isCircuitMode() || isPlanarianMode()) {
    const frame = activeCircuitFrame();
    addLayerOption("circuit.voltage", "voltage");
    if (frame.memory_samples.length > 0) {
      addLayerOption("circuit.memory", "memory");
    }
    for (const layer of frame.readout_layers) {
      addLayerOption(`readout:${layer.layer_id}`, layer.layer_id.replace("readout.", ""));
    }
    if ([...controls.scalarLayer.options].some((option) => option.value === selected)) {
      controls.scalarLayer.value = selected;
    }
    return;
  }
  const frame = activeFrame();
  for (const layer of frame.scalar_layers) {
    addLayerOption(layer.field_id, layer.field_id.replace("field.", ""));
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

function addLayerOption(value, label) {
  const option = document.createElement("option");
  option.value = value;
  option.textContent = label;
  controls.scalarLayer.append(option);
}

function resetView() {
  view = { zoom: 1, offsetX: 0, offsetY: 0 };
}

function updateTimelineControls() {
  if (isPlanarian3DMode()) {
    controls.frameSlider.disabled = true;
    controls.frameSlider.max = "0";
    controls.frameSlider.value = "0";
    controls.framePosition.textContent = `live ${Math.trunc(planarian3dStats?.step || 0)}`;
    return;
  }
  if (isPlanarianMode()) {
    const frameCount = Math.max(1, planarianFrames.length);
    controls.frameSlider.max = String(frameCount - 1);
    controls.frameSlider.value = String(currentPlanarianIndex);
    controls.frameSlider.disabled = frameCount <= 1;
    controls.framePosition.textContent = `${currentPlanarianIndex + 1}/${frameCount}`;
    return;
  }
  if (isCircuitMode()) {
    controls.frameSlider.disabled = true;
    controls.frameSlider.max = "0";
    controls.frameSlider.value = "0";
    controls.framePosition.textContent = "circuit";
    return;
  }
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
  if (isPlanarian3DMode()) {
    controls.runtimeStatus.textContent = "Matter 3D";
  } else if (isPlanarianMode()) {
    controls.runtimeStatus.textContent = "planarian";
  } else if (isCircuitMode()) {
    controls.runtimeStatus.textContent = "circuit";
  } else {
    controls.runtimeStatus.textContent = mode === "live" ? "Matter live" : "sequence";
  }
}

function updateStats() {
  if (isPlanarian3DMode()) {
    updatePlanarian3DStats();
    return;
  }
  if (isPlanarianMode()) {
    updatePlanarianStats();
    return;
  }
  if (isCircuitMode()) {
    updateCircuitStats();
    return;
  }
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

function updateCircuitStats() {
  const frame = activeCircuitFrame();
  const diagnostics = frame.diagnostics;
  const layer = controls.scalarLayer.value || "circuit.voltage";
  const layerLabel = layer.replace("circuit.", "").replace("readout:", "");
  stats.textContent = [
    `${frame.nodes.length} nodes`,
    `${frame.conductance_edges.length} conductance edges`,
    `${frame.current_regions.length} current terms`,
    `${frame.memory_samples.length} memory samples`,
    `${frame.readout_layers.length} readouts`,
    `t ${formatNumber(frame.time_seconds)}s`,
    diagnostics ? `step ${diagnostics.step_index}` : null,
    diagnostics ? `${diagnostics.active_gates} gates` : null,
    diagnostics ? `dV ${formatNumber(diagnostics.max_voltage_delta)}` : null,
    layerLabel,
  ].filter(Boolean).join("  ");
}

function updatePlanarianStats() {
  const frame = activeCircuitFrame();
  const diagnostics = frame.diagnostics;
  const layer = controls.scalarLayer.value || "circuit.voltage";
  const layerLabel = layer.replace("circuit.", "").replace("readout:", "");
  stats.textContent = [
    "planarian AP",
    `${frame.nodes.length} nodes`,
    `${frame.conductance_edges.length} conductance edges`,
    `${planarianFrames.length} frames`,
    `${planarianSequence.region_bands.length} AP regions`,
    `step ${diagnostics?.step_index ?? 0}/${planarianSequence.step_count}`,
    `t ${formatNumber(frame.time_seconds)}s`,
    `${frame.memory_samples.length} memory`,
    `${frame.readout_layers.length} readouts`,
    diagnostics ? `dV ${formatNumber(diagnostics.max_voltage_delta)}` : null,
    layerLabel,
  ].filter(Boolean).join("  ");
}

function updatePlanarian3DStats() {
  const layer = controls.scalarLayer.value || "circuit.voltage";
  const layerLabel = layer.replace("circuit.", "").replace("readout:", "");
  const selected = selectedPlanarianNode === null ? "none" : selectedPlanarianNode;
  const edit = lastPlanarianEdit
    ? `edit ${lastPlanarianEdit.accepted ? "accepted" : "rejected"} r${lastPlanarianEdit.revision_after}`
    : "no edit";
  stats.textContent = [
    "planarian 3D",
    `${Math.trunc(planarian3dStats?.body_vertex_count || 0)} body vertices`,
    `${Math.trunc(planarian3dStats?.body_triangle_count || 0)} body triangles`,
    `${Math.trunc(planarian3dStats?.node_count || 0)} nodes`,
    `${Math.trunc(planarian3dStats?.edge_count || 0)} conductance edges`,
    `step ${Math.trunc(planarian3dStats?.step || 0)}`,
    `revision ${Math.trunc(planarian3dStats?.revision || 0)}`,
    `t ${formatNumber(planarian3dStats?.time_seconds || 0)}s`,
    `${formatNumber(planarian3dStepMs)}ms`,
    `${Math.trunc(planarian3dStats?.active_gates || 0)} gates`,
    `node ${selected}`,
    layerLabel,
    edit,
  ].join("  ");
}

function draw() {
  updateViewportVisibility();
  if (isPlanarian3DMode()) {
    planarian3dView.render();
    return;
  }
  resizeCanvas();
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  drawBackground();
  if (!activeFrame()) {
    drawLoading();
    return;
  }
  if (isPlanarianMode()) {
    drawPlanarianFrame();
    return;
  }
  if (isCircuitMode()) {
    drawCircuitFrame();
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

function updateViewportVisibility() {
  const show3d = isPlanarian3DMode();
  canvas.hidden = show3d;
  viewport3d.hidden = !show3d;
}

function drawCircuitFrame() {
  if (controls.edges.checked) {
    drawCircuitConductanceEdges();
  }
  if (controls.regions.checked) {
    drawCircuitCurrentRegions();
  }
  drawCircuitNodes();
  if (controls.labels.checked) {
    drawLabels();
  }
}

function drawPlanarianFrame() {
  if (controls.regions.checked) {
    drawPlanarianAxisBands();
  }
  if (controls.edges.checked) {
    drawCircuitConductanceEdges();
  }
  if (controls.regions.checked) {
    drawCircuitCurrentRegions();
  }
  drawCircuitNodes();
  if (controls.labels.checked) {
    drawPlanarianLabels();
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

function drawCircuitConductanceEdges() {
  const frame = activeCircuitFrame();
  ctx.lineCap = "round";
  const scale = window.devicePixelRatio || 1;
  for (const edge of frame.conductance_edges) {
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
    ctx.lineWidth = (0.65 + edge.normalized_conductance * 2.2) * scale;
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

function drawCircuitCurrentRegions() {
  const frame = activeCircuitFrame();
  const scale = window.devicePixelRatio || 1;
  for (const region of frame.current_regions) {
    if (region.all_nodes) {
      continue;
    }
    ctx.fillStyle = rgba(region.color);
    ctx.strokeStyle = rgba({ ...region.color, a: Math.min(0.72, region.color.a * 1.9) });
    ctx.lineWidth = 1.0 * scale;
    for (const nodeIndex of region.node_indices) {
      const node = frame.nodes[nodeIndex];
      if (!node) {
        continue;
      }
      const point = project(node.position);
      const radius = nodeScreenRadius(node) * 2.0;
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

function drawCircuitNodes() {
  const frame = activeCircuitFrame();
  const samples = activeCircuitSamples();
  if (!samples.length) {
    return;
  }
  for (const sample of samples) {
    const node = frame.nodes[sample.node_index];
    if (!node) {
      continue;
    }
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

function drawPlanarianAxisBands() {
  if (!planarianSequence) {
    return;
  }
  const scale = window.devicePixelRatio || 1;
  for (const band of planarianSequence.region_bands) {
    const left = project({ x: 0, y: 0, z: band.z_min });
    const right = project({ x: 0, y: 0, z: band.z_max });
    const x0 = Math.min(left.x, right.x);
    const x1 = Math.max(left.x, right.x);
    ctx.fillStyle = rgba({ ...band.color, a: Math.min(0.18, band.color.a * 0.45) });
    ctx.fillRect(x0, 0, Math.max(1, x1 - x0), canvas.height);
    ctx.strokeStyle = rgba({ ...band.color, a: Math.min(0.42, band.color.a * 1.4) });
    ctx.lineWidth = 1 * scale;
    ctx.beginPath();
    ctx.moveTo(x0, 0);
    ctx.lineTo(x0, canvas.height);
    ctx.stroke();
  }
}

function drawPlanarianLabels() {
  drawPlanarianRegionLabels();
  drawLabels();
}

function drawPlanarianRegionLabels() {
  if (!planarianSequence) {
    return;
  }
  const scale = window.devicePixelRatio || 1;
  ctx.font = `${12 * scale}px sans-serif`;
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  for (const band of planarianSequence.region_bands) {
    const centerZ = (band.z_min + band.z_max) * 0.5;
    const point = project({ x: 0, y: 0, z: centerZ });
    ctx.fillStyle = rgba({ ...band.color, a: 0.86 });
    ctx.fillText(band.label, point.x, 10 * scale);
  }
  ctx.textAlign = "start";
  ctx.textBaseline = "alphabetic";
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
  if (isPlanarianMode()) {
    return planarianFrames[currentPlanarianIndex] || planarianFrames[0] || null;
  }
  if (isCircuitMode()) {
    return circuitFrame;
  }
  if (isLiveMode()) {
    return liveFrame;
  }
  return frames[currentFrameIndex] || frames[0] || null;
}

function activeCircuitFrame() {
  if (isPlanarianMode()) {
    return planarianFrames[currentPlanarianIndex] || planarianFrames[0] || null;
  }
  return circuitFrame;
}

function activeScalarLayer() {
  if (isCircuitMode()) {
    return null;
  }
  const frame = activeFrame();
  return frame?.scalar_layers.find((layer) => layer.field_id === controls.scalarLayer.value)
    || frame?.scalar_layers[0]
    || null;
}

function activeCircuitSamples() {
  const frame = activeCircuitFrame();
  if (!frame) {
    return [];
  }
  const selected = controls.scalarLayer.value;
  if (selected === "circuit.memory") {
    return frame.memory_samples;
  }
  if (selected.startsWith("readout:")) {
    const layerId = selected.slice("readout:".length);
    return frame.readout_layers.find((layer) => layer.layer_id === layerId)?.samples || [];
  }
  return frame.voltage_samples;
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

function readPlanarian3DStats() {
  const values = planarian3dRuntime.stats();
  return {
    step: values[0],
    time_seconds: values[1],
    revision: values[2],
    node_count: values[3],
    edge_count: values[4],
    current_terms: values[5],
    active_current_terms: values[6],
    active_gates: values[7],
    clamped_voltage_nodes: values[8],
    max_voltage_delta: values[9],
    fixed_step_seconds: values[10],
    last_edit_accepted: values[11],
    last_edit_revision_after: values[12],
    body_vertex_count: planarian3dRuntime.body_vertex_count(),
    body_triangle_count: planarian3dRuntime.body_triangle_count(),
  };
}

function decodePlanarianEditResult(values) {
  return {
    accepted: values[0] > 0.5,
    revision_before: values[1],
    revision_after: values[2],
    clamped_values: values[3],
    affected_nodes: values[4],
    affected_edges: values[5],
    affected_currents: values[6],
  };
}

function updatePlanarian3DView() {
  if (!planarian3dRuntime || !planarian3dView) {
    return;
  }
  planarian3dStats = readPlanarian3DStats();
  planarian3dView.updateSnapshot(
    planarian3dRuntime.snapshot(),
    planarian3dRuntime.conductance_values(),
    controls.scalarLayer.value || "circuit.voltage",
  );
  planarian3dView.setVisibility(controls.edges.checked, controls.tier2.checked);
  planarian3dView.render();
}

function resetPlanarian3D() {
  if (!planarian3dRuntime || !planarian3dView) {
    return;
  }
  planarian3dRuntime.reset();
  selectedPlanarianNode = null;
  selectedPlanarianPick = null;
  lastPlanarianIntent = null;
  lastPlanarianEdit = null;
  planarian3dStepMs = 0;
  planarian3dView.selectNode(null);
  updatePlanarian3DSelection();
  updatePlanarian3DView();
}

function applyPlanarian3DEdit(operation, apply) {
  if (!isPlanarian3DMode() || selectedPlanarianNode === null || !selectedPlanarianPick) {
    return;
  }
  const intent = buildPlanarianEditIntent(operation);
  if (!intent) {
    return;
  }
  const resultValues = apply(planarian3dRuntime, selectedPlanarianNode);
  lastPlanarianEdit = decodePlanarianEditResult(resultValues);
  lastPlanarianIntent = intent;
  updatePlanarian3DView();
  updatePlanarian3DSelection();
  updateStats();
}

function buildPlanarianEditIntent(operation) {
  const nodeTarget = selectedPlanarianPick?.target?.SurfaceNode;
  if (!nodeTarget) {
    return null;
  }
  const revision = Math.trunc(planarian3dStats?.revision ?? 0);
  return {
    schema_id: PLANARIAN_EDIT_INTENT_SCHEMA_ID,
    intent_id: [
      PLANARIAN_3D_VISUAL_ID,
      "intent",
      operationKind(operation),
      `node_${String(nodeTarget.node_index).padStart(4, "0")}`,
      `r${revision}`,
    ].join("."),
    selection_id: selectedPlanarianPick.selection_id,
    visual_id: selectedPlanarianPick.visual_id,
    surface_id: selectedPlanarianPick.surface_id,
    substrate_id: selectedPlanarianPick.substrate_id,
    expected_revision: revision,
    target: {
      SurfaceNode: {
        node_index: nodeTarget.node_index,
        node_id: nodeTarget.node_id,
      },
    },
    operation,
  };
}

function updatePlanarian3DSelection() {
  controls.selectedNode.textContent = selectedPlanarianNode === null ? "none" : String(selectedPlanarianNode);
  if (planarian3dView) {
    planarian3dView.selectNode(selectedPlanarianNode);
  }
  if (lastPlanarianEdit) {
    controls.editStatus.textContent = [
      lastPlanarianIntent ? operationKind(lastPlanarianIntent.operation) : null,
      lastPlanarianEdit.accepted ? "accepted" : "rejected",
      `r${Math.trunc(lastPlanarianEdit.revision_before)}->${Math.trunc(lastPlanarianEdit.revision_after)}`,
      lastPlanarianEdit.clamped_values > 0 ? `${Math.trunc(lastPlanarianEdit.clamped_values)} clamped` : null,
    ].filter(Boolean).join(" ");
  } else {
    controls.editStatus.textContent = "no edit";
  }
  updateControlAvailability();
}

function updateVoltageDeltaLabel() {
  const value = Number(controls.voltageDelta.value) || 0;
  controls.voltageDeltaValue.textContent = `${value >= 0 ? "+" : ""}${value.toFixed(2)}`;
}

function operationKind(operation) {
  return Object.keys(operation || {})[0] || "Unknown";
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
  const projected = projectedPoint(point);
  const center = projectedCenter();
  return {
    x: canvas.width * 0.5 + (projected.x - center.x) * scale + view.offsetX,
    y: canvas.height * 0.52 - (projected.y - center.y) * scale + view.offsetY,
  };
}

function projectedPoint(point) {
  if (isPlanarianMode()) {
    return { x: point.z, y: point.x };
  }
  return { x: point.x, y: point.y };
}

function projectedCenter() {
  if (isPlanarianMode()) {
    return { x: bounds.center.z, y: bounds.center.x };
  }
  return { x: bounds.center.x, y: bounds.center.y };
}

function screenScale() {
  return (Math.min(canvas.width, canvas.height) * 0.74 * view.zoom) / bounds.radius;
}

function nodeScreenRadius(node) {
  return Math.max(4.5, node.radius * screenScale());
}

function frameIntervalSeconds() {
  if (isPlanarian3DMode()) {
    return planarian3dStats?.fixed_step_seconds || 1 / 30;
  }
  if (isPlanarianMode()) {
    return Math.max(0.01, planarianSequence.fixed_step_seconds * planarianSequence.frame_stride);
  }
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
