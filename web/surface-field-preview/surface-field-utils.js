// Pure helper functions for the surface-field browser adapter.
(function () {
  "use strict";

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

function formatDeltaNumber(value) {
  const number = Number(value) || 0;
  const magnitude = Math.abs(number);
  if (magnitude > 0 && magnitude < 0.01) {
    return number.toFixed(4);
  }
  return number.toFixed(2);
}

function signedFormatNumber(value) {
  const number = Number(value);
  return `${number >= 0 ? "+" : ""}${number.toFixed(2)}`;
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

  window.RustyOpticsSurfaceFieldUtils = Object.freeze({
    clamp,
    computeBounds,
    computeBoundsFromPositions,
    edgeColor,
    effectKind,
    formatDeltaNumber,
    formatNumber,
    perturbationColor,
    rgba,
    scalarColor,
    signedFormatNumber,
    targetField,
    vectorColor,
    vectorLength,
    visualNodeRadius,
  });
})();