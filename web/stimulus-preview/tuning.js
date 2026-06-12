const TAU = Math.PI * 2;
const TUNING_RT = "rusty.optics.stimulus.tuning.v3";
const COMPATIBLE_TUNING_RTS = new Set([
  "rusty.optics.stimulus.tuning.v1",
  "rusty.optics.stimulus.tuning.v2",
  TUNING_RT,
]);

export function createDefaultTuning(profile = null) {
  const layerCount = profile?.layer_graph?.layers?.length ?? 1;
  const layerIndex = Math.min(2, Math.max(0, layerCount - 1));
  const layerOscillators = createDefaultLayerOscillators(layerCount);
  return {
    c: {
      colorCount: 2,
      col1: "#000000",
      col2: "#ffffff",
      col3: "#00aaff",
    },
    a: {
      oscActive: 0,
      oscFreq: 0.5,
      oscShape: 1,
      oscAmount: 0.2,
    },
    g: {
      scale: 1,
      shearX: 0,
      shearY: 0,
      offsetX: 0,
      offsetY: 0,
      shakeAmp: 0,
      shakeFreq: 5,
      rotSpeed: 0,
      stepFactor: 0,
      eccentricity: 0,
    },
    p: {
      trailAmount: 0,
      blurRadius: 0,
      glowStrength: 0,
      brightness: 0,
      contrast: 1,
    },
    x: {
      geometryMix: 1,
      fadeActive: 0,
      fadeFreq: 0.12,
      fadeAmount: 0,
      flatFreq: 2,
      flatShape: 1,
    },
    m: {
      layerBlendMode: 0,
      layerBlendAmount: 1,
    },
    e: {
      noiseFreq: 3.5,
      noiseStrength: 1,
      noiseBias: 0,
      vigCenter: 0,
      vigEdge: 0.08,
      vigBias: 0,
    },
    layer: {
      index: layerIndex,
      active: 1,
      strength: 1,
      speed: 1,
      frequency: 1,
      angle: 0,
      eccentricity: 0,
      twist: 0,
      pinch: 0,
      waveAmp: 0.18,
      waveFreq: 1,
      extent: 0,
    },
    layerOscillators,
    layerOsc: cloneTuning(layerOscillators[layerIndex] ?? defaultLayerOscillator(layerIndex)),
  };
}

export function createStimulusTuningPanel({ container, profile, initialTuning, onChange }) {
  let tuning = normalizeTuningForProfile(initialTuning ?? createDefaultTuning(profile), profile);
  container.replaceChildren();
  const form = document.createElement("form");
  form.className = "tuning-form";
  form.autocomplete = "off";
  form.addEventListener("submit", (event) => event.preventDefault());
  container.append(form);

  const layerOptions = (profile?.layer_graph?.layers ?? []).map((layer, index) => ({
    value: String(index),
    label: `${index + 1} ${compactLayerName(layer.layer_id ?? layer.pattern ?? "layer")}`,
  }));

  const controls = [
    group("Colors", [
      color("c.col1", "A"),
      color("c.col2", "B"),
      color("c.col3", "C"),
      rangeControl("c.colorCount", "Count", 2, 3, 1),
    ]),
    group("Motion", [
      rangeControl("a.oscActive", "Osc", 0, 1, 1),
      rangeControl("a.oscFreq", "Hz", 0, 12, 0.05),
      rangeControl("a.oscAmount", "Amt", 0, 1, 0.01),
      select("a.oscShape", "Shape", [
        ["0", "Sine"],
        ["1", "Square"],
        ["2", "Triangle"],
      ]),
    ]),
    group("Geometry", [
      rangeControl("g.scale", "Scale", 0.15, 6, 0.01),
      rangeControl("g.offsetX", "X", -1, 1, 0.01),
      rangeControl("g.offsetY", "Y", -1, 1, 0.01),
      rangeControl("g.shearX", "Shear X", -2, 2, 0.01),
      rangeControl("g.shearY", "Shear Y", -2, 2, 0.01),
      rangeControl("g.rotSpeed", "Rot/s", -2, 2, 0.01),
      rangeControl("g.shakeAmp", "Shake", 0, 0.5, 0.005),
      rangeControl("g.shakeFreq", "Shake Hz", 0, 24, 0.1),
      rangeControl("g.stepFactor", "Steps", 0, 16, 1),
      rangeControl("g.eccentricity", "Ecc", -0.95, 0.95, 0.01),
    ]),
    group("Effects", [
      rangeControl("p.brightness", "Bright", -1, 1, 0.01),
      rangeControl("p.contrast", "Contrast", 0, 4, 0.01),
      rangeControl("p.blurRadius", "Blur", 0, 18, 0.1),
      rangeControl("p.glowStrength", "Glow", 0, 1, 0.01),
      rangeControl("p.trailAmount", "Trail", 0, 1, 0.01),
    ]),
    group("Transition", [
      rangeControl("x.geometryMix", "Geom", 0, 1, 0.01),
      rangeControl("x.fadeActive", "Fade", 0, 1, 1),
      rangeControl("x.fadeFreq", "Fade Hz", 0, 4, 0.01),
      rangeControl("x.fadeAmount", "Fade Amt", 0, 1, 0.01),
      rangeControl("x.flatFreq", "Flat Hz", 0, 30, 0.05),
      select("x.flatShape", "Flat", [
        ["0", "Sine"],
        ["1", "Square"],
        ["2", "Triangle"],
      ]),
    ]),
    group("Blend", [
      select("m.layerBlendMode", "Mode", [
        ["0", "Stack"],
        ["1", "Mean"],
        ["2", "Cross"],
      ]),
      rangeControl("m.layerBlendAmount", "Blend", 0, 1, 0.01),
    ]),
    group("Noise", [
      rangeControl("e.noiseFreq", "Freq", 0.1, 16, 0.05),
      rangeControl("e.noiseStrength", "Strength", 0, 2, 0.01),
      rangeControl("e.noiseBias", "Bias", -1, 1, 0.01),
      rangeControl("e.vigEdge", "Edge", 0, 0.5, 0.005),
      rangeControl("e.vigCenter", "Center", 0, 1, 0.01),
      rangeControl("e.vigBias", "Bias", -1, 1, 0.01),
    ]),
    group("Layer", [
      select("layer.index", "Target", layerOptions.map((item) => [item.value, item.label])),
      rangeControl("layer.active", "Active", 0, 1, 1),
      rangeControl("layer.strength", "Strength", 0, 2, 0.01),
      rangeControl("layer.frequency", "Freq x", 0.1, 8, 0.01),
      rangeControl("layer.speed", "Speed x", -4, 4, 0.01),
      rangeControl("layer.angle", "Angle", -180, 180, 1),
      rangeControl("layer.eccentricity", "Ecc", -0.95, 0.95, 0.01),
      rangeControl("layer.twist", "Twist", -8, 8, 0.01),
      rangeControl("layer.pinch", "Pinch", -2, 2, 0.01),
      rangeControl("layer.waveAmp", "Wave", 0, 1, 0.01),
      rangeControl("layer.extent", "Extent", 0, 1, 0.01),
    ]),
    group("Layer Osc", [
      rangeControl("layerOsc.active", "Osc", 0, 1, 1),
      select("layerOsc.target", "Target", [
        ["opacity", "Opacity"],
        ["weight", "Weight"],
        ["frequency", "Freq"],
        ["phase", "Phase"],
        ["rotation", "Rotate"],
        ["wave", "Wave"],
      ]),
      rangeControl("layerOsc.freq", "Hz", 0, 12, 0.05),
      rangeControl("layerOsc.amount", "Amt", 0, 2, 0.01),
      rangeControl("layerOsc.phase", "Phase", 0, 1, 0.01),
      select("layerOsc.shape", "Shape", [
        ["0", "Sine"],
        ["1", "Square"],
        ["2", "Triangle"],
      ]),
    ]),
  ];

  for (const fieldset of controls) {
    form.append(fieldset);
  }

  syncFields(form, tuning);
  form.addEventListener("input", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLInputElement || target instanceof HTMLSelectElement)) {
      return;
    }
    const path = target.dataset.path;
    if (!path) {
      return;
    }
    const value = target.type === "color"
      ? target.value
      : Number.isFinite(Number.parseFloat(target.value))
        ? Number.parseFloat(target.value)
        : target.value;
    setPath(tuning, path, value);
    if (path === "layer.index") {
      syncLayerOscillatorView(tuning, profile);
      syncFields(form, tuning);
    } else if (path.startsWith("layerOsc.")) {
      storeVisibleLayerOscillator(tuning, profile);
      syncOutput(target, tuning);
    } else {
      syncOutput(target, tuning);
    }
    onChange?.(cloneTuning(tuning));
  });

  return {
    getTuning() {
      return cloneTuning(tuning);
    },
    setTuning(nextTuning) {
      tuning = normalizeTuningForProfile(nextTuning, profile);
      syncFields(form, tuning);
      onChange?.(cloneTuning(tuning));
    },
  };
}

export function applyTuningToProfile(baseProfile, tuning, elapsedSeconds = 0) {
  const tuned = structuredClone(baseProfile);
  const next = normalizeTuningForProfile(tuning, baseProfile);
  const layers = tuned.layer_graph?.layers ?? [];
  const osc = oscillatorValue(next.a, elapsedSeconds);
  const shake = next.g.shakeAmp * Math.sin(elapsedSeconds * next.g.shakeFreq * TAU);
  const rotation = elapsedSeconds * next.g.rotSpeed * TAU;
  const globalEccentricity = eccentricityScales(next.g.eccentricity);
  const transition = resolveTransition(next.x, next.c, elapsedSeconds);

  tuned.layer_graph.post = tuned.layer_graph.post ?? {};
  tuned.layer_graph.post.brightness = finite(next.p.brightness + osc + next.e.vigBias, 0);
  tuned.layer_graph.post.contrast = finite(next.p.contrast, 1);
  tuned.layer_graph.post.edge_fade = finite(next.e.vigEdge, 0);
  tuned.layer_graph.post.center_fade = finite(next.e.vigCenter, 0);
  tuned.layer_graph.post.noise_amount = 0;
  tuned.layer_graph.post.trail_decay = finite(next.p.trailAmount, 0);
  tuned.layer_graph.post.geometry_mix = transition.geometryMix;
  tuned.layer_graph.post.flat_color = transition.flatColor;
  tuned.layer_graph.post.layer_blend_mode = Math.max(0, Math.min(2, Math.floor(finite(next.m.layerBlendMode, 0))));
  tuned.layer_graph.post.layer_blend_amount = clamp01(next.m.layerBlendAmount);
  tuned.layer_graph.post.layer_blend_target = selectedLayerIndex(next, Math.max(1, layers.length));

  const ramp = colorRamp(next.c);
  layers.forEach((layer, index) => {
    layer.weight = finite(layer.weight, 1);
    layer.opacity = finite(layer.opacity, 1);
    layer.colors = rampForLayer(ramp, index);
    layer.warp = layer.warp ?? {};
    layer.warp.scale = {
      x: finite(layer.warp.scale?.x, 1) * next.g.scale * globalEccentricity.x,
      y: finite(layer.warp.scale?.y, 1) * next.g.scale * globalEccentricity.y,
    };
    layer.warp.offset = {
      x: finite(layer.warp.offset?.x, 0) + next.g.offsetX + shake,
      y: finite(layer.warp.offset?.y, 0) + next.g.offsetY - shake * 0.5,
    };
    layer.warp.shear_x = finite(layer.warp.shear_x, 0) + next.g.shearX;
    layer.warp.shear_y = finite(layer.warp.shear_y, 0) + next.g.shearY;
    layer.rotation_radians = finite(layer.rotation_radians, 0) + rotation;
    if (next.g.stepFactor > 0) {
      layer.phase_offset = Math.round(finite(layer.phase_offset, 0) * next.g.stepFactor) / next.g.stepFactor;
    }
    if (layer.pattern === "PerlinNoise" || layer.pattern === "NoiseField") {
      layer.spatial_frequency = next.e.noiseFreq;
      layer.noise = layer.noise ?? {};
      layer.noise.amplitude = next.e.noiseStrength;
      layer.noise.bias = next.e.noiseBias;
    }
  });

  const selected = layers[Math.max(0, Math.min(layers.length - 1, Math.floor(next.layer.index)))];
  if (selected) {
    selected.opacity = next.layer.active > 0 ? finite(selected.opacity, 1) : 0;
    selected.weight = finite(selected.weight, 1) * next.layer.strength;
    selected.spatial_frequency = finite(selected.spatial_frequency, 1) * next.layer.frequency;
    selected.rotation_radians = finite(selected.rotation_radians, 0) + degreesToRadians(next.layer.angle);
    selected.temporal = selected.temporal ?? {};
    selected.temporal.speed_hz = finite(selected.temporal.speed_hz, 0) * next.layer.speed;
    selected.warp = selected.warp ?? {};
    const layerEccentricity = eccentricityScales(next.layer.eccentricity);
    selected.warp.scale = selected.warp.scale ?? { x: 1, y: 1 };
    selected.warp.scale.x = finite(selected.warp.scale.x, 1) * layerEccentricity.x;
    selected.warp.scale.y = finite(selected.warp.scale.y, 1) * layerEccentricity.y;
    selected.warp.twist = finite(selected.warp.twist, 0) + next.layer.twist;
    selected.warp.pinch = finite(selected.warp.pinch, 0) + next.layer.pinch;
    selected.interference = selected.interference ?? {};
    selected.interference.wave_modulation = Math.max(
      finite(selected.interference.wave_modulation, 0),
      next.layer.waveAmp,
    );
    selected.interference.radial_decay = finite(selected.interference.radial_decay, 0) + next.layer.extent;
  }

  layers.forEach((layer, index) => {
    applyLayerOscillator(layer, next.layerOscillators[index], elapsedSeconds);
  });

  return tuned;
}

export function applyCanvasTuning(canvas, tuning) {
  const next = mergeTuning(createDefaultTuning(), tuning);
  const filters = [];
  if (next.p.blurRadius > 0) {
    filters.push(`blur(${next.p.blurRadius.toFixed(2)}px)`);
  }
  if (next.p.glowStrength > 0) {
    const glow = 4 + next.p.glowStrength * 18;
    filters.push(`drop-shadow(0 0 ${glow.toFixed(1)}px rgba(255,255,255,${(0.15 + next.p.glowStrength * 0.35).toFixed(3)}))`);
  }
  canvas.style.filter = filters.length > 0 ? filters.join(" ") : "none";
}

export function randomizeTuning(profile = null, seed = Date.now()) {
  const rng = mulberry32(seed >>> 0);
  const tuning = createDefaultTuning(profile);
  tuning.c.colorCount = rng() > 0.55 ? 3 : 2;
  tuning.c.col1 = randomColor(rng, 0.04, 0.22);
  tuning.c.col2 = randomColor(rng, 0.72, 1.0);
  tuning.c.col3 = randomColor(rng, 0.35, 0.9);
  tuning.a.oscActive = rng() > 0.55 ? 1 : 0;
  tuning.a.oscFreq = randomRange(rng, 0.1, 8);
  tuning.a.oscAmount = randomRange(rng, 0.02, 0.35);
  tuning.a.oscShape = Math.floor(randomRange(rng, 0, 3));
  tuning.g.scale = randomRange(rng, 0.75, 3.6);
  tuning.g.shearX = randomRange(rng, -0.45, 0.45);
  tuning.g.shearY = randomRange(rng, -0.45, 0.45);
  tuning.g.offsetX = randomRange(rng, -0.22, 0.22);
  tuning.g.offsetY = randomRange(rng, -0.22, 0.22);
  tuning.g.shakeAmp = rng() > 0.7 ? randomRange(rng, 0.005, 0.08) : 0;
  tuning.g.shakeFreq = randomRange(rng, 0.5, 12);
  tuning.g.rotSpeed = randomRange(rng, -0.08, 0.08);
  tuning.g.eccentricity = randomRange(rng, -0.65, 0.65);
  tuning.p.blurRadius = rng() > 0.7 ? randomRange(rng, 0.3, 2.4) : 0;
  tuning.p.glowStrength = rng() > 0.75 ? randomRange(rng, 0.08, 0.35) : 0;
  tuning.p.brightness = randomRange(rng, -0.18, 0.12);
  tuning.p.contrast = randomRange(rng, 0.85, 1.9);
  tuning.x.geometryMix = rng() > 0.25 ? randomRange(rng, 0.35, 1.0) : randomRange(rng, 0, 0.25);
  tuning.x.fadeActive = rng() > 0.72 ? 1 : 0;
  tuning.x.fadeFreq = randomRange(rng, 0.03, 0.8);
  tuning.x.fadeAmount = tuning.x.fadeActive > 0 ? randomRange(rng, 0.2, 0.85) : 0;
  tuning.x.flatFreq = randomRange(rng, 0.5, 12);
  tuning.x.flatShape = rng() > 0.35 ? 1 : Math.floor(randomRange(rng, 0, 3));
  tuning.m.layerBlendMode = Math.floor(randomRange(rng, 0, 3));
  tuning.m.layerBlendAmount = randomRange(rng, 0.15, 1.0);
  tuning.e.noiseFreq = randomRange(rng, 1.0, 9.0);
  tuning.e.noiseStrength = randomRange(rng, 0.05, 1.2);
  tuning.e.noiseBias = randomRange(rng, -0.18, 0.18);
  tuning.e.vigEdge = randomRange(rng, 0.02, 0.16);
  tuning.e.vigCenter = rng() > 0.8 ? randomRange(rng, 0.05, 0.35) : 0;
  tuning.layer.index = Math.floor(randomRange(rng, 0, Math.max(1, profile?.layer_graph?.layers?.length ?? 1)));
  tuning.layer.strength = randomRange(rng, 0.5, 1.5);
  tuning.layer.frequency = randomRange(rng, 0.65, 2.3);
  tuning.layer.speed = randomRange(rng, -1.8, 2.6);
  tuning.layer.angle = randomRange(rng, -45, 45);
  tuning.layer.eccentricity = randomRange(rng, -0.45, 0.45);
  tuning.layer.twist = randomRange(rng, -1.2, 1.8);
  tuning.layer.pinch = randomRange(rng, -0.35, 0.55);
  tuning.layer.waveAmp = randomRange(rng, 0, 0.45);
  tuning.layer.extent = randomRange(rng, 0, 0.65);
  tuning.layerOscillators = createDefaultLayerOscillators(profile?.layer_graph?.layers?.length ?? 1);
  tuning.layerOscillators.forEach((oscillator, index) => {
    oscillator.active = rng() > 0.62 ? 1 : 0;
    oscillator.target = ["opacity", "weight", "frequency", "phase", "rotation", "wave"][Math.floor(randomRange(rng, 0, 6))] ?? "opacity";
    oscillator.freq = randomRange(rng, 0.05, 4.0);
    oscillator.amount = oscillator.target === "phase" || oscillator.target === "rotation"
      ? randomRange(rng, 0.02, 0.35)
      : randomRange(rng, 0.1, 1.0);
    oscillator.phase = index / Math.max(1, tuning.layerOscillators.length);
    oscillator.shape = rng() > 0.65 ? 1 : 0;
  });
  tuning.layerOsc = cloneTuning(tuning.layerOscillators[tuning.layer.index] ?? defaultLayerOscillator(tuning.layer.index));
  return tuning;
}

export function loadTuningFromHash(hash, profile = null) {
  const raw = (hash ?? "").replace(/^#/, "").trim();
  if (!raw) {
    return null;
  }
  const decoded = decodeBase64Json(raw);
  if (!decoded) {
    return null;
  }
  if (COMPATIBLE_TUNING_RTS.has(decoded.rt)) {
    return normalizeTuningForProfile(decoded.tuning, profile);
  }
  return normalizeTuningForProfile(strobePresetToTuning(decoded, profile), profile);
}

export function tuningToHash(tuning) {
  const payload = {
    rt: TUNING_RT,
    tuning,
  };
  return encodeBase64Json(payload);
}

export function strobePresetToTuning(preset, profile = null) {
  const tuning = createDefaultTuning(profile);
  if (preset.c) {
    tuning.c.colorCount = finite(preset.c.colorCount, tuning.c.colorCount);
    tuning.c.col1 = normalizeHex(preset.c.col1, tuning.c.col1);
    tuning.c.col2 = normalizeHex(preset.c.col2, tuning.c.col2);
    tuning.c.col3 = normalizeHex(preset.c.col3, tuning.c.col3);
  }
  if (preset.a) {
    tuning.a.oscActive = finite(preset.a.oscActive, tuning.a.oscActive);
    tuning.a.oscFreq = finite(preset.a.oscFreq, tuning.a.oscFreq);
    tuning.a.oscShape = finite(preset.a.oscShape, tuning.a.oscShape);
  }
  if (preset.g) {
    tuning.g.scale = finite(preset.g.scale, tuning.g.scale);
    tuning.g.shearX = finite(preset.g.shearX, tuning.g.shearX);
    tuning.g.shearY = finite(preset.g.shearY, tuning.g.shearY);
    tuning.g.offsetX = finite(preset.g.offsetX, tuning.g.offsetX);
    tuning.g.offsetY = finite(preset.g.offsetY, tuning.g.offsetY);
    tuning.g.shakeAmp = finite(preset.g.shakeAmp, tuning.g.shakeAmp);
    tuning.g.shakeFreq = finite(preset.g.shakeFreq, tuning.g.shakeFreq);
    tuning.g.rotSpeed = finite(preset.g.rotSpeed, tuning.g.rotSpeed) / 8;
    tuning.g.stepFactor = finite(preset.g.stepFactor, tuning.g.stepFactor);
  }
  if (preset.p) {
    tuning.p.trailAmount = finite(preset.p.trailAmount, tuning.p.trailAmount);
    tuning.p.blurRadius = finite(preset.p.blurRadius, tuning.p.blurRadius);
    tuning.p.glowStrength = finite(preset.p.glowStrength, tuning.p.glowStrength);
    tuning.p.brightness = finite(preset.p.brightness, tuning.p.brightness);
    tuning.p.contrast = finite(preset.p.contrast, tuning.p.contrast);
  }
  if (preset.e) {
    tuning.e.noiseFreq = finite(preset.e.noiseFreq, tuning.e.noiseFreq);
    tuning.e.noiseStrength = finite(preset.e.noiseStrength, tuning.e.noiseStrength);
    tuning.e.noiseBias = finite(preset.e.noiseBias, tuning.e.noiseBias) - 0.5;
    tuning.e.vigCenter = finite(preset.e.vigCenter, tuning.e.vigCenter);
    tuning.e.vigEdge = Math.min(0.5, finite(preset.e.vigEdge, 0.5) / 16);
    tuning.e.vigBias = finite(preset.e.vigBias, tuning.e.vigBias);
  }
  const firstStripe = preset.s?.[0] ?? preset.r?.[0];
  if (firstStripe) {
    tuning.layer.active = finite(firstStripe.active, tuning.layer.active);
    tuning.layer.strength = finite(firstStripe.strength, tuning.layer.strength);
    tuning.layer.speed = finite(firstStripe.speed, tuning.layer.speed);
    tuning.layer.frequency = finite(firstStripe.waveFreq, tuning.layer.frequency);
    tuning.layer.angle = finite(firstStripe.angle, tuning.layer.angle);
    tuning.layer.twist = finite(firstStripe.distortAmp, tuning.layer.twist);
    tuning.layer.waveAmp = finite(firstStripe.waveAmp, tuning.layer.waveAmp);
    tuning.layer.extent = finite(firstStripe.extent, tuning.layer.extent);
  }
  return tuning;
}

function normalizeTuningForProfile(source, profile = null) {
  const next = mergeTuning(createDefaultTuning(profile), source);
  ensureLayerOscillators(next, profile);
  syncLayerOscillatorView(next, profile);
  return next;
}

function createDefaultLayerOscillators(layerCount) {
  return Array.from({ length: Math.max(1, layerCount) }, (_, index) => defaultLayerOscillator(index));
}

function defaultLayerOscillator(index = 0) {
  return {
    active: 0,
    target: "opacity",
    freq: 0.5,
    amount: 0,
    phase: index * 0.125,
    shape: 0,
  };
}

function ensureLayerOscillators(tuning, profile = null) {
  const layerCount = Math.max(1, profile?.layer_graph?.layers?.length ?? tuning.layerOscillators?.length ?? 1);
  const defaults = createDefaultLayerOscillators(layerCount);
  if (!Array.isArray(tuning.layerOscillators)) {
    tuning.layerOscillators = [];
  }
  for (let index = 0; index < layerCount; index += 1) {
    tuning.layerOscillators[index] = mergeTuning(defaults[index], tuning.layerOscillators[index]);
  }
  tuning.layerOscillators.length = layerCount;
  tuning.layerOsc = mergeTuning(defaultLayerOscillator(selectedLayerIndex(tuning, layerCount)), tuning.layerOsc);
}

function syncLayerOscillatorView(tuning, profile = null) {
  ensureLayerOscillators(tuning, profile);
  const index = selectedLayerIndex(tuning, tuning.layerOscillators.length);
  tuning.layerOsc = cloneTuning(tuning.layerOscillators[index]);
}

function storeVisibleLayerOscillator(tuning, profile = null) {
  ensureLayerOscillators(tuning, profile);
  const index = selectedLayerIndex(tuning, tuning.layerOscillators.length);
  tuning.layerOscillators[index] = mergeTuning(defaultLayerOscillator(index), tuning.layerOsc);
}

function selectedLayerIndex(tuning, layerCount) {
  return Math.max(0, Math.min(Math.max(0, layerCount - 1), Math.floor(finite(tuning.layer?.index, 0))));
}

function resolveTransition(transition, colors, elapsedSeconds) {
  const flatValue = oscillatorUnitValue({
    freq: transition.flatFreq,
    shape: transition.flatShape,
    phase: 0,
  }, elapsedSeconds);
  let geometryMix = finite(transition.geometryMix, 1);
  if (finite(transition.fadeActive, 0) > 0.5 && finite(transition.fadeAmount, 0) > 0) {
    const fadeSigned = oscillatorSignedUnitValue({
      freq: transition.fadeFreq,
      shape: 0,
      phase: 0,
    }, elapsedSeconds);
    geometryMix += fadeSigned * finite(transition.fadeAmount, 0);
  }
  const ramp = colorRamp(colors);
  return {
    geometryMix: clamp01(geometryMix),
    flatColor: mixColor(ramp[0], ramp[Math.min(1, ramp.length - 1)], flatValue),
  };
}

function applyLayerOscillator(layer, oscillator, elapsedSeconds) {
  if (!oscillator || finite(oscillator.active, 0) < 0.5 || finite(oscillator.amount, 0) === 0) {
    return;
  }
  const amount = finite(oscillator.amount, 0);
  const signed = oscillatorSignedUnitValue(oscillator, elapsedSeconds);
  const unit = 0.5 + signed * 0.5;
  const target = String(oscillator.target ?? "opacity");
  if (target === "opacity") {
    const fade = clamp01(1 - amount + amount * unit);
    layer.opacity = finite(layer.opacity, 1) * fade;
  } else if (target === "weight") {
    const fade = clamp01(1 - amount + amount * unit);
    layer.weight = finite(layer.weight, 1) * fade;
  } else if (target === "frequency") {
    layer.spatial_frequency = finite(layer.spatial_frequency, 1) * Math.max(0.02, 1 + signed * amount);
  } else if (target === "phase") {
    layer.phase_offset = finite(layer.phase_offset, 0) + signed * amount;
  } else if (target === "rotation") {
    layer.rotation_radians = finite(layer.rotation_radians, 0) + signed * amount * TAU;
  } else if (target === "wave") {
    layer.interference = layer.interference ?? {};
    layer.interference.wave_modulation = finite(layer.interference.wave_modulation, 0) + signed * amount;
  }
}

function oscillatorSignedUnitValue(oscillator, elapsedSeconds) {
  const phase = elapsedSeconds * finite(oscillator.freq, 0) + finite(oscillator.phase, 0);
  const shape = Number(oscillator.shape);
  if (shape === 1) {
    return Math.sin(phase * TAU) >= 0 ? 1 : -1;
  }
  if (shape === 2) {
    return 4 * Math.abs((phase % 1) - 0.5) - 1;
  }
  return Math.sin(phase * TAU);
}

function oscillatorUnitValue(oscillator, elapsedSeconds) {
  return clamp01(0.5 + oscillatorSignedUnitValue(oscillator, elapsedSeconds) * 0.5);
}

function eccentricityScales(value) {
  const eccentricity = Math.max(-0.95, Math.min(0.95, finite(value, 0)));
  const axis = 1 + Math.abs(eccentricity) * 2;
  return eccentricity >= 0
    ? { x: axis, y: 1 / axis }
    : { x: 1 / axis, y: axis };
}

function mixColor(a, b, t) {
  return {
    r: mix(a.r, b.r, t),
    g: mix(a.g, b.g, t),
    b: mix(a.b, b.b, t),
    a: 1,
  };
}

function group(label, children) {
  const fieldset = document.createElement("fieldset");
  const legend = document.createElement("legend");
  legend.textContent = label;
  fieldset.append(legend);
  for (const child of children) {
    fieldset.append(child);
  }
  return fieldset;
}

function rangeControl(path, label, min, max, step) {
  const id = `tuning-${path.replaceAll(".", "-")}`;
  const wrapper = document.createElement("label");
  wrapper.htmlFor = id;
  wrapper.className = "range-control";
  const name = document.createElement("span");
  name.textContent = label;
  const input = document.createElement("input");
  input.id = id;
  input.type = "range";
  input.min = String(min);
  input.max = String(max);
  input.step = String(step);
  input.dataset.path = path;
  const output = document.createElement("output");
  output.dataset.for = id;
  wrapper.append(name, input, output);
  return wrapper;
}

function color(path, label) {
  const id = `tuning-${path.replaceAll(".", "-")}`;
  const wrapper = document.createElement("label");
  wrapper.htmlFor = id;
  wrapper.className = "color-control";
  const name = document.createElement("span");
  name.textContent = label;
  const input = document.createElement("input");
  input.id = id;
  input.type = "color";
  input.dataset.path = path;
  wrapper.append(name, input);
  return wrapper;
}

function select(path, label, options) {
  const id = `tuning-${path.replaceAll(".", "-")}`;
  const wrapper = document.createElement("label");
  wrapper.htmlFor = id;
  wrapper.className = "select-control";
  const name = document.createElement("span");
  name.textContent = label;
  const input = document.createElement("select");
  input.id = id;
  input.dataset.path = path;
  for (const [value, text] of options) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = text;
    input.append(option);
  }
  wrapper.append(name, input);
  return wrapper;
}

function syncFields(form, tuning) {
  for (const input of form.querySelectorAll("[data-path]")) {
    const value = getPath(tuning, input.dataset.path);
    input.value = input.type === "color" ? normalizeHex(value, "#000000") : String(value);
    syncOutput(input, tuning);
  }
}

function syncOutput(input, tuning) {
  const output = input.parentElement?.querySelector(`output[data-for="${input.id}"]`);
  if (!output) {
    return;
  }
  const value = getPath(tuning, input.dataset.path);
  output.value = typeof value === "number" ? compactNumber(value) : String(value);
}

function colorRamp(colors) {
  const col1 = hexToColor(colors.col1);
  const col2 = hexToColor(colors.col2);
  const col3 = hexToColor(colors.col3);
  return colors.colorCount >= 3 ? [col1, col2, col3] : [col1, col2];
}

function rampForLayer(ramp, index) {
  if (ramp.length < 3) {
    return [
      { position: 0, color: { ...ramp[0], a: 1 } },
      { position: 1, color: { ...ramp[1], a: 1 } },
    ];
  }
  const high = index % 2 === 0 ? ramp[1] : ramp[2];
  return [
    { position: 0, color: { ...ramp[0], a: 1 } },
    { position: 1, color: { ...high, a: 1 } },
  ];
}

function oscillatorValue(oscillator, elapsedSeconds) {
  if (oscillator.oscActive < 0.5 || oscillator.oscAmount === 0) {
    return 0;
  }
  const phase = elapsedSeconds * oscillator.oscFreq;
  let value = Math.sin(phase * TAU);
  if (Number(oscillator.oscShape) === 1) {
    value = value >= 0 ? 1 : -1;
  } else if (Number(oscillator.oscShape) === 2) {
    value = 4 * Math.abs((phase % 1) - 0.5) - 1;
  }
  return value * oscillator.oscAmount;
}

function decodeBase64Json(raw) {
  try {
    const padded = raw.replaceAll("-", "+").replaceAll("_", "/").padEnd(Math.ceil(raw.length / 4) * 4, "=");
    let json = atob(padded);
    json = json.replaceAll('"speed""', '"speed"');
    return JSON.parse(json);
  } catch {
    return null;
  }
}

function encodeBase64Json(value) {
  const json = JSON.stringify(value);
  return btoa(json).replaceAll("+", "-").replaceAll("/", "_").replaceAll("=", "");
}

function mergeTuning(base, override) {
  if (!override) {
    return cloneTuning(base);
  }
  const next = cloneTuning(base);
  deepAssign(next, override);
  return next;
}

function deepAssign(target, source) {
  for (const [key, value] of Object.entries(source ?? {})) {
    if (value && typeof value === "object" && !Array.isArray(value)) {
      target[key] = target[key] && typeof target[key] === "object" ? target[key] : {};
      deepAssign(target[key], value);
    } else {
      target[key] = value;
    }
  }
}

function cloneTuning(value) {
  return structuredClone(value);
}

function getPath(source, path) {
  return path.split(".").reduce((value, key) => value?.[key], source);
}

function setPath(target, path, value) {
  const keys = path.split(".");
  let cursor = target;
  for (let i = 0; i < keys.length - 1; i += 1) {
    cursor[keys[i]] = cursor[keys[i]] ?? {};
    cursor = cursor[keys[i]];
  }
  cursor[keys[keys.length - 1]] = value;
}

function compactLayerName(id) {
  return String(id).split(".").slice(-2).join(".");
}

function compactNumber(value) {
  if (!Number.isFinite(value)) {
    return "0";
  }
  if (Math.abs(value) >= 10 || Number.isInteger(value)) {
    return value.toFixed(0);
  }
  return value.toFixed(2);
}

function normalizeHex(value, fallback) {
  return /^#[0-9a-f]{6}$/i.test(String(value)) ? String(value) : fallback;
}

function hexToColor(hex) {
  const value = normalizeHex(hex, "#ffffff");
  return {
    r: Number.parseInt(value.slice(1, 3), 16) / 255,
    g: Number.parseInt(value.slice(3, 5), 16) / 255,
    b: Number.parseInt(value.slice(5, 7), 16) / 255,
  };
}

function finite(value, fallback) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}

function degreesToRadians(value) {
  return finite(value, 0) * Math.PI / 180;
}

function mix(a, b, t) {
  return a * (1 - t) + b * t;
}

function clamp01(value) {
  return Math.min(1, Math.max(0, finite(value, 0)));
}

function randomColor(rng, min, max) {
  const channels = [randomRange(rng, min, max), randomRange(rng, min, max), randomRange(rng, min, max)];
  return `#${channels.map((value) => Math.round(value * 255).toString(16).padStart(2, "0")).join("")}`;
}

function randomRange(rng, min, max) {
  return min + (max - min) * rng();
}

function mulberry32(seed) {
  return function next() {
    let t = seed += 0x6d2b79f5;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
