const canvas = document.querySelector("#viewport");
const ctx = canvas.getContext("2d");
const stats = document.querySelector("#stats");
const controls = {
  mesh: document.querySelector("#toggle-mesh"),
  coordinates: document.querySelector("#toggle-coordinates"),
  collider: document.querySelector("#toggle-collider"),
  sdf: document.querySelector("#toggle-sdf"),
  particles: document.querySelector("#toggle-particles"),
  liveParticles: document.querySelector("#toggle-live-particles"),
  reset: document.querySelector("#reset-view"),
  resetParticles: document.querySelector("#reset-particles"),
  playback: document.querySelector("#toggle-playback"),
};

let frame = null;
let runtimeSequence = null;
let runtimeFrameIndex = 0;
let runtimePlaybackPaused = false;
let runtimePlaybackAccumulator = 0;
let center = { x: 0, y: 0, z: 0 };
let radius = 1;
let view = { yaw: -0.28, pitch: -0.78, zoom: 1.0 };
let drag = null;
let liveParticleState = [];
let particleAnimation = null;
let lastParticleTimestamp = 0;
let particleResetIndex = 0;
let particleResetHoldSeconds = 0;

const params = new URLSearchParams(window.location.search);
const frameUrl = params.get("frame") || "/fixtures/hand_mesh/hand_mesh_browser_debug_frame.json";

fetch(frameUrl)
  .then((response) => {
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    return response.json();
  })
  .then((payload) => {
    loadPayload(payload);
    fitFrame();
    initializeParticleControls();
    updateStats();
    draw();
    startParticleAnimation();
  })
  .catch((error) => {
    stats.value = `Frame load failed: ${error.message}`;
    draw();
  });

function loadPayload(payload) {
  if (RealtimeHandSdf.isSurfaceSequence(payload)) {
    runtimeSequence = RealtimeHandSdf.normalizeSurfaceSequence(payload);
    runtimeFrameIndex = 0;
    runtimePlaybackPaused = false;
    runtimePlaybackAccumulator = 0;
    frame = RealtimeHandSdf.buildRuntimeFrame(runtimeSequence, runtimeFrameIndex);
    controls.playback.disabled = false;
    controls.playback.textContent = "Pause";
    return;
  }

  runtimeSequence = null;
  runtimeFrameIndex = 0;
  runtimePlaybackPaused = true;
  runtimePlaybackAccumulator = 0;
  frame = payload;
  controls.playback.disabled = true;
  controls.playback.textContent = "Pause";
}

for (const input of [
  controls.mesh,
  controls.coordinates,
  controls.collider,
  controls.sdf,
  controls.particles,
  controls.liveParticles,
]) {
  input.addEventListener("change", () => {
    if (input === controls.liveParticles) {
      if (controls.liveParticles.checked) {
        startParticleAnimation();
      } else if (!runtimeSequence || runtimePlaybackPaused) {
        stopParticleAnimation();
      }
    }
    draw();
  });
}

controls.reset.addEventListener("click", () => {
  view = { yaw: -0.28, pitch: -0.78, zoom: 1.0 };
  draw();
});

controls.resetParticles.addEventListener("click", () => {
  resetParticlesToSphere();
  updateStats();
  draw();
  startParticleAnimation();
});

controls.playback.addEventListener("click", () => {
  if (!runtimeSequence) {
    return;
  }
  runtimePlaybackPaused = !runtimePlaybackPaused;
  controls.playback.textContent = runtimePlaybackPaused ? "Play" : "Pause";
  if (!runtimePlaybackPaused) {
    startParticleAnimation();
  } else if (!controls.liveParticles.checked || liveParticleState.length === 0) {
    stopParticleAnimation();
  }
  updateStats();
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
  const min = runtimeSequence?.bounds_min || frame.mesh.bounds_min;
  const max = runtimeSequence?.bounds_max || frame.mesh.bounds_max;
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

function initializeParticleControls() {
  const liveReady = Boolean(runtimeSequence || frame?.particle_sdf_overlay?.sdf_grid);
  controls.liveParticles.disabled = !liveReady;
  controls.resetParticles.disabled = !liveReady;
  controls.playback.disabled = !runtimeSequence;
  if (liveReady) {
    resetParticlesToSphere();
  } else {
    controls.liveParticles.checked = false;
    liveParticleState = [];
  }
}

function updateStats() {
  const overlay = frame.particle_sdf_overlay;
  const frameLabel = runtimeSequence
    ? `frame ${runtimeFrameIndex + 1}/${runtimeSequence.frame_count}`
    : null;
  const items = [
    `${frame.mesh.vertices.length} vertices`,
    `${frame.mesh.triangles.length} triangles`,
    `${frame.coordinates.anchors.length} coordinates`,
    `${frame.sdf_slice.width}x${frame.sdf_slice.height} SDF`,
    frameLabel,
    runtimeSequence ? "runtime SDF" : null,
    runtimeSequence && runtimePlaybackPaused ? "paused" : null,
    overlay || runtimeSequence
      ? `${liveParticleState.length || overlay?.particles.samples.length || 0} particles`
      : null,
    runtimeSequence ? "random sphere reset" : overlay?.sdf_grid ? "sphere reset" : null,
  ];
  stats.value = items.filter(Boolean).join("  ");
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
  if (controls.particles.checked && (frame.particle_sdf_overlay || liveParticleState.length > 0)) {
    drawParticleOverlay(frame.particle_sdf_overlay);
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

function drawParticleOverlay(overlay) {
  if (liveParticleState.length > 0) {
    drawLiveParticleTrails();
    for (const particle of liveParticleState) {
      drawParticleMarker(particle.position, particle.radius, particle.color);
    }
    return;
  }

  drawLines(overlay.trails, 1.1);
  for (const sample of overlay.particles.samples) {
    drawParticleMarker(sample.position, sample.radius, sample.color);
  }
}

function drawParticleMarker(position, sourceRadius, color) {
  const projected = project(position);
  const radius = Math.max(2.8, sourceRadius * screenScale() * 1.35);
  ctx.fillStyle = rgba(color);
  ctx.beginPath();
  ctx.arc(projected.x, projected.y, radius, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = "rgba(235, 250, 255, 0.82)";
  ctx.lineWidth = Math.max(1, radius * 0.18);
  ctx.stroke();
}

function drawLiveParticleTrails() {
  for (const particle of liveParticleState) {
    for (let index = 1; index < particle.trail.length; index += 1) {
      const start = project(particle.trail[index - 1]);
      const end = project(particle.trail[index]);
      const t = index / Math.max(1, particle.trail.length - 1);
      ctx.strokeStyle = `rgba(58, 216, 255, ${0.10 + t * 0.34})`;
      ctx.lineWidth = (0.7 + t * 0.7) * (window.devicePixelRatio || 1);
      ctx.beginPath();
      ctx.moveTo(start.x, start.y);
      ctx.lineTo(end.x, end.y);
      ctx.stroke();
    }
  }
}

function resetParticlesToSphere() {
  if (runtimeSequence) {
    liveParticleState = RealtimeHandSdf.resetParticles(
      runtimeSequence,
      frame,
      particleResetIndex++,
    );
    particleResetHoldSeconds = 0.75;
    return;
  }

  const overlay = frame?.particle_sdf_overlay;
  const grid = overlay?.sdf_grid;
  if (!grid) {
    liveParticleState = [];
    return;
  }

  const bounds = sdfGridBounds(grid);
  const count = Math.max(1, overlay.initial_particle_count || overlay.particles.samples.length);
  const sourceRadius = overlay.particles.samples[0]?.radius || grid.voxel_size * 0.32;
  const sphereRadius = Math.min(bounds.size.x, bounds.size.y, bounds.size.z) * 0.42;
  const seed = particleResetIndex++;
  liveParticleState = [];
  for (let index = 0; index < count; index += 1) {
    const direction = fibonacciSphereDirection(index, count, seed);
    const radialJitter = 0.92 + unitHash(index, seed + 19) * 0.16;
    const position = add(bounds.center, scaleVector(direction, sphereRadius * radialJitter));
    const inwardVelocity = scaleVector(direction, -grid.voxel_size * 1.2);
    liveParticleState.push({
      position,
      velocity: inwardVelocity,
      radius: sourceRadius,
      color: { r: 0.20, g: 0.84, b: 1.0, a: 0.90 },
      trail: [position],
    });
  }
}

function startParticleAnimation() {
  if (particleAnimation !== null || !shouldAnimate()) {
    return;
  }
  lastParticleTimestamp = performance.now();
  particleAnimation = requestAnimationFrame(stepParticleAnimation);
}

function stopParticleAnimation() {
  if (particleAnimation !== null) {
    cancelAnimationFrame(particleAnimation);
    particleAnimation = null;
  }
}

function stepParticleAnimation(timestamp) {
  particleAnimation = null;
  if (!shouldAnimate()) {
    return;
  }

  const deltaSeconds = clamp((timestamp - lastParticleTimestamp) / 1000, 0.001, 0.05);
  lastParticleTimestamp = timestamp;
  const frameChanged = updateRuntimePlayback(deltaSeconds);
  if (controls.liveParticles.checked && liveParticleState.length > 0) {
    if (particleResetHoldSeconds > 0) {
      particleResetHoldSeconds = Math.max(0, particleResetHoldSeconds - deltaSeconds);
    } else {
      stepLiveParticles(deltaSeconds);
    }
  }
  if (frameChanged) {
    updateStats();
  }
  draw();
  if (shouldAnimate()) {
    particleAnimation = requestAnimationFrame(stepParticleAnimation);
  }
}

function shouldAnimate() {
  const playbackActive = Boolean(runtimeSequence && !runtimePlaybackPaused);
  const staticParticlesActive = Boolean(frame?.particle_sdf_overlay?.sdf_grid);
  const particlesActive = Boolean(
    controls.liveParticles.checked &&
    liveParticleState.length > 0 &&
    (runtimeSequence || staticParticlesActive),
  );
  return playbackActive || particlesActive;
}

function updateRuntimePlayback(deltaSeconds) {
  if (!runtimeSequence || runtimePlaybackPaused) {
    return false;
  }
  runtimePlaybackAccumulator += deltaSeconds;
  let advanced = false;
  while (runtimePlaybackAccumulator >= runtimeSequence.frame_seconds) {
    runtimePlaybackAccumulator -= runtimeSequence.frame_seconds;
    runtimeFrameIndex = (runtimeFrameIndex + 1) % runtimeSequence.frame_count;
    advanced = true;
  }
  if (advanced) {
    frame = RealtimeHandSdf.buildRuntimeFrame(runtimeSequence, runtimeFrameIndex);
  }
  return advanced;
}

function stepLiveParticles(deltaSeconds) {
  if (runtimeSequence) {
    RealtimeHandSdf.stepParticles(liveParticleState, frame, deltaSeconds);
    return;
  }

  const grid = frame.particle_sdf_overlay.sdf_grid;
  const bounds = sdfGridBounds(grid);
  const targetDistance = (liveParticleState[0]?.radius || grid.voxel_size * 0.32) * 0.45;
  const maxSpeed = grid.voxel_size * 26.0;
  const substeps = Math.max(1, Math.ceil(deltaSeconds / (1 / 90)));
  const subDelta = deltaSeconds / substeps;

  for (let substep = 0; substep < substeps; substep += 1) {
    for (const particle of liveParticleState) {
      const sample = sampleSdf(grid, particle.position);
      const gradient = sample ? sdfGradient(grid, sample.cell) : null;
      let acceleration = { x: 0, y: 0, z: 0 };
      if (sample && gradient) {
        const error = sample.distance - targetDistance;
        acceleration = add(acceleration, scaleVector(gradient, -error * 32.0));
      } else {
        acceleration = add(
          acceleration,
          scaleVector(normalize(subtract(bounds.center, particle.position)), grid.voxel_size * 18.0),
        );
      }

      particle.velocity = add(particle.velocity, scaleVector(acceleration, subDelta));
      particle.velocity = scaleVector(particle.velocity, Math.max(0, 1.0 - 2.8 * subDelta));
      particle.velocity = clampVectorLength(particle.velocity, maxSpeed);
      particle.position = add(particle.position, scaleVector(particle.velocity, subDelta));
      confineParticleToSdfBounds(particle, bounds, grid.voxel_size);
    }
  }

  for (const particle of liveParticleState) {
    particle.color = colorForParticle(grid, particle);
    particle.trail.push({ ...particle.position });
    while (particle.trail.length > 14) {
      particle.trail.shift();
    }
  }
}

function confineParticleToSdfBounds(particle, bounds, padding) {
  let bounced = false;
  for (const axis of ["x", "y", "z"]) {
    const min = bounds.min[axis] + padding;
    const max = bounds.max[axis] - padding;
    if (particle.position[axis] < min) {
      particle.position[axis] = min;
      particle.velocity[axis] = Math.abs(particle.velocity[axis]) * 0.35;
      bounced = true;
    } else if (particle.position[axis] > max) {
      particle.position[axis] = max;
      particle.velocity[axis] = -Math.abs(particle.velocity[axis]) * 0.35;
      bounced = true;
    }
  }
  if (bounced) {
    particle.trail = [particle.position];
  }
}

function colorForParticle(grid, particle) {
  const sample = sampleSdf(grid, particle.position);
  const distance01 = sample ? clamp(Math.abs(sample.distance) / (grid.voxel_size * 5.0), 0, 1) : 1;
  const speed01 = clamp(length(particle.velocity) / (grid.voxel_size * 22.0), 0, 1);
  return {
    r: 0.12 + speed01 * 0.35,
    g: 0.68 + (1 - distance01) * 0.24,
    b: 1.0,
    a: 0.82 + (1 - distance01) * 0.16,
  };
}

function sdfGridBounds(grid) {
  const size = {
    x: grid.dimensions[0] * grid.voxel_size,
    y: grid.dimensions[1] * grid.voxel_size,
    z: grid.dimensions[2] * grid.voxel_size,
  };
  const min = grid.origin;
  const max = add(min, size);
  return {
    min,
    max,
    size,
    center: scaleVector(add(min, max), 0.5),
  };
}

function sampleSdf(grid, point) {
  const cell = gridCellForPoint(grid, point);
  if (!cell) {
    return null;
  }
  const distance = distanceAt(grid, cell[0], cell[1], cell[2]);
  return distance === null ? null : { distance, cell };
}

function sdfGradient(grid, cell) {
  const [x, y, z] = cell;
  if (
    x <= 0 ||
    y <= 0 ||
    z <= 0 ||
    x >= grid.dimensions[0] - 1 ||
    y >= grid.dimensions[1] - 1 ||
    z >= grid.dimensions[2] - 1
  ) {
    return null;
  }
  const dx = distanceAt(grid, x + 1, y, z) - distanceAt(grid, x - 1, y, z);
  const dy = distanceAt(grid, x, y + 1, z) - distanceAt(grid, x, y - 1, z);
  const dz = distanceAt(grid, x, y, z + 1) - distanceAt(grid, x, y, z - 1);
  return normalize({ x: dx, y: dy, z: dz });
}

function gridCellForPoint(grid, point) {
  const local = {
    x: (point.x - grid.origin.x) / grid.voxel_size - 0.5,
    y: (point.y - grid.origin.y) / grid.voxel_size - 0.5,
    z: (point.z - grid.origin.z) / grid.voxel_size - 0.5,
  };
  const x = Math.round(local.x);
  const y = Math.round(local.y);
  const z = Math.round(local.z);
  if (
    x < 0 ||
    y < 0 ||
    z < 0 ||
    x >= grid.dimensions[0] ||
    y >= grid.dimensions[1] ||
    z >= grid.dimensions[2]
  ) {
    return null;
  }
  return [x, y, z];
}

function distanceAt(grid, x, y, z) {
  const index = (z * grid.dimensions[1] + y) * grid.dimensions[0] + x;
  return grid.distances[index] ?? null;
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

function fibonacciSphereDirection(index, count, seed) {
  const goldenAngle = Math.PI * (3 - Math.sqrt(5));
  const offset = 2 / count;
  const y = index * offset - 1 + offset * 0.5;
  const radius = Math.sqrt(Math.max(0, 1 - y * y));
  const theta = (index + seed * 0.37) * goldenAngle;
  return {
    x: Math.cos(theta) * radius,
    y,
    z: Math.sin(theta) * radius,
  };
}

function unitHash(index, seed) {
  let value = (index * 2654435761) ^ (seed * 2246822519);
  value ^= value >>> 16;
  value = Math.imul(value, 2246822507);
  value ^= value >>> 13;
  value = Math.imul(value, 3266489909);
  value ^= value >>> 16;
  return (value >>> 8) / 16777215;
}

function add(left, right) {
  return { x: left.x + right.x, y: left.y + right.y, z: left.z + right.z };
}

function subtract(left, right) {
  return { x: left.x - right.x, y: left.y - right.y, z: left.z - right.z };
}

function scaleVector(vector, scalar) {
  return { x: vector.x * scalar, y: vector.y * scalar, z: vector.z * scalar };
}

function length(vector) {
  return Math.sqrt(vector.x * vector.x + vector.y * vector.y + vector.z * vector.z);
}

function normalize(vector) {
  const vectorLength = length(vector);
  if (!Number.isFinite(vectorLength) || vectorLength <= 1e-8) {
    return { x: 0, y: 1, z: 0 };
  }
  return scaleVector(vector, 1 / vectorLength);
}

function clampVectorLength(vector, maxLength) {
  const vectorLength = length(vector);
  if (!Number.isFinite(vectorLength) || vectorLength <= maxLength) {
    return vector;
  }
  return scaleVector(vector, maxLength / vectorLength);
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}
