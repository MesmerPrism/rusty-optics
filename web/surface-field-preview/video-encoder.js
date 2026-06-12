export async function encodeVideoAsync(frames, options = {}) {
  if (!Array.isArray(frames) || frames.length === 0) {
    throw new Error("Video export requires at least one frame");
  }
  if (typeof MediaRecorder !== "function") {
    throw new Error("Video export is not available in this browser");
  }
  const width = Math.max(1, Math.trunc(options.width || frames[0].width));
  const height = Math.max(1, Math.trunc(options.height || frames[0].height));
  const fps = Math.max(1, Math.trunc(options.fps || 12));
  const format = options.format === "mp4" ? "mp4" : "webm";
  const mimeType = selectVideoMimeType(format);
  if (!mimeType) {
    throw new Error(`${format.toUpperCase()} export is not supported by this browser`);
  }

  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  canvas.style.cssText = "position:fixed;left:-10000px;top:-10000px;width:1px;height:1px;opacity:0;pointer-events:none";
  document.body.append(canvas);
  const context = canvas.getContext("2d", { willReadFrequently: true });
  context.putImageData(frames[0], 0, 0);
  const stream = canvas.captureStream(fps);
  const recorder = new MediaRecorder(stream, {
    mimeType,
    videoBitsPerSecond: Math.max(1000000, Math.trunc(options.videoBitsPerSecond || width * height * fps * 0.22)),
  });
  const chunks = [];
  recorder.addEventListener("dataavailable", (event) => {
    if (event.data?.size) {
      chunks.push(event.data);
    }
  });
  const finished = new Promise((resolve, reject) => {
    recorder.addEventListener("stop", resolve, { once: true });
    recorder.addEventListener("error", () => reject(recorder.error), { once: true });
  });

  try {
    const track = stream.getVideoTracks()[0];
    recorder.start(250);
    await wait(80);
    const frameDelayMs = Math.max(1, 1000 / fps);
    for (let frameIndex = 0; frameIndex < frames.length; frameIndex += 1) {
      const frame = frames[frameIndex];
      validateRgbaFrame(frame, width, height, format.toUpperCase());
      context.putImageData(frame, 0, 0);
      track?.requestFrame?.();
      options.onProgress?.(frameIndex + 1, frames.length);
      await wait(frameDelayMs);
    }
    recorder.requestData();
    await wait(80);
    recorder.stop();
    await finished;
    const blob = new Blob(chunks, { type: mimeType });
    if (blob.size < 1024) {
      throw new Error(`${format.toUpperCase()} recorder produced no video frames`);
    }
    return {
      blob,
      mimeType,
      extension: mimeType.includes("mp4") ? "mp4" : "webm",
    };
  } finally {
    for (const videoTrack of stream.getTracks()) {
      videoTrack.stop();
    }
    canvas.remove();
  }
}

function validateRgbaFrame(frame, width, height, format) {
  if (!frame || frame.width !== width || frame.height !== height) {
    throw new Error(`${format} frame dimensions must be constant`);
  }
  if (!frame.data || frame.data.length !== width * height * 4) {
    throw new Error(`${format} frame data must be RGBA for ${width}x${height}`);
  }
}

function selectVideoMimeType(format) {
  const candidates = format === "mp4"
    ? [
      "video/mp4;codecs=avc1.42E01E",
      "video/mp4;codecs=h264",
      "video/mp4",
    ]
    : [
      "video/webm;codecs=vp9",
      "video/webm;codecs=vp8",
      "video/webm",
    ];
  return candidates.find((candidate) => MediaRecorder.isTypeSupported(candidate)) || "";
}

function wait(ms) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}
