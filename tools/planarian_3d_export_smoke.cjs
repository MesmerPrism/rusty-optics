#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

let chromium;
try {
  ({ chromium } = require("playwright"));
} catch (error) {
  throw new Error([
    "Playwright is required for the Planarian 3D export smoke.",
    "Set NODE_PATH to the Codex bundled node_modules path or install Playwright for this repo.",
    error.message,
  ].join(" "));
}

const DEFAULTS = {
  url: "http://127.0.0.1:8792/web/surface-field-preview/",
  outDir: "local-artifacts/planarian3d-export-smoke",
  format: "gif",
  views: ["surface", "graph"],
  palette: "neon-rgb",
  start: "reset",
  loop: "showcase",
  layer: "activity",
  material: "opaque",
  seconds: 8,
  fps: 12,
  width: 720,
  height: 860,
  step: 1,
  warmup: 0,
  density: 3,
  readyTimeoutMs: 120000,
  exportTimeoutMs: 300000,
  python: "python",
  report: "",
};

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (!["gif", "apng"].includes(options.format)) {
    throw new Error("This smoke validates GIF or APNG exports only; use --format gif or --format apng");
  }

  fs.mkdirSync(options.outDir, { recursive: true });
  const reportPath = options.report
    ? path.resolve(options.report)
    : path.join(options.outDir, `planarian_3d_export_${options.format}_smoke_report.json`);

  const browser = await chromium.launch({
    headless: true,
    args: [
      "--use-gl=swiftshader",
      "--enable-unsafe-swiftshader",
      "--disable-dev-shm-usage",
    ],
  });

  const outputs = [];
  try {
    for (const view of options.views) {
      const page = await browser.newPage({
        acceptDownloads: true,
        viewport: {
          width: Math.max(1024, options.width + 280),
          height: Math.max(900, Math.min(1200, options.height + 180)),
        },
        deviceScaleFactor: 1,
      });
      page.setDefaultTimeout(Math.max(options.readyTimeoutMs, options.exportTimeoutMs));
      try {
        const output = await exportView(page, options, view);
        outputs.push(output);
        console.log(`${view}: ${output.frame_count} frames ${output.width}x${output.height} ${output.path}`);
      } finally {
        await page.close();
      }
    }
  } finally {
    await browser.close();
  }

  const report = {
    url: options.url,
    format: options.format,
    views: options.views,
    expected: {
      frame_count: options.seconds * options.fps,
      width: options.width,
      height: options.height,
      fps: options.fps,
    },
    outputs,
  };
  fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`, "utf8");
  console.log(`report: ${reportPath}`);
}

async function exportView(page, options, view) {
  const url = new URL(options.url);
  url.searchParams.set("view", "planarian3d");
  url.searchParams.set("density", String(options.density));
  await page.goto(url.toString(), { waitUntil: "domcontentloaded", timeout: options.readyTimeoutMs });
  await waitForPlanarian3DReady(page, options.readyTimeoutMs);
  await configureExportControls(page, options, view);
  await waitForPlanarianTeachingReadout(page, options.readyTimeoutMs);

  const [download] = await Promise.all([
    page.waitForEvent("download", { timeout: options.exportTimeoutMs }),
    page.evaluate(() => {
      const button = document.querySelector("#export-planarian-gif");
      if (!button) {
        throw new Error("Planarian export button is missing");
      }
      button.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
    }),
  ]);
  await page.waitForFunction(
    () => document.querySelector("#viewport-3d")?.dataset.planarian3dExportStatus === "saved",
    undefined,
    { timeout: options.exportTimeoutMs },
  );

  const fallbackExtension = options.format === "apng" ? "png" : "gif";
  const suggestedName = sanitizeFilename(download.suggestedFilename() || `planarian-${view}.${fallbackExtension}`);
  const downloadPath = path.join(options.outDir, suggestedName);
  await download.saveAs(downloadPath);

  const pageMetadata = await page.$eval("#viewport-3d", (element) => {
    const raw = element.dataset.planarian3dExportMetadata || "{}";
    try {
      return JSON.parse(raw);
    } catch {
      return {};
    }
  });
  const metadataValidation = validateExportMetadata(pageMetadata, options, view);
  const validation = validateExportWithPillow(options.python, downloadPath, options.format, {
    frameCount: options.seconds * options.fps,
    width: options.width,
    height: options.height,
  });
  const stat = fs.statSync(downloadPath);
  return {
    view,
    path: path.resolve(downloadPath),
    bytes: stat.size,
    frame_count: validation.frame_count,
    width: validation.width,
    height: validation.height,
    mode: validation.mode,
    validation,
    metadata_validation: metadataValidation,
    dynamics: pageMetadata.dynamics || "",
    page_metadata: pageMetadata,
  };
}

async function waitForPlanarianTeachingReadout(page, timeout) {
  await page.selectOption("#planarian-compare-scenario", "4");
  await page.dispatchEvent("#planarian-compare-scenario", "change");
  await page.waitForFunction(
    () => {
      const viewport = document.querySelector("#viewport-3d");
      const relation = viewport?.dataset.planarian3dOutcomeTeachingRelation || "";
      const readout = viewport?.dataset.planarian3dOutcomeTeachingReadout || "";
      return relation.includes("memory vs no-memory")
        && readout.includes("posterior transient depolarization");
    },
    undefined,
    { timeout },
  );
}

function validateExportMetadata(metadata, options, view) {
  const errors = [];
  if (metadata.format !== options.format) {
    errors.push(`expected metadata format ${options.format}, got ${metadata.format || "<missing>"}`);
  }
  if (metadata.view !== view) {
    errors.push(`expected metadata view ${view}, got ${metadata.view || "<missing>"}`);
  }
  if (metadata.voltage_unit !== "normalized") {
    errors.push(`expected normalized voltage unit, got ${metadata.voltage_unit || "<missing>"}`);
  }
  if (!String(metadata.voltage_unit_policy || "").includes("not calibrated millivolts")) {
    errors.push("metadata voltage_unit_policy must state that this is not calibrated millivolts");
  }
  if (!String(metadata.evidence_type || "").includes("synthetic")) {
    errors.push("metadata evidence_type must identify a synthetic educational source");
  }
  if (!String(metadata.dynamics || "").includes("not a PlanformDB-derived predictor")) {
    errors.push("metadata dynamics must state that the export is not a PlanformDB-derived predictor");
  }
  if (!String(metadata.expected_outcome || "").trim()) {
    errors.push("metadata expected_outcome must be populated");
  }
  const anchors = Array.isArray(metadata.literature_anchors) ? metadata.literature_anchors : [];
  if (!anchors.some((anchor) => String(anchor).includes("target:head_vs_tail_voltage"))) {
    errors.push("metadata literature_anchors must include head_vs_tail_voltage");
  }
  if (!anchors.some((anchor) => String(anchor).includes("target:ap_transient_memory"))) {
    errors.push("metadata literature_anchors must include ap_transient_memory for the default memory showcase");
  }
  const sourceTargets = Array.isArray(metadata.source_targets) ? metadata.source_targets : [];
  const sourceTargetIds = new Set(sourceTargets.map((target) => String(target.target_id || "")));
  if (!sourceTargetIds.has("head_vs_tail_voltage")) {
    errors.push("metadata source_targets must include head_vs_tail_voltage");
  }
  if (!sourceTargetIds.has("ap_transient_memory")) {
    errors.push("metadata source_targets must include ap_transient_memory");
  }
  if (!sourceTargets.some((target) => Array.isArray(target.source_ids) && target.source_ids.length > 0)) {
    errors.push("metadata source_targets must include parsed source_ids");
  }
  const sourceTargetPolicy = String(metadata.source_target_policy || "");
  if (!sourceTargetPolicy.includes("not calibrated physiology")) {
    errors.push("metadata source_target_policy must preserve the non-calibrated physiology boundary");
  }
  if (!sourceTargetPolicy.includes("PlanformDB-derived records")) {
    errors.push("metadata source_target_policy must mention PlanformDB-derived records as provenance/review metadata");
  }
  if (errors.length) {
    throw new Error(`Export metadata validation failed: ${errors.join("; ")}`);
  }
  return {
    voltage_unit: metadata.voltage_unit,
    evidence_type: metadata.evidence_type,
    literature_anchor_count: anchors.length,
    source_target_count: sourceTargets.length,
    source_target_policy: sourceTargetPolicy,
  };
}

async function waitForPlanarian3DReady(page, timeout) {
  await page.waitForFunction(
    () => {
      const status = document.querySelector("#runtime-status")?.textContent?.trim();
      const viewport = document.querySelector("#viewport-3d");
      const bodyVertexCount = Number(viewport?.dataset.bodyVertexCount || 0);
      const anchorCount = Number(viewport?.dataset.sampleAnchorCount || 0);
      const adapterStage = viewport?.dataset.planarian3dAdapterStage || "";
      return status === "Matter 3D"
        && adapterStage === "ready"
        && bodyVertexCount >= 1000
        && anchorCount > 0;
    },
    undefined,
    { timeout },
  );
}

async function configureExportControls(page, options, view) {
  await page.selectOption("#planarian-export-format", options.format);
  await page.selectOption("#planarian-gif-view", view);
  await page.selectOption("#planarian-gif-palette", options.palette);
  await page.selectOption("#planarian-export-start", options.start);
  await page.selectOption("#planarian-export-loop", options.loop);
  await page.selectOption("#planarian-export-layer", options.layer);
  await page.selectOption("#planarian-export-material", options.material);
  await page.fill("#planarian-gif-seconds", String(options.seconds));
  await page.fill("#planarian-gif-fps", String(options.fps));
  await page.fill("#planarian-gif-width", String(options.width));
  await page.fill("#planarian-gif-height", String(options.height));
  await page.fill("#planarian-gif-steps", String(options.step));
  await page.fill("#planarian-export-warmup", String(options.warmup));
}

function validateExportWithPillow(python, filePath, format, expected) {
  const code = [
    "import json, sys",
    "import struct",
    "from PIL import Image, ImageSequence",
    "path = sys.argv[1]",
    "fmt = sys.argv[2]",
    "expected_frames = int(sys.argv[3])",
    "expected_width = int(sys.argv[4])",
    "expected_height = int(sys.argv[5])",
    "def apng_chunks(path):",
    "    with open(path, 'rb') as handle:",
    "        data = handle.read()",
    "    if not data.startswith(b'\\x89PNG\\r\\n\\x1a\\n'):",
    "        return {'is_png': False}",
    "    offset = 8",
    "    actl = None",
    "    fctl = 0",
    "    fdat = 0",
    "    while offset + 12 <= len(data):",
    "        length = struct.unpack('>I', data[offset:offset + 4])[0]",
    "        ctype = data[offset + 4:offset + 8]",
    "        payload = data[offset + 8:offset + 8 + length]",
    "        if ctype == b'acTL' and len(payload) >= 8:",
    "            actl = {'frame_count': struct.unpack('>I', payload[:4])[0], 'play_count': struct.unpack('>I', payload[4:8])[0]}",
    "        elif ctype == b'fcTL':",
    "            fctl += 1",
    "        elif ctype == b'fdAT':",
    "            fdat += 1",
    "        offset += 12 + length",
    "    return {'is_png': True, 'acTL': actl, 'fcTL_count': fctl, 'fdAT_count': fdat}",
    "im = Image.open(path)",
    "pillow_frame_count = getattr(im, 'n_frames', None) or sum(1 for _ in ImageSequence.Iterator(im))",
    "frame_count = pillow_frame_count",
    "chunk_info = None",
    "if fmt == 'apng':",
    "    chunk_info = apng_chunks(path)",
    "    actl = chunk_info.get('acTL') or {}",
    "    frame_count = actl.get('frame_count') or chunk_info.get('fcTL_count') or pillow_frame_count",
    "result = {'path': path, 'format': fmt, 'frame_count': frame_count, 'pillow_frame_count': pillow_frame_count, 'width': im.size[0], 'height': im.size[1], 'mode': im.mode}",
    "if chunk_info is not None: result['apng_chunks'] = chunk_info",
    "errors = []",
    "if expected_frames and frame_count != expected_frames: errors.append(f'expected {expected_frames} frames, got {frame_count}')",
    "if fmt == 'apng' and pillow_frame_count != expected_frames: errors.append(f'Pillow expected {expected_frames} APNG frames, got {pillow_frame_count}')",
    "if fmt == 'apng' and (not chunk_info or not chunk_info.get('acTL')): errors.append('APNG acTL chunk missing')",
    "if fmt == 'apng' and chunk_info and chunk_info.get('fcTL_count') != expected_frames: errors.append(f\"expected {expected_frames} fcTL chunks, got {chunk_info.get('fcTL_count')}\")",
    "if expected_width and im.size[0] != expected_width: errors.append(f'expected width {expected_width}, got {im.size[0]}')",
    "if expected_height and im.size[1] != expected_height: errors.append(f'expected height {expected_height}, got {im.size[1]}')",
    "if errors: result['errors'] = errors",
    "print(json.dumps(result))",
    "sys.exit(1 if errors else 0)",
  ].join("\n");
  const result = spawnSync(python, [
    "-c",
    code,
    filePath,
    format,
    String(expected.frameCount),
    String(expected.width),
    String(expected.height),
  ], { encoding: "utf8" });
  if (result.status !== 0) {
    throw new Error([
      `Pillow ${format.toUpperCase()} validation failed.`,
      result.stdout.trim(),
      result.stderr.trim(),
    ].filter(Boolean).join(" "));
  }
  return JSON.parse(result.stdout);
}

function parseArgs(argv) {
  const options = { ...DEFAULTS };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = () => {
      index += 1;
      if (index >= argv.length) {
        throw new Error(`Missing value for ${arg}`);
      }
      return argv[index];
    };
    switch (arg) {
      case "--url":
        options.url = next();
        break;
      case "--out-dir":
        options.outDir = path.resolve(next());
        break;
      case "--format":
        options.format = next();
        break;
      case "--views":
        options.views = next().split(",").map((value) => value.trim()).filter(Boolean);
        break;
      case "--palette":
        options.palette = next();
        break;
      case "--start":
        options.start = next();
        break;
      case "--loop":
        options.loop = next();
        break;
      case "--layer":
        options.layer = next();
        break;
      case "--material":
        options.material = next();
        break;
      case "--seconds":
        options.seconds = parsePositiveInteger(next(), "seconds");
        break;
      case "--fps":
        options.fps = parsePositiveInteger(next(), "fps");
        break;
      case "--width":
        options.width = parsePositiveInteger(next(), "width");
        break;
      case "--height":
        options.height = parsePositiveInteger(next(), "height");
        break;
      case "--step":
        options.step = parsePositiveInteger(next(), "step");
        break;
      case "--warmup":
        options.warmup = parseNonNegativeInteger(next(), "warmup");
        break;
      case "--density":
        options.density = parseNonNegativeInteger(next(), "density");
        break;
      case "--ready-timeout-ms":
        options.readyTimeoutMs = parsePositiveInteger(next(), "ready-timeout-ms");
        break;
      case "--export-timeout-ms":
        options.exportTimeoutMs = parsePositiveInteger(next(), "export-timeout-ms");
        break;
      case "--python":
        options.python = next();
        break;
      case "--report":
        options.report = next();
        break;
      default:
        throw new Error(`Unknown argument: ${arg}`);
    }
  }
  options.views = options.views.map((view) => view === "graph" ? "graph" : "surface");
  options.outDir = path.resolve(options.outDir);
  return options;
}

function parsePositiveInteger(value, label) {
  const number = Number(value);
  if (!Number.isFinite(number) || number < 1) {
    throw new Error(`${label} must be a positive integer`);
  }
  return Math.trunc(number);
}

function parseNonNegativeInteger(value, label) {
  const number = Number(value);
  if (!Number.isFinite(number) || number < 0) {
    throw new Error(`${label} must be a non-negative integer`);
  }
  return Math.trunc(number);
}

function sanitizeFilename(filename) {
  return filename.replace(/[^a-z0-9_.-]+/gi, "_");
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
