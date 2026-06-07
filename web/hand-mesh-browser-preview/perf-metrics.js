window.HandMeshPerfMetrics = (() => {
  function create({ output, isEnabled, snapshot }) {
    const state = {
      average: {},
      last: null,
      lastPanelUpdateMs: 0,
      frameCount: 0,
    };

    function beginFrame(simDeltaSeconds, realDeltaMs) {
      const current = readSnapshot(snapshot);
      return {
        startedAtMs: nowMs(),
        simDeltaMs: simDeltaSeconds * 1000,
        realDeltaMs,
        particleCount: current.particleCount,
        runtimeFrameIndex: current.runtimeFrameIndex,
        playbackAdvancedFrames: 0,
        particleTriangleChecks: 0,
        runtimeSdfTriangleChecks: 0,
        drawLineSegments: 0,
        drawPoints: 0,
        drawParticleMarkers: 0,
        drawTrailSegments: 0,
        trailsEnabled: current.trailsEnabled,
        particlesEnabled: current.particlesEnabled,
        sdfEnabled: current.sdfEnabled,
        meshEnabled: current.meshEnabled,
        colliderEnabled: current.colliderEnabled,
        coordinatesEnabled: current.coordinatesEnabled,
      };
    }

    function measure(metrics, key, callback) {
      if (!metrics) {
        return callback();
      }
      const startedAt = nowMs();
      const result = callback();
      metrics[key] = (metrics[key] || 0) + nowMs() - startedAt;
      return result;
    }

    function finish(metrics) {
      if (!metrics) {
        return;
      }
      metrics.totalLoopMs = nowMs() - metrics.startedAtMs;
      metrics.visualizationMs = (metrics.drawMs || 0) + (metrics.runtimeFrameBuildMs || 0);
      metrics.simulationMs = metrics.simulationMs || 0;
      metrics.loopFps = metrics.totalLoopMs > 0 ? 1000 / metrics.totalLoopMs : 0;
      metrics.rafFps = metrics.realDeltaMs > 0 ? 1000 / metrics.realDeltaMs : 0;
      metrics.simRate = metrics.realDeltaMs > 0 ? metrics.simDeltaMs / metrics.realDeltaMs : 0;
      metrics.particleCount = readSnapshot(snapshot).particleCount;
      state.last = metrics;
      state.frameCount += 1;
      for (const [key, value] of Object.entries(metrics)) {
        if (typeof value !== "number" || !Number.isFinite(value) || key === "startedAtMs") {
          continue;
        }
        state.average[key] = smoothAverage(state.average[key], value);
      }
      updatePanel(metrics, false);
    }

    function update(force = false) {
      updatePanel(state.last, force);
    }

    function updatePanel(metrics, force) {
      if (!output) {
        return;
      }
      output.hidden = !isEnabled();
      if (output.hidden) {
        return;
      }
      const timestamp = nowMs();
      if (!force && timestamp - state.lastPanelUpdateMs < 350) {
        return;
      }
      state.lastPanelUpdateMs = timestamp;
      if (!metrics) {
        output.textContent = "Waiting for realtime metrics";
        return;
      }
      const current = readSnapshot(snapshot);
      const avg = state.average;
      const totalChecks = (metrics.particleTriangleChecks || 0) + (metrics.runtimeSdfTriangleChecks || 0);
      const avgTotalChecks = (avg.particleTriangleChecks || 0) + (avg.runtimeSdfTriangleChecks || 0);
      output.textContent = [
        `Perf loop ${formatMs(avg.totalLoopMs)} avg (${formatFps(fpsFromMs(avg.totalLoopMs))} fps) / ${formatMs(metrics.totalLoopMs)} last (${formatFps(fpsFromMs(metrics.totalLoopMs))} fps)`,
        `browser interval ${formatMs(avg.realDeltaMs)} avg (${formatFps(fpsFromMs(avg.realDeltaMs))} fps) / ${formatMs(metrics.realDeltaMs)} last (${formatFps(fpsFromMs(metrics.realDeltaMs))} fps)`,
        `simulation dt ${formatMs(avg.simDeltaMs)} avg / ${formatMs(metrics.simDeltaMs)} last  ${formatRate(avg.simRate)} realtime`,
        `main cost: ${dominantCost(avg)}`,
        `simulation: ${formatMs(avg.simulationMs)} avg / ${formatMs(metrics.simulationMs)} last`,
        `visualization: ${formatMs(avg.visualizationMs)} avg / ${formatMs(metrics.visualizationMs)} last`,
        `  draw: ${formatMs(avg.drawMs)} avg / ${formatMs(metrics.drawMs)} last`,
        `  mesh+SDF rebuild: ${formatMs(avg.runtimeFrameBuildMs)} avg / ${formatMs(metrics.runtimeFrameBuildMs)} last`,
        `particle step/run: ${formatMs(avg.particleStepMs)} avg / ${formatMs(metrics.particleStepMs)} last  substeps ${metrics.particleSubsteps || 0}`,
        `SDF slice/rebuild: ${formatMs(avg.runtimeSdfSliceMs)} avg / ${formatMs(metrics.runtimeSdfSliceMs)} last  cells ${metrics.runtimeSdfCells || 0}`,
        `playback advance: ${formatCount(avg.playbackAdvancedFrames || 0)} avg / ${formatCount(metrics.playbackAdvancedFrames || 0)} last`,
        `draw sections: sdf ${formatMs(avg.drawSdfMs)} mesh ${formatMs(avg.drawMeshMs)} collider ${formatMs(avg.drawColliderMs)} coords ${formatMs(avg.drawCoordinatesMs)} particles ${formatMs(avg.drawParticlesMs)}`,
        `checks/frame: ${formatCount(avgTotalChecks)} avg / ${formatCount(totalChecks)} last`,
        `  particles ${formatCount(metrics.particleTriangleChecks || 0)}  SDF ${formatCount(metrics.runtimeSdfTriangleChecks || 0)}  triangles ${metrics.runtimeTriangleCount || current.triangleCount || 0}`,
        `draw counts: lines ${formatCount(metrics.drawLineSegments || 0)} points ${formatCount(metrics.drawPoints || 0)} particle marks ${formatCount(metrics.drawParticleMarkers || 0)} trail segs ${formatCount(metrics.drawTrailSegments || 0)}`,
        `state: ${current.particleCount} particles, trails ${current.trailsEnabled ? "on" : "off"}, playback ${current.playbackPaused ? "paused" : "running"}`,
      ].join("\n");
    }

    return { state, beginFrame, measure, finish, update };
  }

  function readSnapshot(snapshot) {
    if (typeof snapshot !== "function") {
      return {};
    }
    return snapshot() || {};
  }

  function smoothAverage(previous, value) {
    if (!Number.isFinite(previous)) {
      return value;
    }
    return previous * 0.86 + value * 0.14;
  }

  function dominantCost(avg) {
    const candidates = [
      ["particle simulation", avg.simulationMs || 0],
      ["canvas drawing", avg.drawMs || 0],
      ["runtime mesh/SDF rebuild", avg.runtimeFrameBuildMs || 0],
    ].sort((left, right) => right[1] - left[1]);
    const [label, value] = candidates[0];
    return `${label} (${formatMs(value)} avg)`;
  }

  function formatMs(value) {
    if (!Number.isFinite(value)) {
      return "0.0ms";
    }
    return `${value.toFixed(value >= 10 ? 1 : 2)}ms`;
  }

  function formatFps(value) {
    if (!Number.isFinite(value)) {
      return "0.0";
    }
    return value.toFixed(value >= 10 ? 1 : 2);
  }

  function fpsFromMs(value) {
    if (!Number.isFinite(value) || value <= 0) {
      return 0;
    }
    return 1000 / value;
  }

  function formatRate(value) {
    if (!Number.isFinite(value)) {
      return "0.00x";
    }
    return `${value.toFixed(2)}x`;
  }

  function formatCount(value) {
    if (!Number.isFinite(value)) {
      return "0";
    }
    if (Math.abs(value) >= 1_000_000) {
      return `${(value / 1_000_000).toFixed(2)}M`;
    }
    if (Math.abs(value) >= 1_000) {
      return `${(value / 1_000).toFixed(1)}k`;
    }
    return `${Math.round(value)}`;
  }

  function nowMs() {
    return performance.now();
  }

  return { create };
})();
