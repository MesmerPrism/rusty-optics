// Planarian 3D export controller for the surface-field browser adapter.
(function () {
  "use strict";

  const PLANARIAN_EXPORT_DEFAULTS = {
    format: "apng",
    view: "surface",
    palette: "neon-rgb",
    start: "reset",
    loop: "showcase",
    layer: "activity",
    material: "opaque",
    seconds: 8,
    fps: 12,
    width: 720,
    height: 860,
    stepsPerFrame: 1,
    warmupSteps: 0,
  };

  const PLANARIAN_EXPORT_FORMATS = new Set(["gif", "apng", "webm", "mp4"]);

  function createPlanarian3DExportController(dependencies) {
    const {
      clearPlanarian3DInteractionStateWithoutView,
      controls,
      encoderUrls,
      formatPlanarianSourceTargetAnchor,
      getState,
      isPlanarian3DMode,
      planarianGraphDensityInfo,
      planarianSourceTargetPolicy,
      planarianSourceTargetsFromAnchors,
      readPlanarian3DStats,
      scenarioInfo,
      setPlanarian3DStats,
      setPlaying,
      updatePlanarian3DView,
      viewport3d,
    } = dependencies;

    function setStatus(message) {
      setPlanarianGifStatus(controls, viewport3d, message);
    }

    function refreshPlanarian3DStats() {
      const stats = readPlanarian3DStats();
      setPlanarian3DStats(stats);
      return stats;
    }

    async function exportPlanarian3DGif() {
      const state = getState();
      const planarian3dRuntime = state.planarian3dRuntime;
      const planarian3dView = state.planarian3dView;
      let planarian3dStats = state.planarian3dStats;
      if (!isPlanarian3DMode() || !planarian3dRuntime || !planarian3dView) {
        setStatus("export unavailable");
        return;
      }
      const settings = readPlanarianGifSettings(controls);
      const frameCount = Math.max(1, settings.seconds * settings.fps);
      const originalPlaying = Boolean(state.playing);
      const originalVisibility = {
        body: controls.body.checked,
        nodes: controls.nodes.checked,
        edges: controls.edges.checked,
        tier2: controls.tier2.checked,
      };
      const originalPalette = planarian3dView.colorPalette || "neon-rgb";
      const originalMaterial = planarian3dView.bodyMaterialMode || "opaque";
      const originalPose = planarian3dView.getCameraPose?.() || null;
      const originalScenario = Math.trunc(
        planarian3dStats?.scenario_code ?? controls.planarianScenario.value ?? 3,
      );
      const frames = [];
      let captureState = null;
      controls.exportPlanarianGif.disabled = true;
      setPlaying(false);
      viewport3d.dataset.planarian3dExportStatus = "capturing";
      viewport3d.dataset.planarian3dExportError = "";
      viewport3d.dataset.planarian3dGifError = "";
      viewport3d.dataset.planarian3dExportPlannedFrameCount = String(frameCount);
      setStatus(`${settings.format} capture 0/${frameCount}`);

      try {
        if (settings.start === "reset") {
          planarian3dRuntime.reset_to_scenario(originalScenario);
          planarian3dStats = refreshPlanarian3DStats();
        }
        clearPlanarian3DInteractionStateWithoutView();
        planarian3dView.selectNode(null);
        planarian3dView.selectEdge(null);
        planarian3dView.updateEditHighlights([]);
        planarian3dView.setColorPalette(settings.palette);
        planarian3dView.setBodyMaterialMode(settings.material);
        applyPlanarianGifVisibility(controls, planarian3dView, settings.view);
        planarian3dView.setDefaultCameraPose?.();
        if (settings.warmupSteps > 0) {
          setStatus(`${settings.format} warm ${settings.warmupSteps}`);
          planarian3dRuntime.step(settings.warmupSteps);
          planarian3dStats = refreshPlanarian3DStats();
        }

        captureState = planarian3dView.beginFrameCapture({
          width: settings.width,
          height: settings.height,
        });
        const captureLayer = settings.layer === "current"
          ? controls.scalarLayer.value || "circuit.activity"
          : "circuit.activity";
        for (let frameIndex = 0; frameIndex < frameCount; frameIndex += 1) {
          if (frameIndex > 0) {
            planarian3dRuntime.step(settings.stepsPerFrame);
          }
          const activityValues = planarian3dRuntime.node_activity?.() || null;
          planarian3dView.updateSnapshot(
            planarian3dRuntime.snapshot(),
            planarian3dRuntime.conductance_values(),
            captureLayer,
            activityValues,
          );
          planarian3dStats = refreshPlanarian3DStats();
          frames.push(planarian3dView.captureFrame({
            width: settings.width,
            height: settings.height,
          }));
          if (frameIndex % 3 === 0 || frameIndex === frameCount - 1) {
            setStatus(`${settings.format} capture ${frameIndex + 1}/${frameCount}`);
            await nextAnimationFrame();
          }
        }
        planarian3dView.endFrameCapture(captureState);
        captureState = null;

        const exportFrames = shapePlanarianExportFrames(frames, settings);
        viewport3d.dataset.planarian3dExportStatus = "encoding";
        const exportResult = await encodePlanarianExportFrames(
          exportFrames,
          settings,
          encoderUrls,
          setStatus,
        );
        const filename = [
          "planarian",
          settings.format,
          settings.view,
          settings.palette,
          settings.loop,
          settings.start,
          captureLayer.replace(/^circuit\./, ""),
          `${planarianGraphDensityInfo(planarian3dStats?.graph_density_code).nodes}nodes`,
          `${settings.width}x${settings.height}`,
          `${settings.fps}fps`,
        ].join("-").replace(/[^a-z0-9_.-]+/gi, "_");
        downloadBlob(exportResult.blob, `${filename}.${exportResult.extension}`);
        writePlanarianExportMetadata({
          bytes: exportResult.blob.size,
          captureLayer,
          extension: exportResult.extension,
          filename: `${filename}.${exportResult.extension}`,
          formatPlanarianSourceTargetAnchor,
          frameCount: exportFrames.length,
          planarian3dStats,
          planarianGraphDensityInfo,
          planarianSourceTargetPolicy,
          planarianSourceTargetsFromAnchors,
          scenarioInfo,
          settings,
          viewport3d,
        });
        viewport3d.dataset.planarian3dExportBytes = String(exportResult.blob.size);
        viewport3d.dataset.planarian3dGifBytes = String(exportResult.blob.size);
        viewport3d.dataset.planarian3dExportStatus = "saved";
        setStatus(`${settings.format} saved ${(exportResult.blob.size / 1024 / 1024).toFixed(1)} MB`);
      } catch (error) {
        viewport3d.dataset.planarian3dExportStatus = "error";
        viewport3d.dataset.planarian3dExportError = error?.message || String(error);
        viewport3d.dataset.planarian3dGifError = error?.message || String(error);
        setStatus(`${settings.format} error`);
        console.warn(`Planarian export failed: ${error?.message || error}`);
      } finally {
        if (captureState) {
          planarian3dView.endFrameCapture(captureState);
        }
        controls.body.checked = originalVisibility.body;
        controls.nodes.checked = originalVisibility.nodes;
        controls.edges.checked = originalVisibility.edges;
        controls.tier2.checked = originalVisibility.tier2;
        planarian3dView.setColorPalette(originalPalette);
        planarian3dView.setBodyMaterialMode(originalMaterial);
        if (originalPose) {
          planarian3dView.setCameraPose(originalPose);
        }
        updatePlanarian3DView();
        controls.exportPlanarianGif.disabled = !isPlanarian3DMode();
        setPlaying(originalPlaying);
      }
    }

    return Object.freeze({
      exportPlanarian3DGif,
      readPlanarianGifSettings: () => readPlanarianGifSettings(controls),
      setPlanarianGifStatus: setStatus,
    });
  }

  function shapePlanarianExportFrames(frames, settings) {
    if (settings.loop !== "showcase" || frames.length < 8) {
      return frames;
    }
    const segment = selectPlanarianDynamicSegment(frames);
    const forwardCount = Math.max(4, Math.floor(frames.length / 2) + 1);
    const forwardFrames = resamplePlanarianFramesByVisualChange(segment, forwardCount);
    const shapedFrames = forwardFrames.slice();
    for (let index = forwardFrames.length - 2; shapedFrames.length < frames.length; index -= 1) {
      shapedFrames.push(forwardFrames[Math.max(1, index)]);
      if (index <= 1 && shapedFrames.length < frames.length) {
        index = forwardFrames.length - 1;
      }
    }
    return shapedFrames;
  }

  function selectPlanarianDynamicSegment(frames) {
    const deltas = planarianFrameDeltas(frames);
    const total = deltas.reduce((sum, delta) => sum + delta, 0);
    if (total <= 0) {
      return frames;
    }
    const minEndIndex = Math.min(frames.length - 1, Math.max(8, Math.floor(frames.length * 0.45)));
    const maxEndIndex = Math.min(frames.length - 1, Math.max(minEndIndex, Math.floor(frames.length * 0.85)));
    let running = 0;
    let endIndex = maxEndIndex;
    for (let index = 0; index < deltas.length; index += 1) {
      running += deltas[index];
      if (running >= total * 0.94) {
        endIndex = index + 1;
        break;
      }
    }
    endIndex = Math.max(minEndIndex, Math.min(maxEndIndex, endIndex));
    return frames.slice(0, endIndex + 1);
  }

  function resamplePlanarianFramesByVisualChange(frames, targetCount) {
    if (frames.length <= 1 || targetCount <= 1) {
      return frames.slice(0, targetCount);
    }
    const deltas = planarianFrameDeltas(frames);
    const cumulative = [0];
    for (const delta of deltas) {
      cumulative.push(cumulative[cumulative.length - 1] + delta);
    }
    const total = cumulative[cumulative.length - 1];
    if (total <= 0) {
      return evenlySamplePlanarianFrames(frames, targetCount);
    }
    const sampled = [];
    let sourceIndex = 0;
    for (let index = 0; index < targetCount; index += 1) {
      const target = total * index / Math.max(1, targetCount - 1);
      while (sourceIndex < cumulative.length - 1 && cumulative[sourceIndex] < target) {
        sourceIndex += 1;
      }
      const previousIndex = Math.max(0, sourceIndex - 1);
      const usePrevious = Math.abs(cumulative[previousIndex] - target)
        < Math.abs(cumulative[sourceIndex] - target);
      sampled.push(frames[usePrevious ? previousIndex : sourceIndex]);
    }
    return sampled;
  }

  function evenlySamplePlanarianFrames(frames, targetCount) {
    const sampled = [];
    for (let index = 0; index < targetCount; index += 1) {
      const sourceIndex = Math.round((frames.length - 1) * index / Math.max(1, targetCount - 1));
      sampled.push(frames[sourceIndex]);
    }
    return sampled;
  }

  function planarianFrameDeltas(frames) {
    const deltas = [];
    for (let index = 1; index < frames.length; index += 1) {
      deltas.push(planarianFrameDelta(frames[index - 1], frames[index]));
    }
    return deltas;
  }

  function planarianFrameDelta(previousFrame, nextFrame) {
    const previous = previousFrame.data;
    const next = nextFrame.data;
    const pixelStride = Math.max(16, Math.floor(previousFrame.width * previousFrame.height / 3600));
    let sum = 0;
    let count = 0;
    for (let offset = 0; offset < previous.length; offset += pixelStride * 4) {
      sum += Math.abs(previous[offset] - next[offset])
        + Math.abs(previous[offset + 1] - next[offset + 1])
        + Math.abs(previous[offset + 2] - next[offset + 2]);
      count += 3;
    }
    return count > 0 ? sum / count : 0;
  }

  async function encodePlanarianExportFrames(frames, settings, encoderUrls, setStatus) {
    if (settings.format === "gif") {
      setStatus("gif palette");
      const { encodeGifAsync, buildAdaptivePalette } = await import(encoderUrls.gif);
      const palette = buildAdaptivePalette(frames, { background: [12, 15, 20] });
      setStatus(`gif encoding 0/${frames.length}`);
      return {
        blob: await encodeGifAsync(frames, {
          width: settings.width,
          height: settings.height,
          delayCs: Math.round(100 / settings.fps),
          palette,
          dither: true,
          onProgress: (encoded, total) => setStatus(`gif encoding ${encoded}/${total}`),
        }),
        extension: "gif",
      };
    }
    if (settings.format === "apng") {
      const { encodeApngAsync } = await import(encoderUrls.apng);
      setStatus(`apng encoding 0/${frames.length}`);
      return {
        blob: await encodeApngAsync(frames, {
          width: settings.width,
          height: settings.height,
          fps: settings.fps,
          onProgress: (encoded, total) => setStatus(`apng encoding ${encoded}/${total}`),
        }),
        extension: "png",
      };
    }
    const { encodeVideoAsync } = await import(encoderUrls.video);
    setStatus(`${settings.format} recording 0/${frames.length}`);
    const video = await encodeVideoAsync(frames, {
      width: settings.width,
      height: settings.height,
      fps: settings.fps,
      format: settings.format,
      onProgress: (encoded, total) => setStatus(`${settings.format} recording ${encoded}/${total}`),
    });
    return {
      blob: video.blob,
      extension: video.extension,
    };
  }

  function readPlanarianGifSettings(controls) {
    const format = controls.planarianExportFormat?.value || PLANARIAN_EXPORT_DEFAULTS.format;
    const loop = controls.planarianExportLoop?.value === "forward" ? "forward" : PLANARIAN_EXPORT_DEFAULTS.loop;
    return {
      format: PLANARIAN_EXPORT_FORMATS.has(format) ? format : PLANARIAN_EXPORT_DEFAULTS.format,
      view: controls.planarianGifView?.value === "graph" ? "graph" : PLANARIAN_EXPORT_DEFAULTS.view,
      palette: controls.planarianGifPalette?.value === "teaching" ? "teaching" : PLANARIAN_EXPORT_DEFAULTS.palette,
      start: controls.planarianExportStart?.value === "current" ? "current" : PLANARIAN_EXPORT_DEFAULTS.start,
      loop,
      layer: controls.planarianExportLayer?.value === "current" ? "current" : PLANARIAN_EXPORT_DEFAULTS.layer,
      material: controls.planarianExportMaterial?.value === "boosted" ? "boosted" : PLANARIAN_EXPORT_DEFAULTS.material,
      seconds: readBoundedIntegerControl(controls.planarianGifSeconds, PLANARIAN_EXPORT_DEFAULTS.seconds, 1, 20),
      fps: readBoundedIntegerControl(controls.planarianGifFps, PLANARIAN_EXPORT_DEFAULTS.fps, 4, 24),
      width: readBoundedIntegerControl(controls.planarianGifWidth, PLANARIAN_EXPORT_DEFAULTS.width, 320, 1280),
      height: readBoundedIntegerControl(controls.planarianGifHeight, PLANARIAN_EXPORT_DEFAULTS.height, 240, 1440),
      stepsPerFrame: readBoundedIntegerControl(controls.planarianGifSteps, PLANARIAN_EXPORT_DEFAULTS.stepsPerFrame, 1, 8),
      warmupSteps: readBoundedIntegerControl(controls.planarianExportWarmup, PLANARIAN_EXPORT_DEFAULTS.warmupSteps, 0, 240),
    };
  }

  function readBoundedIntegerControl(control, fallback, min, max) {
    const value = Math.trunc(Number(control?.value));
    if (!Number.isFinite(value)) {
      return fallback;
    }
    return Math.max(min, Math.min(max, value));
  }

  function applyPlanarianGifVisibility(controls, planarian3dView, view) {
    const showGraph = view === "graph";
    controls.body.checked = !showGraph;
    controls.nodes.checked = showGraph;
    controls.edges.checked = showGraph;
    controls.tier2.checked = showGraph;
    planarian3dView.setVisibility(
      controls.edges.checked,
      controls.tier2.checked,
      controls.body.checked,
      controls.nodes.checked,
    );
  }

  function setPlanarianGifStatus(controls, viewport3d, message) {
    if (controls.planarianGifStatus) {
      controls.planarianGifStatus.textContent = message;
    }
    if (viewport3d) {
      viewport3d.dataset.planarian3dExportStatusText = message;
    }
  }

  function writePlanarianExportMetadata({
    bytes,
    captureLayer,
    extension,
    filename,
    formatPlanarianSourceTargetAnchor,
    frameCount,
    planarian3dStats,
    planarianGraphDensityInfo,
    planarianSourceTargetPolicy,
    planarianSourceTargetsFromAnchors,
    scenarioInfo,
    settings,
    viewport3d,
  }) {
    const scenario = scenarioInfo(planarian3dStats?.scenario_code);
    const density = planarianGraphDensityInfo(planarian3dStats?.graph_density_code);
    const literatureAnchors = planarian3dStats?.literature_anchors || [];
    const sourceTargets = planarianSourceTargetsFromAnchors(literatureAnchors);
    const metadata = {
      format: settings.format,
      extension,
      view: settings.view,
      palette: settings.palette,
      start: settings.start,
      loop: settings.loop,
      layer: captureLayer,
      material: settings.material,
      frame_count: frameCount,
      width: settings.width,
      height: settings.height,
      fps: settings.fps,
      duration_seconds: settings.seconds,
      steps_per_frame: settings.stepsPerFrame,
      warmup_steps: settings.warmupSteps,
      filename,
      bytes,
      scenario: scenario.label,
      graph_density: density.label,
      graph_nodes: density.nodes,
      dynamics: planarianExportDynamicsDescription(settings.loop),
      evidence_type: planarian3dStats?.evidence_type || "",
      expected_outcome: planarian3dStats?.expected_outcome || scenario.outcome,
      voltage_unit: planarian3dStats?.voltage_unit_label || "normalized",
      voltage_unit_policy: planarian3dStats?.voltage_unit_policy || "",
      literature_anchors: literatureAnchors,
      source_targets: sourceTargets,
      source_target_summary: sourceTargets.map(formatPlanarianSourceTargetAnchor).join("; "),
      source_target_policy: planarianSourceTargetPolicy(sourceTargets),
    };
    viewport3d.dataset.planarian3dExportMetadata = JSON.stringify(metadata);
    viewport3d.dataset.planarian3dExportFormat = metadata.format;
    viewport3d.dataset.planarian3dExportView = metadata.view;
    viewport3d.dataset.planarian3dExportFrameCount = String(metadata.frame_count);
    viewport3d.dataset.planarian3dExportWidth = String(metadata.width);
    viewport3d.dataset.planarian3dExportHeight = String(metadata.height);
    viewport3d.dataset.planarian3dExportFps = String(metadata.fps);
    viewport3d.dataset.planarian3dExportLoop = metadata.loop;
    viewport3d.dataset.planarian3dExportDynamics = metadata.dynamics;
    viewport3d.dataset.planarian3dExportSourceTargetPolicy = metadata.source_target_policy;
    viewport3d.dataset.planarian3dExportFilename = metadata.filename;
  }

  function planarianExportDynamicsDescription(loop) {
    if (loop === "showcase") {
      return [
        "Matter-owned synthetic planarian memory scenario frames",
        "Optics-selected active segment resampled and mirrored for a seamless visual loop",
        "no added or recalibrated physiology dynamics",
        "not a PlanformDB-derived predictor",
      ].join("; ");
    }
    return [
      "Matter-owned synthetic planarian circuit stepped forward at fixed display cadence",
      "Optics applies color, camera, material, and export encoding only",
      "not a PlanformDB-derived predictor",
    ].join("; ");
  }

  function downloadBlob(blob, filename) {
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = filename;
    document.body.append(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 2000);
  }

  function nextAnimationFrame() {
    return new Promise((resolve) => window.requestAnimationFrame(resolve));
  }

  window.RustyOpticsPlanarian3DExport = Object.freeze({
    PLANARIAN_EXPORT_DEFAULTS,
    createPlanarian3DExportController,
    evenlySamplePlanarianFrames,
    planarianExportDynamicsDescription,
    planarianFrameDelta,
    planarianFrameDeltas,
    resamplePlanarianFramesByVisualChange,
    selectPlanarianDynamicSegment,
    shapePlanarianExportFrames,
  });
})();
