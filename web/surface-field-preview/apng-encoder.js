const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
const CRC32_TABLE = buildCrc32Table();

export async function encodeApngAsync(frames, options = {}) {
  if (!Array.isArray(frames) || frames.length === 0) {
    throw new Error("APNG export requires at least one frame");
  }
  const width = Math.max(1, Math.trunc(options.width || frames[0].width));
  const height = Math.max(1, Math.trunc(options.height || frames[0].height));
  const fps = Math.max(1, Math.trunc(options.fps || 12));
  const bytes = [...PNG_SIGNATURE];
  writeChunk(bytes, "IHDR", makeIhdr(width, height));
  writeChunk(bytes, "acTL", makeAnimationControl(frames.length, 0));

  let sequence = 0;
  for (let frameIndex = 0; frameIndex < frames.length; frameIndex += 1) {
    const frame = frames[frameIndex];
    validateRgbaFrame(frame, width, height, "APNG");
    writeChunk(bytes, "fcTL", makeFrameControl(sequence, width, height, fps));
    sequence += 1;

    const scanlines = rgbaToPngScanlines(frame.data, width, height);
    const compressed = await zlibCompress(scanlines);
    if (frameIndex === 0) {
      writeChunk(bytes, "IDAT", compressed);
    } else {
      const payload = new Uint8Array(compressed.length + 4);
      writeU32Array(payload, 0, sequence);
      payload.set(compressed, 4);
      writeChunk(bytes, "fdAT", payload);
      sequence += 1;
    }
    options.onProgress?.(frameIndex + 1, frames.length);
    await yieldToBrowser();
  }

  writeChunk(bytes, "IEND", []);
  return new Blob([new Uint8Array(bytes)], { type: "image/png" });
}

function validateRgbaFrame(frame, width, height, format) {
  if (!frame || frame.width !== width || frame.height !== height) {
    throw new Error(`${format} frame dimensions must be constant`);
  }
  if (!frame.data || frame.data.length !== width * height * 4) {
    throw new Error(`${format} frame data must be RGBA for ${width}x${height}`);
  }
}

function makeIhdr(width, height) {
  const data = new Uint8Array(13);
  writeU32Array(data, 0, width);
  writeU32Array(data, 4, height);
  data[8] = 8;
  data[9] = 6;
  data[10] = 0;
  data[11] = 0;
  data[12] = 0;
  return data;
}

function makeAnimationControl(frameCount, playCount) {
  const data = new Uint8Array(8);
  writeU32Array(data, 0, frameCount);
  writeU32Array(data, 4, playCount);
  return data;
}

function makeFrameControl(sequence, width, height, fps) {
  const data = new Uint8Array(26);
  writeU32Array(data, 0, sequence);
  writeU32Array(data, 4, width);
  writeU32Array(data, 8, height);
  writeU32Array(data, 12, 0);
  writeU32Array(data, 16, 0);
  writeU16Array(data, 20, 1);
  writeU16Array(data, 22, fps);
  data[24] = 0;
  data[25] = 0;
  return data;
}

function rgbaToPngScanlines(rgba, width, height) {
  const stride = width * 4;
  const scanlines = new Uint8Array((stride + 1) * height);
  for (let y = 0; y < height; y += 1) {
    const sourceOffset = y * stride;
    const targetOffset = y * (stride + 1);
    scanlines[targetOffset] = 0;
    scanlines.set(rgba.subarray(sourceOffset, sourceOffset + stride), targetOffset + 1);
  }
  return scanlines;
}

async function zlibCompress(bytes) {
  if (typeof CompressionStream === "function") {
    const stream = new Blob([bytes]).stream().pipeThrough(new CompressionStream("deflate"));
    return new Uint8Array(await new Response(stream).arrayBuffer());
  }
  return zlibStore(bytes);
}

function zlibStore(bytes) {
  const output = [0x78, 0x01];
  let offset = 0;
  while (offset < bytes.length) {
    const remaining = bytes.length - offset;
    const length = Math.min(65535, remaining);
    output.push(offset + length >= bytes.length ? 0x01 : 0x00);
    output.push(length & 0xff, (length >> 8) & 0xff);
    const inverted = length ^ 0xffff;
    output.push(inverted & 0xff, (inverted >> 8) & 0xff);
    for (let index = 0; index < length; index += 1) {
      output.push(bytes[offset + index]);
    }
    offset += length;
  }
  writeU32(output, adler32(bytes));
  return new Uint8Array(output);
}

function adler32(bytes) {
  let a = 1;
  let b = 0;
  for (const value of bytes) {
    a = (a + value) % 65521;
    b = (b + a) % 65521;
  }
  return ((b << 16) | a) >>> 0;
}

function writeChunk(bytes, type, data) {
  const payload = data instanceof Uint8Array ? data : new Uint8Array(data);
  writeU32(bytes, payload.length);
  const typeBytes = asciiBytes(type);
  writeBytes(bytes, typeBytes);
  writeBytes(bytes, payload);
  writeU32(bytes, crc32(typeBytes, payload));
}

function crc32(typeBytes, payload) {
  let crc = 0xffffffff;
  for (const value of typeBytes) {
    crc = CRC32_TABLE[(crc ^ value) & 0xff] ^ (crc >>> 8);
  }
  for (const value of payload) {
    crc = CRC32_TABLE[(crc ^ value) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function buildCrc32Table() {
  const table = new Uint32Array(256);
  for (let index = 0; index < 256; index += 1) {
    let value = index;
    for (let bit = 0; bit < 8; bit += 1) {
      value = (value & 1) ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
    }
    table[index] = value >>> 0;
  }
  return table;
}

function writeU16Array(bytes, offset, value) {
  bytes[offset] = (value >>> 8) & 0xff;
  bytes[offset + 1] = value & 0xff;
}

function writeU32Array(bytes, offset, value) {
  bytes[offset] = (value >>> 24) & 0xff;
  bytes[offset + 1] = (value >>> 16) & 0xff;
  bytes[offset + 2] = (value >>> 8) & 0xff;
  bytes[offset + 3] = value & 0xff;
}

function writeU32(bytes, value) {
  bytes.push((value >>> 24) & 0xff, (value >>> 16) & 0xff, (value >>> 8) & 0xff, value & 0xff);
}

function asciiBytes(value) {
  return [...value].map((char) => char.charCodeAt(0));
}

function writeBytes(bytes, values) {
  for (const value of values) {
    bytes.push(value);
  }
}

function yieldToBrowser() {
  return new Promise((resolve) => globalThis.setTimeout(resolve, 0));
}
