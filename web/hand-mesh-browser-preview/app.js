const canvas = document.querySelector("#viewport");
const ctx = canvas.getContext("2d");
const stats = document.querySelector("#stats");
const controls = {
  mesh: document.querySelector("#toggle-mesh"),
  coordinates: document.querySelector("#toggle-coordinates"),
  collider: document.querySelector("#toggle-collider"),
  sdf: document.querySelector("#toggle-sdf"),
  reset: document.querySelector("#reset-view"),
};

let frame = null;
let center = { x: 0, y: 0, z: 0 };
let radius = 1;
let view = { yaw: -0.28, pitch: -0.78, zoom: 1.0 };
let drag = null;

const frameUrl = "/fixtures/hand_mesh/hand_mesh_browser_debug_frame.json";

fetch(frameUrl)
  .then((response) => {
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    return response.json();
  })
  .then((payload) => {
    frame = payload;
    fitFrame();
    updateStats();
    draw();
  })
  .catch((error) => {
    stats.value = `Frame load failed: ${error.message}`;
    draw();
  });

for (const input of [controls.mesh, controls.coordinates, controls.collider, controls.sdf]) {
  input.addEventListener("change", draw);
}

controls.reset.addEventListener("click", () => {
  view = { yaw: -0.28, pitch: -0.78, zoom: 1.0 };
  draw();
});

canvas.addEventListener("pointerdown", (event) => {
  canvas.setPointerCapture(event.pointerId);
  drag = { x: event.clientX, y: event.clientY, yaw: view.yaw, pitch: view.pitch };
});

canvas.addEventListener("pointermove", (event) => {
  if (!drag) {
    return;
  }
  view.yaw = drag.yaw + (event.clientX - drag.x) * 0.008;
  view.pitch = clamp(drag.pitch + (event.clientY - drag.y) * 0.008, -1.35, 1.15);
  draw();
});

canvas.addEventListener("pointerup", () => {
  drag = null;
});

canvas.addEventListener("wheel", (event) => {
  event.preventDefault();
  const nextZoom = view.zoom * (event.deltaY > 0 ? 0.92 : 1.08);
  view.zoom = clamp(nextZoom, 0.45, 3.5);
  draw();
}, { passive: false });

window.addEventListener("resize", draw);

function fitFrame() {
  const min = frame.mesh.bounds_min;
  const max = frame.mesh.bounds_max;
  center = {
    x: (min.x + max.x) * 0.5,
    y: (min.y + max.y) * 0.5,
    z: (min.z + max.z) * 0.5,
  };
  const dx = max.x - min.x;
  const dy = max.y - min.y;
  const dz = max.z - min.z;
  radius = Math.max(0.001, Math.sqrt(dx * dx + dy * dy + dz * dz) * 0.55);
}

function updateStats() {
  stats.value = [
    `${frame.mesh.vertices.length} vertices`,
    `${frame.mesh.triangles.length} triangles`,
    `${frame.coordinates.anchors.length} coordinates`,
    `${frame.sdf_slice.width}x${frame.sdf_slice.height} SDF`,
  ].join("  ");
}

function draw() {
  resizeCanvas();
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  drawBackground();
  if (!frame) {
    drawLoading();
    return;
  }
  if (controls.sdf.checked) {
    drawSdf();
  }
  if (controls.mesh.checked) {
    drawLines(frame.mesh.edges, 1.35);
  }
  if (controls.collider.checked) {
    drawLines(frame.collider.shell_edges, 1.1);
    drawLines(frame.collider.contact_normals, 2.0);
    drawPoints(frame.collider.contact_points);
  }
  if (controls.coordinates.checked) {
    drawLines(frame.coordinates.axes, 1.2);
    drawPoints(frame.coordinates.anchors);
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
  ctx.fillText("Loading mesh debug frame", 24, 36);
}

function drawLines(lines, width) {
  for (const line of lines) {
    const start = project(line.start);
    const end = project(line.end);
    ctx.strokeStyle = rgba(line.color);
    ctx.lineWidth = width * (window.devicePixelRatio || 1);
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(end.x, end.y);
    ctx.stroke();
  }
}

function drawPoints(points) {
  for (const point of points) {
    const projected = project(point.position);
    const size = Math.max(2.2, point.radius * screenScale() * 1.8);
    ctx.fillStyle = rgba(point.color);
    ctx.beginPath();
    ctx.arc(projected.x, projected.y, size, 0, Math.PI * 2);
    ctx.fill();
  }
}

function drawSdf() {
  const scale = Math.max(2, Math.min(7, screenScale() * frame.sdf_slice.width * 0.0018));
  for (const cell of frame.sdf_slice.cells) {
    const projected = project(cell.position);
    ctx.fillStyle = sdfColor(cell.normalized_distance);
    ctx.fillRect(projected.x - scale * 0.5, projected.y - scale * 0.5, scale, scale);
  }
}

function project(point) {
  const yaw = view.yaw;
  const pitch = view.pitch;
  const cy = Math.cos(yaw);
  const sy = Math.sin(yaw);
  const cp = Math.cos(pitch);
  const sp = Math.sin(pitch);
  const px = point.x - center.x;
  const py = point.y - center.y;
  const pz = point.z - center.z;
  const x1 = px * cy + pz * sy;
  const z1 = -px * sy + pz * cy;
  const y2 = py * cp - z1 * sp;
  const x = canvas.width * 0.5 + x1 * screenScale();
  const y = canvas.height * 0.56 - y2 * screenScale();
  return { x, y };
}

function screenScale() {
  return (Math.min(canvas.width, canvas.height) * 0.42 * view.zoom) / radius;
}

function rgba(color) {
  const r = Math.round(clamp(color.r, 0, 1) * 255);
  const g = Math.round(clamp(color.g, 0, 1) * 255);
  const b = Math.round(clamp(color.b, 0, 1) * 255);
  const a = clamp(color.a, 0, 1);
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

function sdfColor(value) {
  const t = clamp(value, 0, 1);
  const r = Math.round(32 + 196 * t);
  const g = Math.round(90 + 74 * (1 - Math.abs(t - 0.5) * 2));
  const b = Math.round(150 + 78 * (1 - t));
  return `rgba(${r}, ${g}, ${b}, 0.32)`;
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}
