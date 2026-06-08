const canvas = document.querySelector("#viewport");
const ctx = canvas.getContext("2d");
const stats = document.querySelector("#stats");
const controls = {
  scalarLayer: document.querySelector("#scalar-layer"),
  edges: document.querySelector("#toggle-edges"),
  tier2: document.querySelector("#toggle-tier2"),
  regions: document.querySelector("#toggle-regions"),
  polarity: document.querySelector("#toggle-polarity"),
  labels: document.querySelector("#toggle-labels"),
  reset: document.querySelector("#reset-view"),
};

const params = new URLSearchParams(window.location.search);
const frameUrl = params.get("frame") || "/fixtures/fields/surface_field_visual_frame.json";
let frame = null;
let bounds = null;
let view = { zoom: 1, offsetX: 0, offsetY: 0 };
let drag = null;

fetch(frameUrl)
  .then((response) => {
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    return response.json();
  })
  .then((payload) => {
    frame = payload;
    bounds = computeBounds(frame);
    fillLayerSelect();
    resetView();
    updateStats();
    draw();
  })
  .catch((error) => {
    stats.value = `Frame load failed: ${error.message}`;
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

controls.reset.addEventListener("click", () => {
  resetView();
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

function fillLayerSelect() {
  controls.scalarLayer.innerHTML = "";
  for (const layer of frame.scalar_layers) {
    const option = document.createElement("option");
    option.value = layer.field_id;
    option.textContent = layer.field_id.replace("field.", "");
    controls.scalarLayer.append(option);
  }
  const wound = frame.scalar_layers.find((layer) => layer.field_id.includes("wound"));
  if (wound) {
    controls.scalarLayer.value = wound.field_id;
  }
}

function resetView() {
  view = { zoom: 1, offsetX: 0, offsetY: 0 };
}

function updateStats() {
  if (!frame) {
    return;
  }
  const layer = activeScalarLayer();
  stats.value = [
    `${frame.nodes.length} nodes`,
    `${frame.edges.length} edges`,
    `${frame.scalar_layers.length} scalar layers`,
    `${frame.vector_layers[0]?.arrows.length || 0} arrows`,
    layer ? `${layer.field_id}: ${formatNumber(layer.min_value)}..${formatNumber(layer.max_value)}` : null,
  ].filter(Boolean).join("  ");
}

function draw() {
  resizeCanvas();
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  drawBackground();
  if (!frame) {
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
    ctx.lineWidth = (edge.tier === 1 ? 1.25 : 0.75) * (window.devicePixelRatio || 1);
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(end.x, end.y);
    ctx.stroke();
  }
}

function drawRegions() {
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
  for (const layer of frame.vector_layers) {
    for (const arrow of layer.arrows) {
      const start = project(arrow.start);
      const end = project(arrow.end);
      ctx.strokeStyle = rgba(arrow.color);
      ctx.fillStyle = rgba(arrow.color);
      ctx.lineWidth = 1.8 * (window.devicePixelRatio || 1);
      ctx.beginPath();
      ctx.moveTo(start.x, start.y);
      ctx.lineTo(end.x, end.y);
      ctx.stroke();
      drawArrowHead(start, end, 7 * (window.devicePixelRatio || 1));
    }
  }
}

function drawLabels() {
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

function activeScalarLayer() {
  return frame?.scalar_layers.find((layer) => layer.field_id === controls.scalarLayer.value)
    || frame?.scalar_layers[0]
    || null;
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

function project(point) {
  const scale = screenScale();
  return {
    x: canvas.width * 0.5 + (point.x - bounds.center.x) * scale + view.offsetX,
    y: canvas.height * 0.52 - (point.y - bounds.center.y) * scale + view.offsetY,
  };
}

function screenScale() {
  return (Math.min(canvas.width, canvas.height) * 0.72 * view.zoom) / bounds.radius;
}

function nodeScreenRadius(node) {
  return Math.max(6, node.radius * screenScale());
}

function rgba(color) {
  const r = Math.round(clamp(color.r, 0, 1) * 255);
  const g = Math.round(clamp(color.g, 0, 1) * 255);
  const b = Math.round(clamp(color.b, 0, 1) * 255);
  const a = clamp(color.a, 0, 1);
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

function formatNumber(value) {
  return Number(value).toFixed(2);
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}
