import {
  applyCanvasTuning,
  applyTuningToProfile,
  createDefaultTuning,
  createStimulusTuningPanel,
  loadTuningFromHash,
  randomizeTuning,
  tuningToHash,
} from "./tuning.js";
import {
  createBrowserPreset,
  createQuestHandoff,
  downloadJson,
  loadStoredPresets,
  questHandoffFilename,
  saveStoredPreset,
} from "./handoff.js";

const PROFILE_QUERY_KEY = "profile";
const MAX_LAYERS = 16;
const LAYER_STRIDE = 48;
const TARGET_FPS = 60;

const PATTERN = {
  Stripes: 0,
  Rings: 1,
  Rays: 2,
  Checkerboard: 3,
  NoiseField: 4,
  PerlinNoise: 5,
  Ripple: 6,
  Interference: 7,
};

const BLEND = {
  Add: 0,
  Multiply: 1,
  Max: 2,
};

const LAYER_BLEND = {
  Stack: 0,
  Mean: 1,
  Cross: 2,
};

const MIRROR = {
  None: 0,
  Horizontal: 1,
  Vertical: 2,
  Axes: 3,
  Kaleidoscope3: 4,
  Kaleidoscope6: 5,
};

const NOISE = {
  CellValue: 0,
  SmoothValue: 1,
  PerlinGradient: 2,
};

const OFF = {
  pattern: 0,
  blend: 1,
  mirror: 2,
  seed: 3,
  weight: 4,
  opacity: 5,
  frequency: 6,
  rotation: 7,
  phase: 8,
  temporalSpeed: 9,
  temporalPhase: 10,
  temporalAmplitude: 11,
  warpScaleX: 12,
  warpScaleY: 13,
  warpOffsetX: 14,
  warpOffsetY: 15,
  twist: 16,
  pinch: 17,
  shearX: 18,
  shearY: 19,
  color0R: 20,
  color0G: 21,
  color0B: 22,
  color0A: 23,
  color1R: 24,
  color1G: 25,
  color1B: 26,
  color1A: 27,
  sourceAX: 28,
  sourceAY: 29,
  sourceBX: 30,
  sourceBY: 31,
  sourceBWeight: 32,
  radialDecay: 33,
  waveModulation: 34,
  noiseAlgorithm: 35,
  noiseOctaves: 36,
  noiseLacunarity: 37,
  noiseGain: 38,
  noiseDomainWarp: 39,
  noiseVelocityX: 40,
  noiseVelocityY: 41,
  noiseAmplitude: 42,
  noiseBias: 43,
};

const canvas = document.getElementById("stimulus-canvas");
const toolbar = document.getElementById("toolbar");
const profileSelect = document.getElementById("profile-select");
const temporalGate = document.getElementById("temporal-gate");
const timeScale = document.getElementById("time-scale");
const toggleRun = document.getElementById("toggle-run");
const randomizeTuningButton = document.getElementById("randomize-tuning");
const resetTuningButton = document.getElementById("reset-tuning");
const savedPresetSelect = document.getElementById("saved-preset-select");
const savePresetButton = document.getElementById("save-preset");
const loadPresetButton = document.getElementById("load-preset");
const exportQuestButton = document.getElementById("export-quest");
const copyLinkButton = document.getElementById("copy-link");
const toggleChrome = document.getElementById("toggle-chrome");
const enterFullscreen = document.getElementById("enter-fullscreen");
const backendStatus = document.getElementById("backend-status");
const probeStatus = document.getElementById("probe-status");
const tuningControls = document.getElementById("tuning-controls");

let profile = null;
let adapterProfile = null;
let tuning = null;
let tuningPanel = null;
let renderer = null;
let running = true;
let frameCount = 0;
let startStampSeconds = performance.now() / 1000;
let pausedElapsedSeconds = 0;
let lastProbeSeconds = 0;
let lastCpuProbe = null;
let storedPresets = [];

bootstrap().catch((error) => {
  backendStatus.value = "failed";
  probeStatus.value = error instanceof Error ? error.message : String(error);
  console.error(error);
});

async function bootstrap() {
  const query = new URLSearchParams(window.location.search);
  const queryProfile = query.get(PROFILE_QUERY_KEY);
  if (queryProfile) {
    const option = document.createElement("option");
    option.value = queryProfile;
    option.textContent = "Query";
    option.selected = true;
    profileSelect.append(option);
  }
  if (query.get("chrome") === "0") {
    document.body.classList.add("chrome-hidden");
    toggleChrome.textContent = "Show UI";
  }
  storedPresets = loadStoredPresets();
  refreshPresetSelect();

  bindControls();
  await loadSelectedProfile();
  await createBestRenderer();
  requestAnimationFrame(renderLoop);
}

function bindControls() {
  toolbar.addEventListener("submit", (event) => event.preventDefault());
  profileSelect.addEventListener("change", () => {
    loadSelectedProfile()
      .then(createBestRenderer)
      .catch((error) => {
        backendStatus.value = "load failed";
        probeStatus.value = error instanceof Error ? error.message : String(error);
      });
  });
  toggleRun.addEventListener("click", () => {
    running = !running;
    if (running) {
      startStampSeconds = performance.now() / 1000 - pausedElapsedSeconds;
      toggleRun.textContent = "Pause";
    } else {
      pausedElapsedSeconds = currentElapsedSeconds();
      toggleRun.textContent = "Run";
    }
  });
  randomizeTuningButton.addEventListener("click", () => {
    if (!profile || !tuningPanel) {
      return;
    }
    tuningPanel.setTuning(randomizeTuning(profile));
  });
  resetTuningButton.addEventListener("click", () => {
    if (!profile || !tuningPanel) {
      return;
    }
    tuningPanel.setTuning(createDefaultTuning(profile));
  });
  savePresetButton.addEventListener("click", async () => {
    if (!profile || !tuning) {
      return;
    }
    try {
      const preset = await createBrowserPreset({
        profile,
        tuning,
        elapsedSeconds: currentElapsedSeconds(),
      });
      storedPresets = saveStoredPreset(preset);
      refreshPresetSelect(preset.preset_id);
      probeStatus.value = "preset saved";
    } catch (error) {
      console.error(error);
      probeStatus.value = "save failed";
    }
  });
  loadPresetButton.addEventListener("click", () => {
    if (!tuningPanel) {
      return;
    }
    const preset = storedPresets.find((candidate) => candidate.preset_id === savedPresetSelect.value);
    if (!preset?.tuning) {
      probeStatus.value = "no preset";
      return;
    }
    tuningPanel.setTuning(preset.tuning);
    probeStatus.value = "preset loaded";
  });
  exportQuestButton.addEventListener("click", async () => {
    if (!profile || !tuning) {
      return;
    }
    try {
      const handoff = await createQuestHandoff({
        profile,
        tuning,
        elapsedSeconds: currentElapsedSeconds(),
      });
      downloadJson(handoff, questHandoffFilename(handoff.source.profile_id));
      probeStatus.value = "quest handoff exported";
      window.__rustyOpticsStimulusLastQuestHandoff = handoff;
    } catch (error) {
      console.error(error);
      probeStatus.value = "export failed";
    }
  });
  copyLinkButton.addEventListener("click", async () => {
    if (!tuning) {
      return;
    }
    const url = new URL(window.location.href);
    url.hash = tuningToHash(tuning);
    window.history.replaceState(null, "", url.href);
    try {
      if (!navigator.clipboard?.writeText) {
        throw new Error("clipboard unavailable");
      }
      await navigator.clipboard.writeText(url.href);
      probeStatus.value = "link copied";
    } catch {
      probeStatus.value = "link in address";
    }
  });
  toggleChrome.addEventListener("click", () => {
    const hidden = document.body.classList.toggle("chrome-hidden");
    toggleChrome.textContent = hidden ? "Show UI" : "Hide UI";
  });
  enterFullscreen.addEventListener("click", async () => {
    if (!document.fullscreenElement) {
      await document.documentElement.requestFullscreen();
      document.body.classList.add("chrome-hidden");
      toggleChrome.textContent = "Show UI";
    } else {
      await document.exitFullscreen();
    }
  });
}

function refreshPresetSelect(selectedId = "") {
  savedPresetSelect.replaceChildren();
  const empty = document.createElement("option");
  empty.value = "";
  empty.textContent = "None";
  savedPresetSelect.append(empty);
  for (const preset of storedPresets) {
    const option = document.createElement("option");
    option.value = preset.preset_id;
    const labelTime = preset.saved_at ? new Date(preset.saved_at).toLocaleTimeString() : "saved";
    option.textContent = `${preset.profile_id ?? "profile"} ${labelTime}`;
    option.selected = preset.preset_id === selectedId;
    savedPresetSelect.append(option);
  }
  loadPresetButton.disabled = storedPresets.length === 0;
}

async function loadSelectedProfile() {
  const response = await fetch(profileSelect.value, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`profile ${response.status}`);
  }
  profile = await response.json();
  tuning = loadTuningFromHash(window.location.hash, profile) ?? createDefaultTuning(profile);
  tuningPanel = createStimulusTuningPanel({
    container: tuningControls,
    profile,
    initialTuning: tuning,
    onChange(nextTuning) {
      tuning = nextTuning;
      refreshAdapterProfile(currentElapsedSeconds());
    },
  });
  refreshAdapterProfile(Math.max(0, profile.temporal?.black_lead_in_seconds ?? 0));
  validatePresentation(adapterProfile.presentation);
  startStampSeconds =
    performance.now() / 1000 - Math.max(0, adapterProfile.temporal.blackLeadInSeconds + 0.075);
  frameCount = 0;
  lastProbeSeconds = 0;
  updateProbe(0);
}

async function createBestRenderer() {
  if (renderer?.destroy) {
    renderer.destroy();
  }
  renderer = null;
  if (navigator.gpu) {
    try {
      renderer = await WebGpuStimulusRenderer.create(canvas, adapterProfile);
      backendStatus.value = "webgpu compute";
      return;
    } catch (error) {
      console.warn("WebGPU stimulus renderer unavailable; using CPU fallback.", error);
    }
  }
  renderer = new CpuCanvasRenderer(canvas, adapterProfile);
  backendStatus.value = "cpu canvas";
}

function renderLoop() {
  const elapsedSeconds = currentElapsedSeconds();
  refreshAdapterProfile(elapsedSeconds);
  if (running && renderer) {
    renderer.render(elapsedSeconds, temporalGate.checked);
    frameCount += 1;
  }
  if (performance.now() / 1000 - lastProbeSeconds > 0.5) {
    updateProbe(elapsedSeconds);
    lastProbeSeconds = performance.now() / 1000;
  }
  requestAnimationFrame(renderLoop);
}

function refreshAdapterProfile(elapsedSeconds) {
  if (!profile || !tuning) {
    return;
  }
  const tunedProfile = applyTuningToProfile(profile, tuning, elapsedSeconds);
  adapterProfile = normalizeProfile(tunedProfile);
  validatePresentation(adapterProfile.presentation);
  applyCanvasTuning(canvas, tuning);
  renderer?.updateProfile?.(adapterProfile);
}

function currentElapsedSeconds() {
  if (!running) {
    return pausedElapsedSeconds;
  }
  const scale = Number.parseFloat(timeScale.value) || 1;
  return (performance.now() / 1000 - startStampSeconds) * scale;
}

function normalizeProfile(source) {
  const layerGraph = source.layer_graph ?? {};
  const temporal = source.temporal ?? {};
  const post = layerGraph.post ?? {};
  const layers = (layerGraph.layers ?? []).slice(0, MAX_LAYERS);
  return {
    profileId: source.profile_id ?? "stimulus.profile.unknown",
    presentation: source.presentation ?? {},
    temporal: {
      targetCycleHz: finiteOr(temporal.target_cycle_hz, 1),
      dutyCycle: finiteOr(temporal.duty_cycle, 0.5),
      blackLeadInSeconds: finiteOr(temporal.black_lead_in_seconds, 0),
    },
    post: {
      contrast: finiteOr(post.contrast, 1),
      brightness: finiteOr(post.brightness, 0),
      edgeFade: finiteOr(post.edge_fade, 0),
      centerFade: finiteOr(post.center_fade, 0),
      noiseAmount: finiteOr(post.noise_amount, 0),
      trailDecay: finiteOr(post.trail_decay, 0),
      geometryMix: finiteOr(post.geometry_mix, 1),
      flatColor: normalizeColor(post.flat_color, { r: 0, g: 0, b: 0, a: 1 }),
      layerBlendMode: finiteOr(post.layer_blend_mode, LAYER_BLEND.Stack),
      layerBlendAmount: finiteOr(post.layer_blend_amount, 1),
      layerBlendTarget: finiteOr(post.layer_blend_target, 0),
    },
    layers,
    layerData: packLayers(layers),
  };
}

function validatePresentation(presentation) {
  if (presentation.mode !== "StereoEyeField") {
    throw new Error(`presentation ${presentation.mode ?? "missing"}`);
  }
  if (presentation.coverage !== "FullViewport") {
    throw new Error(`coverage ${presentation.coverage ?? "missing"}`);
  }
  if (presentation.eye_count !== 2) {
    throw new Error(`eye_count ${presentation.eye_count ?? "missing"}`);
  }
}

function packLayers(layers) {
  const data = new Float32Array(MAX_LAYERS * LAYER_STRIDE);
  layers.forEach((layer, index) => {
    const base = index * LAYER_STRIDE;
    const warp = layer.warp ?? {};
    const temporal = layer.temporal ?? {};
    const interference = layer.interference ?? {};
    const noise = layer.noise ?? {};
    const colors = layer.colors ?? [];
    const c0 = colors[0]?.color ?? { r: 0, g: 0, b: 0, a: 1 };
    const c1 = colors[colors.length - 1]?.color ?? { r: 1, g: 1, b: 1, a: 1 };
    data[base + OFF.pattern] = PATTERN[layer.pattern] ?? 0;
    data[base + OFF.blend] = BLEND[layer.blend_mode] ?? 0;
    data[base + OFF.mirror] = MIRROR[layer.mirror_mode] ?? 0;
    data[base + OFF.seed] = finiteOr(layer.seed, 1);
    data[base + OFF.weight] = finiteOr(layer.weight, 1);
    data[base + OFF.opacity] = finiteOr(layer.opacity, 1);
    data[base + OFF.frequency] = finiteOr(layer.spatial_frequency, 1);
    data[base + OFF.rotation] = finiteOr(layer.rotation_radians, 0);
    data[base + OFF.phase] = finiteOr(layer.phase_offset, 0);
    data[base + OFF.temporalSpeed] = finiteOr(temporal.speed_hz, 0);
    data[base + OFF.temporalPhase] = finiteOr(temporal.phase_offset, 0);
    data[base + OFF.temporalAmplitude] = finiteOr(temporal.amplitude, 1);
    data[base + OFF.warpScaleX] = finiteOr(warp.scale?.x, 1);
    data[base + OFF.warpScaleY] = finiteOr(warp.scale?.y, 1);
    data[base + OFF.warpOffsetX] = finiteOr(warp.offset?.x, 0);
    data[base + OFF.warpOffsetY] = finiteOr(warp.offset?.y, 0);
    data[base + OFF.twist] = finiteOr(warp.twist, 0);
    data[base + OFF.pinch] = finiteOr(warp.pinch, 0);
    data[base + OFF.shearX] = finiteOr(warp.shear_x, 0);
    data[base + OFF.shearY] = finiteOr(warp.shear_y, 0);
    data[base + OFF.color0R] = finiteOr(c0.r, 0);
    data[base + OFF.color0G] = finiteOr(c0.g, 0);
    data[base + OFF.color0B] = finiteOr(c0.b, 0);
    data[base + OFF.color0A] = finiteOr(c0.a, 1);
    data[base + OFF.color1R] = finiteOr(c1.r, 1);
    data[base + OFF.color1G] = finiteOr(c1.g, 1);
    data[base + OFF.color1B] = finiteOr(c1.b, 1);
    data[base + OFF.color1A] = finiteOr(c1.a, 1);
    data[base + OFF.sourceAX] = finiteOr(interference.source_a?.x, -0.24);
    data[base + OFF.sourceAY] = finiteOr(interference.source_a?.y, 0);
    data[base + OFF.sourceBX] = finiteOr(interference.source_b?.x, 0.24);
    data[base + OFF.sourceBY] = finiteOr(interference.source_b?.y, 0);
    data[base + OFF.sourceBWeight] = finiteOr(interference.source_b_weight, 1);
    data[base + OFF.radialDecay] = finiteOr(interference.radial_decay, 0);
    data[base + OFF.waveModulation] = finiteOr(interference.wave_modulation, 0);
    data[base + OFF.noiseAlgorithm] = NOISE[noise.algorithm] ?? 2;
    data[base + OFF.noiseOctaves] = finiteOr(noise.octaves, 1);
    data[base + OFF.noiseLacunarity] = finiteOr(noise.lacunarity, 2);
    data[base + OFF.noiseGain] = finiteOr(noise.gain, 0.5);
    data[base + OFF.noiseDomainWarp] = finiteOr(noise.domain_warp_strength, 0);
    data[base + OFF.noiseVelocityX] = finiteOr(noise.animation_velocity?.x, 0);
    data[base + OFF.noiseVelocityY] = finiteOr(noise.animation_velocity?.y, 0);
    data[base + OFF.noiseAmplitude] = finiteOr(noise.amplitude, 1);
    data[base + OFF.noiseBias] = finiteOr(noise.bias, 0);
  });
  return data;
}

function finiteOr(value, fallback) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}

function normalizeColor(value, fallback) {
  return {
    r: finiteOr(value?.r, fallback.r),
    g: finiteOr(value?.g, fallback.g),
    b: finiteOr(value?.b, fallback.b),
    a: finiteOr(value?.a, fallback.a),
  };
}

class WebGpuStimulusRenderer {
  static async create(targetCanvas, currentProfile) {
    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
      throw new Error("webgpu adapter");
    }
    const device = await adapter.requestDevice();
    const computeModule = device.createShaderModule({ code: COMPUTE_WGSL });
    const renderModule = device.createShaderModule({ code: RENDER_WGSL });
    await assertShaderModule("compute", computeModule);
    await assertShaderModule("render", renderModule);
    const context = targetCanvas.getContext("webgpu");
    if (!context) {
      throw new Error("webgpu context");
    }
    return new WebGpuStimulusRenderer(targetCanvas, currentProfile, device, context, computeModule, renderModule);
  }

  constructor(targetCanvas, currentProfile, device, context, computeModule, renderModule) {
    this.canvas = targetCanvas;
    this.profile = currentProfile;
    this.device = device;
    this.context = context;
    this.canvasFormat = navigator.gpu.getPreferredCanvasFormat();
    this.globalBuffer = device.createBuffer({
      size: 80,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    this.layerBuffer = device.createBuffer({
      size: MAX_LAYERS * LAYER_STRIDE * 4,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(this.layerBuffer, 0, currentProfile.layerData);
    this.sampler = device.createSampler({
      magFilter: "linear",
      minFilter: "linear",
    });
    this.computePipeline = device.createComputePipeline({
      layout: "auto",
      compute: {
        module: computeModule,
        entryPoint: "main",
      },
    });
    this.renderPipeline = device.createRenderPipeline({
      layout: "auto",
      vertex: {
        module: renderModule,
        entryPoint: "vs",
      },
      fragment: {
        module: renderModule,
        entryPoint: "fs",
        targets: [{ format: this.canvasFormat }],
      },
      primitive: { topology: "triangle-list" },
    });
    this.texture = null;
    this.computeBindGroup = null;
    this.renderBindGroup = null;
    this.width = 0;
    this.height = 0;
    this.readbackPending = false;
    this.lastReadbackFrame = 0;
  }

  destroy() {
    this.texture?.destroy();
    this.globalBuffer?.destroy();
    this.layerBuffer?.destroy();
  }

  updateProfile(currentProfile) {
    this.profile = currentProfile;
    this.device.queue.writeBuffer(this.layerBuffer, 0, currentProfile.layerData);
  }

  render(elapsedSeconds, useTemporalGate) {
    this.resize();
    this.writeGlobals(elapsedSeconds, useTemporalGate);

    const encoder = this.device.createCommandEncoder();
    const computePass = encoder.beginComputePass();
    computePass.setPipeline(this.computePipeline);
    computePass.setBindGroup(0, this.computeBindGroup);
    computePass.dispatchWorkgroups(Math.ceil(this.width / 8), Math.ceil(this.height / 8));
    computePass.end();

    const view = this.context.getCurrentTexture().createView();
    const renderPass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view,
          clearValue: { r: 0, g: 0, b: 0, a: 1 },
          loadOp: "clear",
          storeOp: "store",
        },
      ],
    });
    renderPass.setPipeline(this.renderPipeline);
    renderPass.setBindGroup(0, this.renderBindGroup);
    renderPass.draw(3);
    renderPass.end();

    let readback = null;
    if (!this.readbackPending && frameCount - this.lastReadbackFrame > TARGET_FPS) {
      readback = this.encodeReadback(encoder);
      this.lastReadbackFrame = frameCount;
    }

    this.device.queue.submit([encoder.finish()]);
    if (readback) {
      this.resolveReadback(readback);
    }
  }

  resize() {
    const pixelRatio = Math.min(window.devicePixelRatio || 1, 2);
    const nextWidth = Math.max(1, Math.min(2048, Math.floor(this.canvas.clientWidth * pixelRatio)));
    const nextHeight = Math.max(1, Math.min(2048, Math.floor(this.canvas.clientHeight * pixelRatio)));
    if (nextWidth === this.width && nextHeight === this.height) {
      return;
    }
    this.width = nextWidth;
    this.height = nextHeight;
    this.canvas.width = nextWidth;
    this.canvas.height = nextHeight;
    this.context.configure({
      device: this.device,
      format: this.canvasFormat,
      alphaMode: "opaque",
    });
    this.texture?.destroy();
    this.texture = this.device.createTexture({
      size: [this.width, this.height],
      format: "rgba8unorm",
      usage:
        GPUTextureUsage.STORAGE_BINDING |
        GPUTextureUsage.TEXTURE_BINDING |
        GPUTextureUsage.COPY_SRC,
    });
    this.computeBindGroup = this.device.createBindGroup({
      layout: this.computePipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: this.texture.createView() },
        { binding: 1, resource: { buffer: this.globalBuffer } },
        { binding: 2, resource: { buffer: this.layerBuffer } },
      ],
    });
    this.renderBindGroup = this.device.createBindGroup({
      layout: this.renderPipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: this.sampler },
        { binding: 1, resource: this.texture.createView() },
      ],
    });
  }

  writeGlobals(elapsedSeconds, useTemporalGate) {
    const temporal = this.profile.temporal;
    const post = this.profile.post;
    const values = new Float32Array(20);
    values[0] = this.width;
    values[1] = this.height;
    values[2] = elapsedSeconds;
    values[3] = this.profile.layers.length;
    values[4] = temporal.targetCycleHz;
    values[5] = temporal.dutyCycle;
    values[6] = temporal.blackLeadInSeconds;
    values[7] = useTemporalGate ? 1 : 0;
    values[8] = post.contrast;
    values[9] = post.brightness;
    values[10] = post.edgeFade;
    values[11] = post.centerFade;
    values[12] = post.flatColor.r;
    values[13] = post.flatColor.g;
    values[14] = post.flatColor.b;
    values[15] = post.geometryMix;
    values[16] = post.layerBlendMode;
    values[17] = post.layerBlendAmount;
    values[18] = post.layerBlendTarget;
    values[19] = 0;
    this.device.queue.writeBuffer(this.globalBuffer, 0, values);
  }

  encodeReadback(encoder) {
    const sampleWidth = Math.min(this.width, 128);
    const sampleHeight = Math.min(this.height, 128);
    const originX = Math.floor((this.width - sampleWidth) * 0.5);
    const originY = Math.floor((this.height - sampleHeight) * 0.5);
    const bytesPerRow = Math.ceil((sampleWidth * 4) / 256) * 256;
    const size = bytesPerRow * sampleHeight;
    const buffer = this.device.createBuffer({
      size,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });
    encoder.copyTextureToBuffer(
      { texture: this.texture, origin: { x: originX, y: originY } },
      { buffer, bytesPerRow, rowsPerImage: sampleHeight },
      { width: sampleWidth, height: sampleHeight, depthOrArrayLayers: 1 },
    );
    this.readbackPending = true;
    return { buffer, bytesPerRow, sampleWidth, sampleHeight };
  }

  resolveReadback(readback) {
    const { buffer, bytesPerRow, sampleWidth, sampleHeight } = readback;
    buffer.mapAsync(GPUMapMode.READ).then(() => {
      const data = new Uint8Array(buffer.getMappedRange());
      let min = 255;
      let max = 0;
      let sum = 0;
      let count = 0;
      for (let y = 0; y < sampleHeight; y += 8) {
        for (let x = 0; x < sampleWidth; x += 8) {
          const value = data[y * bytesPerRow + x * 4];
          min = Math.min(min, value);
          max = Math.max(max, value);
          sum += value;
          count += 1;
        }
      }
      window.__rustyOpticsStimulus.gpuProbe = {
        min,
        max,
        mean: count > 0 ? sum / count : 0,
        width: sampleWidth,
        height: sampleHeight,
      };
      buffer.unmap();
      buffer.destroy();
      this.readbackPending = false;
    }).catch(() => {
      buffer.destroy();
      this.readbackPending = false;
    });
  }
}

async function assertShaderModule(label, module) {
  if (!module.getCompilationInfo) {
    return;
  }
  const info = await module.getCompilationInfo();
  const errors = info.messages.filter((message) => message.type === "error");
  if (errors.length > 0) {
    const first = errors[0];
    throw new Error(`${label} shader ${first.lineNum}:${first.linePos} ${first.message}`);
  }
}

class CpuCanvasRenderer {
  constructor(targetCanvas, currentProfile) {
    this.canvas = targetCanvas;
    this.profile = currentProfile;
    this.context = targetCanvas.getContext("2d", { alpha: false });
    this.offscreen = document.createElement("canvas");
    this.offscreen.width = 320;
    this.offscreen.height = 200;
    this.offscreenContext = this.offscreen.getContext("2d", { alpha: false });
  }

  updateProfile(currentProfile) {
    this.profile = currentProfile;
  }

  render(elapsedSeconds, useTemporalGate) {
    const pixelRatio = Math.min(window.devicePixelRatio || 1, 2);
    const width = Math.max(1, Math.floor(this.canvas.clientWidth * pixelRatio));
    const height = Math.max(1, Math.floor(this.canvas.clientHeight * pixelRatio));
    if (this.canvas.width !== width || this.canvas.height !== height) {
      this.canvas.width = width;
      this.canvas.height = height;
    }
    const image = this.offscreenContext.createImageData(this.offscreen.width, this.offscreen.height);
    let ptr = 0;
    for (let y = 0; y < this.offscreen.height; y += 1) {
      for (let x = 0; x < this.offscreen.width; x += 1) {
        const color = sampleProfile(this.profile, (x + 0.5) / this.offscreen.width, (y + 0.5) / this.offscreen.height, elapsedSeconds, useTemporalGate);
        image.data[ptr++] = clampByte(color[0] * 255);
        image.data[ptr++] = clampByte(color[1] * 255);
        image.data[ptr++] = clampByte(color[2] * 255);
        image.data[ptr++] = 255;
      }
    }
    this.offscreenContext.putImageData(image, 0, 0);
    this.context.imageSmoothingEnabled = true;
    this.context.drawImage(this.offscreen, 0, 0, width, height);
  }
}

function updateProbe(elapsedSeconds) {
  if (!adapterProfile) {
    return;
  }
  const probe = cpuProbe(adapterProfile, elapsedSeconds, temporalGate.checked);
  lastCpuProbe = probe;
  const gpuProbe = window.__rustyOpticsStimulus?.gpuProbe;
  const gpuText = gpuProbe ? ` gpu ${gpuProbe.min}-${gpuProbe.max}` : "";
  probeStatus.value = `cpu ${probe.min.toFixed(3)}-${probe.max.toFixed(3)}${gpuText}`;
  window.__rustyOpticsStimulus = {
    backend: backendStatus.value,
    profileId: adapterProfile.profileId,
    presentation: adapterProfile.presentation,
    cpuProbe: probe,
    gpuProbe,
    post: adapterProfile.post,
    tuning,
    presetCount: storedPresets.length,
    ready: true,
    frameCount,
  };
}

function cpuProbe(currentProfile, elapsedSeconds, useTemporalGate) {
  let min = 1;
  let max = 0;
  let sum = 0;
  let hash = 2166136261;
  let count = 0;
  for (let y = 0; y < 8; y += 1) {
    for (let x = 0; x < 8; x += 1) {
      const color = sampleProfile(currentProfile, (x + 0.5) / 8, (y + 0.5) / 8, elapsedSeconds, useTemporalGate);
      const value = (color[0] + color[1] + color[2]) / 3;
      min = Math.min(min, value);
      max = Math.max(max, value);
      sum += value;
      hash ^= Math.floor(value * 65535);
      hash = Math.imul(hash, 16777619) >>> 0;
      count += 1;
    }
  }
  return {
    min,
    max,
    mean: sum / count,
    hash: hash.toString(16).padStart(8, "0"),
  };
}

function sampleProfile(currentProfile, u, v, elapsedSeconds, useTemporalGate) {
  if (useTemporalGate && !sampleTemporal(currentProfile.temporal, elapsedSeconds)) {
    return [0, 0, 0];
  }
  const post = currentProfile.post;
  let color = sampleGeometryLayers(currentProfile, u, v, elapsedSeconds);
  color = color.map((value) => (value - 0.5) * post.contrast + 0.5 + post.brightness);
  const edge = Math.min(u, 1 - u, v, 1 - v);
  if (post.edgeFade > 0) {
    const edgeMask = smoothstep(0, post.edgeFade, edge);
    color = color.map((value) => value * edgeMask);
  }
  if (post.centerFade > 0) {
    const distance = Math.hypot(u - 0.5, v - 0.5);
    const centerMask = mix(1, smoothstep(0, 0.5, distance), post.centerFade);
    color = color.map((value) => value * centerMask);
  }
  color = color.map((value, index) => {
    const flat = index === 0 ? post.flatColor.r : index === 1 ? post.flatColor.g : post.flatColor.b;
    return mix(flat, value, post.geometryMix);
  });
  return color.map(clamp01);
}

function sampleGeometryLayers(currentProfile, u, v, elapsedSeconds) {
  const post = currentProfile.post;
  const mode = Math.floor(finiteOr(post.layerBlendMode, LAYER_BLEND.Stack));
  const amount = clamp01(finiteOr(post.layerBlendAmount, 1));
  if (mode === LAYER_BLEND.Mean) {
    return mixColorArray(
      compositeStackLayers(currentProfile.layers, u, v, elapsedSeconds),
      compositeMeanLayers(currentProfile.layers, u, v, elapsedSeconds),
      amount,
    );
  }
  if (mode === LAYER_BLEND.Cross) {
    return compositeCrossLayers(
      currentProfile.layers,
      u,
      v,
      elapsedSeconds,
      amount,
      finiteOr(post.layerBlendTarget, 0),
    );
  }
  return compositeStackLayers(currentProfile.layers, u, v, elapsedSeconds);
}

function compositeStackLayers(layers, u, v, elapsedSeconds) {
  let color = [0, 0, 0];
  for (const layer of layers) {
    const layerSample = sampleLayerColor(layer, u, v, elapsedSeconds);
    if (layer.blend_mode === "Multiply") {
      color = color.map((value, index) => value * mix(1, layerSample.color[index], layerSample.weight));
    } else if (layer.blend_mode === "Max") {
      color = color.map((value, index) => Math.max(value, layerSample.color[index] * layerSample.weight));
    } else {
      color = color.map((value, index) => value + layerSample.color[index] * layerSample.weight);
    }
  }
  return color;
}

function compositeMeanLayers(layers, u, v, elapsedSeconds) {
  let color = [0, 0, 0];
  let totalWeight = 0;
  for (const layer of layers) {
    const layerSample = sampleLayerColor(layer, u, v, elapsedSeconds);
    const weight = Math.max(0, layerSample.weight);
    color = color.map((value, index) => value + layerSample.color[index] * weight);
    totalWeight += weight;
  }
  if (totalWeight <= 0.0001) {
    return [0, 0, 0];
  }
  return color.map((value) => value / totalWeight);
}

function compositeCrossLayers(layers, u, v, elapsedSeconds, amount, targetIndex) {
  if (layers.length === 0) {
    return [0, 0, 0];
  }
  const target = Math.max(0, Math.min(layers.length - 1, Math.floor(finiteOr(targetIndex, 0))));
  return mixColorArray(
    weightedLayerColor(layers[0], u, v, elapsedSeconds),
    weightedLayerColor(layers[target], u, v, elapsedSeconds),
    amount,
  );
}

function weightedLayerColor(layer, u, v, elapsedSeconds) {
  const layerSample = sampleLayerColor(layer, u, v, elapsedSeconds);
  const weight = Math.max(0, layerSample.weight);
  return layerSample.color.map((value) => value * weight);
}

function sampleLayerColor(layer, u, v, elapsedSeconds) {
  const sample = sampleLayer(layer, u, v, elapsedSeconds);
  const colors = layer.colors ?? [];
  const c0 = colors[0]?.color ?? { r: 0, g: 0, b: 0 };
  const c1 = colors[colors.length - 1]?.color ?? { r: 1, g: 1, b: 1 };
  return {
    color: [
      mix(c0.r, c1.r, sample),
      mix(c0.g, c1.g, sample),
      mix(c0.b, c1.b, sample),
    ],
    weight: finiteOr(layer.weight, 1) * finiteOr(layer.opacity, 1),
  };
}

function mixColorArray(a, b, amount) {
  return a.map((value, index) => mix(value, b[index], amount));
}

function sampleTemporal(temporal, elapsedSeconds) {
  if (elapsedSeconds < temporal.blackLeadInSeconds) {
    return false;
  }
  const active = elapsedSeconds - temporal.blackLeadInSeconds;
  return fract(active * temporal.targetCycleHz) < temporal.dutyCycle;
}

function sampleLayer(layer, u, v, elapsedSeconds) {
  const frequency = finiteOr(layer.spatial_frequency, 1);
  const phase = finiteOr(layer.phase_offset, 0) + elapsedSeconds * finiteOr(layer.temporal?.speed_hz, 0);
  const p = transformPoint(layer, u, v);
  switch (layer.pattern) {
    case "Rings":
      return wave(Math.hypot(p[0], p[1]) * frequency - phase);
    case "Rays":
      return wave((Math.atan2(p[1], p[0]) / TAU) * frequency + phase);
    case "Checkerboard":
      return (Math.floor((p[0] + 0.5) * frequency) + Math.floor((p[1] + 0.5) * frequency)) % 2 === 0 ? 1 : 0;
    case "NoiseField":
    case "PerlinNoise":
      return sampleNoise(layer, p[0], p[1], elapsedSeconds);
    case "Ripple":
      return sampleRipple(layer, p, frequency, phase);
    case "Interference":
      return sampleInterference(layer, p, frequency, phase);
    case "Stripes":
    default:
      return wave(p[0] * frequency + phase);
  }
}

const TAU = Math.PI * 2;

function transformPoint(layer, u, v) {
  let x = u - 0.5;
  let y = v - 0.5;
  const mirror = layer.mirror_mode ?? "None";
  if (mirror === "Horizontal" || mirror === "Axes") {
    x = Math.abs(x);
  }
  if (mirror === "Vertical" || mirror === "Axes") {
    y = Math.abs(y);
  }
  if (mirror === "Kaleidoscope3" || mirror === "Kaleidoscope6") {
    const sectors = mirror === "Kaleidoscope3" ? 3 : 6;
    const radius = Math.hypot(x, y);
    let angle = Math.atan2(y, x);
    const sector = TAU / sectors;
    angle = Math.abs(((angle + Math.PI) % sector) - sector * 0.5);
    x = Math.cos(angle) * radius;
    y = Math.sin(angle) * radius;
  }
  const warp = layer.warp ?? {};
  x = x * finiteOr(warp.scale?.x, 1) + finiteOr(warp.offset?.x, 0);
  y = y * finiteOr(warp.scale?.y, 1) + finiteOr(warp.offset?.y, 0);
  const shearX = finiteOr(warp.shear_x, 0);
  const shearY = finiteOr(warp.shear_y, 0);
  x += y * shearX;
  y += x * shearY;
  const radius = Math.hypot(x, y);
  const twist = finiteOr(warp.twist, 0) * radius;
  const rotation = finiteOr(layer.rotation_radians, 0) + twist;
  const cosR = Math.cos(rotation);
  const sinR = Math.sin(rotation);
  const pinch = 1 + finiteOr(warp.pinch, 0) * radius;
  return [(x * cosR - y * sinR) * pinch, (x * sinR + y * cosR) * pinch];
}

function sampleRipple(layer, p, frequency, phase) {
  const source = layer.interference?.source_a ?? { x: -0.24, y: 0 };
  const distance = Math.hypot(p[0] - source.x, p[1] - source.y);
  const decay = Math.exp(-distance * finiteOr(layer.interference?.radial_decay, 0));
  return 0.5 + 0.5 * Math.sin((distance * frequency - phase) * TAU) * decay;
}

function sampleInterference(layer, p, frequency, phase) {
  const controls = layer.interference ?? {};
  const a = controls.source_a ?? { x: -0.24, y: 0 };
  const b = controls.source_b ?? { x: 0.24, y: 0 };
  const da = Math.hypot(p[0] - a.x, p[1] - a.y);
  const db = Math.hypot(p[0] - b.x, p[1] - b.y);
  const waveA = Math.sin((da * frequency - phase) * TAU);
  const waveB = Math.sin((db * frequency + phase) * TAU) * finiteOr(controls.source_b_weight, 1);
  const modulation = Math.sin((p[0] + p[1] + phase) * TAU) * finiteOr(controls.wave_modulation, 0);
  const decay = Math.exp(-(da + db) * 0.5 * finiteOr(controls.radial_decay, 0));
  return clamp01(0.5 + 0.25 * (waveA + waveB + modulation) * decay);
}

function sampleNoise(layer, x, y, elapsedSeconds) {
  const noise = layer.noise ?? {};
  const frequency = finiteOr(layer.spatial_frequency, 1);
  const seed = finiteOr(layer.seed, 1);
  let px = x * frequency + finiteOr(noise.animation_velocity?.x, 0) * elapsedSeconds;
  let py = y * frequency + finiteOr(noise.animation_velocity?.y, 0) * elapsedSeconds;
  const warp = finiteOr(noise.domain_warp_strength, 0);
  if (warp > 0) {
    px += (perlin(px + 17.1, py - 3.7, seed + 19) - 0.5) * warp;
    py += (perlin(px - 5.4, py + 11.2, seed + 23) - 0.5) * warp;
  }
  const octaves = Math.max(1, Math.min(8, Math.floor(finiteOr(noise.octaves, 1))));
  const lacunarity = finiteOr(noise.lacunarity, 2);
  const gain = finiteOr(noise.gain, 0.5);
  let amplitude = 1;
  let total = 0;
  let norm = 0;
  for (let i = 0; i < octaves; i += 1) {
    const scale = Math.pow(lacunarity, i);
    const octaveSeed = seed + i * 101;
    const value = noise.algorithm === "CellValue"
      ? hash2(Math.floor(px * scale), Math.floor(py * scale), octaveSeed)
      : noise.algorithm === "SmoothValue"
        ? smoothValueNoise(px * scale, py * scale, octaveSeed)
        : perlin(px * scale, py * scale, octaveSeed);
    total += value * amplitude;
    norm += amplitude;
    amplitude *= gain;
  }
  return clamp01((total / Math.max(norm, 0.0001)) * finiteOr(noise.amplitude, 1) + finiteOr(noise.bias, 0));
}

function smoothValueNoise(x, y, seed) {
  const ix = Math.floor(x);
  const iy = Math.floor(y);
  const fx = fract(x);
  const fy = fract(y);
  const ux = fade(fx);
  const uy = fade(fy);
  const a = hash2(ix, iy, seed);
  const b = hash2(ix + 1, iy, seed);
  const c = hash2(ix, iy + 1, seed);
  const d = hash2(ix + 1, iy + 1, seed);
  return mix(mix(a, b, ux), mix(c, d, ux), uy);
}

function perlin(x, y, seed) {
  const ix = Math.floor(x);
  const iy = Math.floor(y);
  const fx = fract(x);
  const fy = fract(y);
  const ux = fade(fx);
  const uy = fade(fy);
  const a = gradientDot(ix, iy, fx, fy, seed);
  const b = gradientDot(ix + 1, iy, fx - 1, fy, seed);
  const c = gradientDot(ix, iy + 1, fx, fy - 1, seed);
  const d = gradientDot(ix + 1, iy + 1, fx - 1, fy - 1, seed);
  return clamp01(0.5 + 0.5 * mix(mix(a, b, ux), mix(c, d, ux), uy));
}

function gradientDot(ix, iy, x, y, seed) {
  const angle = hash2(ix, iy, seed) * TAU;
  return Math.cos(angle) * x + Math.sin(angle) * y;
}

function hash2(x, y, seed) {
  return fract(Math.sin(x * 127.1 + y * 311.7 + seed * 74.7) * 43758.5453123);
}

function wave(value) {
  return 0.5 + 0.5 * Math.sin(value * TAU);
}

function fade(value) {
  return value * value * value * (value * (value * 6 - 15) + 10);
}

function smoothstep(edge0, edge1, value) {
  const t = clamp01((value - edge0) / Math.max(edge1 - edge0, 0.00001));
  return t * t * (3 - 2 * t);
}

function mix(a, b, t) {
  return a * (1 - t) + b * t;
}

function fract(value) {
  return value - Math.floor(value);
}

function clamp01(value) {
  return Math.min(1, Math.max(0, value));
}

function clampByte(value) {
  return Math.min(255, Math.max(0, Math.round(value)));
}

const COMPUTE_WGSL = `
struct Globals {
  data0: vec4f,
  data1: vec4f,
  data2: vec4f,
  data3: vec4f,
  data4: vec4f,
};

struct LayerData {
  values: array<f32>,
};

@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> globals: Globals;
@group(0) @binding(2) var<storage, read> layers: LayerData;

const TAU: f32 = 6.28318530718;
const STRIDE: u32 = 48u;

fn lf(layer: u32, offset: u32) -> f32 {
  return layers.values[layer * STRIDE + offset];
}

fn clamp01(value: f32) -> f32 {
  return clamp(value, 0.0, 1.0);
}

fn wave(value: f32) -> f32 {
  return 0.5 + 0.5 * sin(value * TAU);
}

fn hash2(p: vec2f, seed: f32) -> f32 {
  return fract(sin(dot(p, vec2f(127.1, 311.7)) + seed * 74.7) * 43758.5453123);
}

fn fade(value: f32) -> f32 {
  return value * value * value * (value * (value * 6.0 - 15.0) + 10.0);
}

fn gradient_dot(ip: vec2f, fp: vec2f, seed: f32) -> f32 {
  let angle = hash2(ip, seed) * TAU;
  return dot(vec2f(cos(angle), sin(angle)), fp);
}

fn perlin(p: vec2f, seed: f32) -> f32 {
  let ip = floor(p);
  let fp = fract(p);
  let u = vec2f(fade(fp.x), fade(fp.y));
  let a = gradient_dot(ip, fp, seed);
  let b = gradient_dot(ip + vec2f(1.0, 0.0), fp - vec2f(1.0, 0.0), seed);
  let c = gradient_dot(ip + vec2f(0.0, 1.0), fp - vec2f(0.0, 1.0), seed);
  let d = gradient_dot(ip + vec2f(1.0, 1.0), fp - vec2f(1.0, 1.0), seed);
  return clamp01(0.5 + 0.5 * mix(mix(a, b, u.x), mix(c, d, u.x), u.y));
}

fn smooth_value_noise(p: vec2f, seed: f32) -> f32 {
  let ip = floor(p);
  let fp = fract(p);
  let u = vec2f(fade(fp.x), fade(fp.y));
  let a = hash2(ip, seed);
  let b = hash2(ip + vec2f(1.0, 0.0), seed);
  let c = hash2(ip + vec2f(0.0, 1.0), seed);
  let d = hash2(ip + vec2f(1.0, 1.0), seed);
  return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn sample_noise(layer: u32, p0: vec2f, elapsed: f32) -> f32 {
  let frequency = lf(layer, 6u);
  let seed = lf(layer, 3u);
  var p = p0 * frequency + vec2f(lf(layer, 40u), lf(layer, 41u)) * elapsed;
  let warp = lf(layer, 39u);
  if (warp > 0.0) {
    p.x = p.x + (perlin(p + vec2f(17.1, -3.7), seed + 19.0) - 0.5) * warp;
    p.y = p.y + (perlin(p + vec2f(-5.4, 11.2), seed + 23.0) - 0.5) * warp;
  }
  let octaves = min(max(u32(lf(layer, 36u)), 1u), 8u);
  let lacunarity = lf(layer, 37u);
  let gain = lf(layer, 38u);
  let algorithm = lf(layer, 35u);
  var amplitude = 1.0;
  var total = 0.0;
  var norm = 0.0;
  var i = 0u;
  loop {
    if (i >= octaves) { break; }
    let scale = pow(lacunarity, f32(i));
    let octave_seed = seed + f32(i) * 101.0;
    var value = perlin(p * scale, octave_seed);
    if (algorithm < 0.5) {
      value = hash2(floor(p * scale), octave_seed);
    } else if (algorithm < 1.5) {
      value = smooth_value_noise(p * scale, octave_seed);
    }
    total = total + value * amplitude;
    norm = norm + amplitude;
    amplitude = amplitude * gain;
    i = i + 1u;
  }
  return clamp01((total / max(norm, 0.0001)) * lf(layer, 42u) + lf(layer, 43u));
}

fn transform_point(layer: u32, uv: vec2f) -> vec2f {
  var p = uv - vec2f(0.5, 0.5);
  let mirror = lf(layer, 2u);
  if ((mirror > 0.5 && mirror < 1.5) || (mirror > 2.5 && mirror < 3.5)) {
    p.x = abs(p.x);
  }
  if (mirror > 1.5 && mirror < 3.5) {
    p.y = abs(p.y);
  }
  if (mirror > 3.5) {
    let sectors = select(3.0, 6.0, mirror > 4.5);
    let radius0 = length(p);
    let sector = TAU / sectors;
    let angle = abs(fract((atan2(p.y, p.x) + 3.14159265) / sector) * sector - sector * 0.5);
    p = vec2f(cos(angle), sin(angle)) * radius0;
  }
  p = p * vec2f(lf(layer, 12u), lf(layer, 13u)) + vec2f(lf(layer, 14u), lf(layer, 15u));
  p.x = p.x + p.y * lf(layer, 18u);
  p.y = p.y + p.x * lf(layer, 19u);
  let radius = length(p);
  let rotation = lf(layer, 7u) + lf(layer, 16u) * radius;
  let c = cos(rotation);
  let s = sin(rotation);
  let pinch = 1.0 + lf(layer, 17u) * radius;
  return vec2f(p.x * c - p.y * s, p.x * s + p.y * c) * pinch;
}

fn sample_layer(layer: u32, uv: vec2f, elapsed: f32) -> f32 {
  let pattern = lf(layer, 0u);
  let frequency = lf(layer, 6u);
  let phase = lf(layer, 8u) + elapsed * lf(layer, 9u);
  let p = transform_point(layer, uv);
  if (pattern < 0.5) {
    return wave(p.x * frequency + phase);
  }
  if (pattern < 1.5) {
    return wave(length(p) * frequency - phase);
  }
  if (pattern < 2.5) {
    return wave((atan2(p.y, p.x) / TAU) * frequency + phase);
  }
  if (pattern < 3.5) {
    let cell = floor((p + vec2f(0.5, 0.5)) * frequency);
    return select(0.0, 1.0, fract((cell.x + cell.y) * 0.5) < 0.25);
  }
  if (pattern < 5.5) {
    return sample_noise(layer, p, elapsed);
  }
  if (pattern < 6.5) {
    let source = vec2f(lf(layer, 28u), lf(layer, 29u));
    let distance = length(p - source);
    let decay = exp(-distance * lf(layer, 33u));
    return 0.5 + 0.5 * sin((distance * frequency - phase) * TAU) * decay;
  }
  let source_a = vec2f(lf(layer, 28u), lf(layer, 29u));
  let source_b = vec2f(lf(layer, 30u), lf(layer, 31u));
  let da = length(p - source_a);
  let db = length(p - source_b);
  let wave_a = sin((da * frequency - phase) * TAU);
  let wave_b = sin((db * frequency + phase) * TAU) * lf(layer, 32u);
  let modulation = sin((p.x + p.y + phase) * TAU) * lf(layer, 34u);
  let decay = exp(-(da + db) * 0.5 * lf(layer, 33u));
  return clamp01(0.5 + 0.25 * (wave_a + wave_b + modulation) * decay);
}

fn layer_weight(layer: u32) -> f32 {
  return max(0.0, lf(layer, 4u) * lf(layer, 5u));
}

fn layer_color(layer: u32, uv: vec2f, elapsed: f32) -> vec3f {
  let value = sample_layer(layer, uv, elapsed);
  let c0 = vec3f(lf(layer, 20u), lf(layer, 21u), lf(layer, 22u));
  let c1 = vec3f(lf(layer, 24u), lf(layer, 25u), lf(layer, 26u));
  return mix(c0, c1, vec3f(value));
}

fn weighted_layer_color(layer: u32, uv: vec2f, elapsed: f32) -> vec3f {
  return layer_color(layer, uv, elapsed) * layer_weight(layer);
}

fn composite_stack_layers(layer_count: u32, uv: vec2f, elapsed: f32) -> vec3f {
  var color = vec3f(0.0, 0.0, 0.0);
  var layer = 0u;
  loop {
    if (layer >= layer_count) { break; }
    let current = layer_color(layer, uv, elapsed);
    let weight = layer_weight(layer);
    let blend = lf(layer, 1u);
    if (blend > 0.5 && blend < 1.5) {
      color = color * mix(vec3f(1.0), current, vec3f(weight));
    } else if (blend > 1.5) {
      color = max(color, current * weight);
    } else {
      color = color + current * weight;
    }
    layer = layer + 1u;
  }
  return color;
}

fn composite_mean_layers(layer_count: u32, uv: vec2f, elapsed: f32) -> vec3f {
  var color = vec3f(0.0, 0.0, 0.0);
  var total_weight = 0.0;
  var layer = 0u;
  loop {
    if (layer >= layer_count) { break; }
    let weight = layer_weight(layer);
    color = color + layer_color(layer, uv, elapsed) * weight;
    total_weight = total_weight + weight;
    layer = layer + 1u;
  }
  if (total_weight <= 0.0001) {
    return vec3f(0.0, 0.0, 0.0);
  }
  return color / total_weight;
}

fn composite_cross_layers(layer_count: u32, uv: vec2f, elapsed: f32, amount: f32, target_index: f32) -> vec3f {
  if (layer_count == 0u) {
    return vec3f(0.0, 0.0, 0.0);
  }
  let target_layer = min(u32(max(target_index, 0.0)), layer_count - 1u);
  return mix(weighted_layer_color(0u, uv, elapsed), weighted_layer_color(target_layer, uv, elapsed), vec3f(amount));
}

fn temporal_gate(elapsed: f32) -> f32 {
  if (globals.data1.w < 0.5) {
    return 1.0;
  }
  if (elapsed < globals.data1.z) {
    return 0.0;
  }
  let active_time = elapsed - globals.data1.z;
  let phase = fract(active_time * globals.data1.x);
  return select(0.0, 1.0, phase < globals.data1.y);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
  let width = u32(globals.data0.x);
  let height = u32(globals.data0.y);
  if (gid.x >= width || gid.y >= height) {
    return;
  }
  let uv = (vec2f(f32(gid.x), f32(gid.y)) + vec2f(0.5, 0.5)) / vec2f(globals.data0.xy);
  let elapsed = globals.data0.z;
  let layer_count = min(u32(globals.data0.w), 16u);
  let blend_mode = globals.data4.x;
  let blend_amount = clamp(globals.data4.y, 0.0, 1.0);
  var color = composite_stack_layers(layer_count, uv, elapsed);
  if (blend_mode > 1.5) {
    color = composite_cross_layers(layer_count, uv, elapsed, blend_amount, globals.data4.z);
  } else if (blend_mode > 0.5) {
    color = mix(color, composite_mean_layers(layer_count, uv, elapsed), vec3f(blend_amount));
  }
  color = (color - vec3f(0.5)) * globals.data2.x + vec3f(0.5 + globals.data2.y);
  let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
  if (globals.data2.z > 0.0) {
    color = color * smoothstep(0.0, globals.data2.z, edge);
  }
  if (globals.data2.w > 0.0) {
    let center_distance = distance(uv, vec2f(0.5, 0.5));
    let center_mask = mix(1.0, smoothstep(0.0, 0.5, center_distance), globals.data2.w);
    color = color * center_mask;
  }
  color = mix(globals.data3.xyz, color, vec3f(clamp(globals.data3.w, 0.0, 1.0)));
  if (globals.data2.x >= 0.0) {
    color = color * temporal_gate(elapsed);
  }
  textureStore(output_texture, vec2i(gid.xy), vec4f(clamp(color, vec3f(0.0), vec3f(1.0)), 1.0));
}
`;

const RENDER_WGSL = `
@group(0) @binding(0) var field_sampler: sampler;
@group(0) @binding(1) var field_texture: texture_2d<f32>;

struct VertexOut {
  @builtin(position) position: vec4f,
  @location(0) uv: vec2f,
};

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
  var positions = array<vec2f, 3>(
    vec2f(-1.0, -1.0),
    vec2f(3.0, -1.0),
    vec2f(-1.0, 3.0)
  );
  let position = positions[vertex_index];
  var out: VertexOut;
  out.position = vec4f(position, 0.0, 1.0);
  out.uv = position * 0.5 + vec2f(0.5, 0.5);
  return out;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4f {
  return textureSample(field_texture, field_sampler, in.uv);
}
`;
