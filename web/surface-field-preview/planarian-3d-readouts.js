// Planarian 3D readout vocabulary and pure formatting helpers.
(function () {
  "use strict";

  const { formatNumber, signedFormatNumber } = window.RustyOpticsSurfaceFieldUtils;

const PLANARIAN_SCENARIOS = new Map([
  [0, { label: "baseline", outcome: "stable AP identity" }],
  [1, { label: "wound", outcome: "cut-band depolarization" }],
  [2, { label: "gap block", outcome: "reduced cross-band coupling" }],
  [3, { label: "memory", outcome: "posterior head-memory persistence" }],
  [4, { label: "no-memory", outcome: "transient relaxes" }],
]);
const PLANARIAN_GRAPH_DENSITIES = new Map([
  [0, { label: "standard", nodes: 160 }],
  [1, { label: "dense", nodes: 320 }],
  [2, { label: "intricate", nodes: 360 }],
  [3, { label: "render", nodes: 720 }],
]);
const PLANARIAN_SOURCE_LABELS = new Map([
  ["durant_2019_bpj", "Durant 2019"],
  ["beane_2011_chembiol", "Beane 2011"],
  ["oviedo_2010_devbiol", "Oviedo 2010"],
  ["beane_2013_dev", "Beane 2013"],
  ["emmons_bell_2015_ijms", "Emmons-Bell 2015"],
  ["planformdb_250", "PlanformDB 2.5.0"],
]);
const PLANARIAN_TARGET_LABELS = new Map([
  ["ap_transient_memory", "early transient memory"],
  ["gap_block_conductance", "gap conductance"],
  ["head_vs_tail_voltage", "head/tail voltage context"],
  ["head_size_scaling", "future size metric"],
  ["species_like_head_labels", "future head labels"],
  ["planformdb_curated_subset", "curated PlanformDB records"],
  ["persistent_axis_recut_history", "future persistent-axis history"],
]);
const PLANARIAN_REGION_LABELS = new Map([
  [1, "tail"],
  [2, "post trunk"],
  [3, "pharynx"],
  [4, "pre trunk"],
  [5, "head"],
]);
const PLANARIAN_OUTCOME_TRACE_STRIDE = 7;
const PLANARIAN_EDIT_EVENT_STRIDE = 15;
const PLANARIAN_EDIT_TARGET_STRIDE = 8;
const PLANARIAN_NEIGHBORHOOD_TARGET_STRIDE = 3;
const PLANARIAN_EVENT_TIMELINE_LIMIT = 8;
const PLANARIAN_OUTCOME_METRICS = [
  { key: "posterior_memory", label: "post memory", color: "#73d9bb" },
  { key: "posterior_head_identity", label: "post head", color: "#f1c65c" },
  { key: "tail_identity_at_tail", label: "tail", color: "#83a7ff" },
];
const PLANARIAN_COMPARISON_METRICS = [
  { key: "posterior_memory", label: "compare memory", color: "#73d9bb" },
  { key: "posterior_head_identity", label: "compare head", color: "#f1c65c" },
];
const PLANARIAN_EDIT_OPERATION_LABELS = new Map([
  [1, "set V"],
  [2, "dV"],
  [3, "memory"],
  [4, "node g"],
  [5, "gate theta"],
  [6, "gate mult"],
  [7, "pulse"],
  [8, "brush dV"],
]);
const PLANARIAN_EDIT_TARGET_LABELS = new Map([
  [1, "node"],
  [2, "edge"],
  [3, "current"],
]);

function planarianLayerLegendLabel(layer) {
  if (layer === "circuit.activity") {
    return "activity dV";
  }
  if (layer === "circuit.voltage") {
    return "voltage";
  }
  if (layer === "circuit.memory") {
    return "memory";
  }
  if (layer.includes("head_identity")) {
    return "head id";
  }
  if (layer.includes("tail_identity")) {
    return "tail id";
  }
  return "layer";
}

function planarian3DAnchorSummary(anchors) {
  const labels = (anchors || [])
    .map(parsePlanarianSourceTargetAnchor)
    .filter(Boolean)
    .map(formatPlanarianSourceTargetAnchor);
  if (labels.length === 0) {
    return "source targets pending";
  }
  return `anchors ${labels.join("; ")}`;
}

function formatPlanarianSourceTargetAnchor(anchor) {
  const sources = (anchor.source_ids || [anchor.source_id])
    .filter(Boolean)
    .map((sourceId) => PLANARIAN_SOURCE_LABELS.get(sourceId) || sourceId)
    .join(" + ");
  const target = PLANARIAN_TARGET_LABELS.get(anchor.target_id) || anchor.target_id;
  return `${sources}: ${target}`;
}

function parsePlanarianSourceTargetAnchor(anchor) {
  if (typeof anchor !== "string" || !anchor.trim()) {
    return null;
  }
  const parts = anchor.split("::");
  const sourcePart = parts.find((part) => part.startsWith("source:"));
  const target = parts.find((part) => part.startsWith("target:"));
  const sourceIds = sourcePart
    ? sourcePart
      .split(";")
      .map((part) => part.trim())
      .filter(Boolean)
      .map((part) => part.startsWith("source:") ? part.slice("source:".length) : part)
      .filter(Boolean)
    : [];
  const targetId = target ? target.slice("target:".length) : "";
  if (sourceIds.length === 0 || !targetId) {
    return null;
  }
  return {
    source_id: sourceIds[0],
    source_ids: sourceIds,
    target_id: targetId,
    status: parts
      .filter((part) => !part.startsWith("source:") && !part.startsWith("target:"))
      .join("::"),
    flags: parts.filter((part) => part.startsWith("future_")),
  };
}

function planarianSourceTargetsFromAnchors(anchors) {
  return (anchors || [])
    .map(parsePlanarianSourceTargetAnchor)
    .filter(Boolean);
}

function planarianSourceTargetPolicy(sourceTargets) {
  const mentionsPlanformDb = sourceTargets.some((target) =>
    target.source_ids?.includes("planformdb_250") || target.target_id.includes("planformdb"));
  return [
    "source targets are metadata over Matter-owned synthetic educational dynamics",
    "not calibrated physiology",
    "not source-fit thresholds or stochastic prediction",
    mentionsPlanformDb
      ? "PlanformDB-derived records are provenance and review metadata only"
      : "PlanformDB-derived records remain provenance and review metadata only",
  ].join("; ");
}

function formatPlanarianEditEvent(event) {
  const operation = PLANARIAN_EDIT_OPERATION_LABELS.get(event.operation_code) || "edit";
  const status = event.accepted ? "accepted" : "rejected";
  const revision = `r${event.revision_before}->${event.revision_after}`;
  const clamped = event.clamped_values > 0 ? `${event.clamped_values} clamped` : null;
  return [
    `#${event.event_index}`,
    `t ${formatNumber(event.time_seconds)}s`,
    `${operation} ${planarianEditTargetLabel(event)}`,
    planarianEditValueLabel(event),
    status,
    revision,
    clamped,
  ].filter(Boolean).join(" ");
}

function planarianEditTargetLabel(event) {
  const kind = PLANARIAN_EDIT_TARGET_LABELS.get(event.target_kind) || "target";
  if (event.target_index >= 0) {
    return `${kind} ${event.target_index}`;
  }
  return kind;
}

function planarianEditValueLabel(event) {
  switch (event.operation_code) {
    case 1:
      return `V ${signedFormatNumber(event.value_a)}`;
    case 2:
      return `dV ${signedFormatNumber(event.value_a)}`;
    case 3:
      return `memory ${formatNumber(event.value_a)}`;
    case 4:
      return `scale ${formatNumber(event.value_a)}`;
    case 5:
      return `theta ${signedFormatNumber(event.value_a)}`;
    case 6:
      return `mult ${formatNumber(event.value_a)}-${formatNumber(event.value_b)}`;
    case 7:
      return `${signedFormatNumber(event.value_a)} ${Math.trunc(event.value_b)} steps`;
    case 8:
      return `dV ${signedFormatNumber(event.value_a)} tier ${Math.trunc(event.value_b)}`;
    default:
      return null;
  }
}

function planarianGraphDensityCodeFromValue(value, fallback = 2) {
  if (value === null || value === undefined || String(value).trim() === "") {
    return fallback;
  }
  const code = Math.trunc(Number(value));
  return PLANARIAN_GRAPH_DENSITIES.has(code) ? code : fallback;
}

function planarianGraphDensityInfo(densityCode) {
  const code = planarianGraphDensityCodeFromValue(densityCode, 2);
  return PLANARIAN_GRAPH_DENSITIES.get(code) || PLANARIAN_GRAPH_DENSITIES.get(2);
}

function operationKind(operation) {
  return Object.keys(operation || {})[0] || "Unknown";
}

function scenarioInfo(scenarioCode) {
  return PLANARIAN_SCENARIOS.get(Math.trunc(scenarioCode ?? 3))
    || { label: "scenario", outcome: "Matter preset" };
}

function regionLabelForCode(regionCode) {
  return PLANARIAN_REGION_LABELS.get(Math.trunc(regionCode || 0)) || "region";
}

function planarianTeachingRelationLabel(scenarioCode, comparisonCode) {
  const scenario = Math.trunc(scenarioCode ?? 3);
  const comparison = comparisonCode === null || comparisonCode === undefined
    ? null
    : Math.trunc(comparisonCode);
  if (scenario === 3 && comparison === 4) {
    return "memory vs no-memory control";
  }
  if (scenario === 4 && comparison === 3) {
    return "no-memory control vs memory";
  }
  if (scenario === 2 && comparison === 0) {
    return "gap block vs baseline";
  }
  if (scenario === 1 && comparison === 0) {
    return "wound vs baseline";
  }
  if (comparison !== null) {
    return `${scenarioInfo(scenario).label} vs ${scenarioInfo(comparison).label}`;
  }
  return "single Matter trace";
}

  window.RustyOpticsPlanarian3DReadouts = Object.freeze({
    PLANARIAN_COMPARISON_METRICS,
    PLANARIAN_EDIT_EVENT_STRIDE,
    PLANARIAN_EDIT_OPERATION_LABELS,
    PLANARIAN_EDIT_TARGET_STRIDE,
    PLANARIAN_EVENT_TIMELINE_LIMIT,
    PLANARIAN_NEIGHBORHOOD_TARGET_STRIDE,
    PLANARIAN_OUTCOME_METRICS,
    PLANARIAN_OUTCOME_TRACE_STRIDE,
    formatPlanarianEditEvent,
    formatPlanarianSourceTargetAnchor,
    operationKind,
    planarian3DAnchorSummary,
    planarianGraphDensityCodeFromValue,
    planarianGraphDensityInfo,
    planarianLayerLegendLabel,
    planarianSourceTargetPolicy,
    planarianSourceTargetsFromAnchors,
    planarianTeachingRelationLabel,
    regionLabelForCode,
    scenarioInfo,
  });
})();