const GIF_SIGNATURE = "GIF89a";
const NETSCAPE_LOOP_EXTENSION = [0x21, 0xff, 0x0b, ...asciiBytes("NETSCAPE2.0"), 0x03, 0x01, 0x00, 0x00, 0x00];

export function encodeGif(frames, options = {}) {
  const context = createGifContext(frames, options);
  for (const frame of frames) {
    writeGifFrame(context, frame);
  }
  return finishGif(context);
}

export async function encodeGifAsync(frames, options = {}) {
  const context = createGifContext(frames, options);
  for (let frameIndex = 0; frameIndex < frames.length; frameIndex += 1) {
    writeGifFrame(context, frames[frameIndex]);
    options.onProgress?.(frameIndex + 1, frames.length);
    await yieldToBrowser();
  }
  return finishGif(context);
}

function createGifContext(frames, options = {}) {
  if (!Array.isArray(frames) || frames.length === 0) {
    throw new Error("GIF export requires at least one frame");
  }
  const width = Math.max(1, Math.trunc(options.width || frames[0].width));
  const height = Math.max(1, Math.trunc(options.height || frames[0].height));
  const delayCs = Math.max(2, Math.trunc(options.delayCs || 8));
  const palette = options.palette || buildRgbCubePalette();
  if (palette.length !== 256 * 3) {
    throw new Error("GIF export palette must contain 256 RGB colors");
  }

  const bytes = [];
  writeAscii(bytes, GIF_SIGNATURE);
  writeU16(bytes, width);
  writeU16(bytes, height);
  bytes.push(0xf7, 0x00, 0x00);
  writeBytes(bytes, palette);
  writeBytes(bytes, NETSCAPE_LOOP_EXTENSION);
  return {
    bytes,
    width,
    height,
    delayCs,
    palette,
    dither: options.dither !== false,
  };
}

function writeGifFrame(context, frame) {
  const { bytes, width, height, delayCs, palette, dither } = context;
  validateRgbaFrame(frame, width, height, "GIF");
  const indexed = rgbaToIndexed(frame.data, palette, { width, height, dither });
  bytes.push(0x21, 0xf9, 0x04, 0x04);
  writeU16(bytes, delayCs);
  bytes.push(0x00, 0x00);
  bytes.push(0x2c);
  writeU16(bytes, 0);
  writeU16(bytes, 0);
  writeU16(bytes, width);
  writeU16(bytes, height);
  bytes.push(0x00);
  bytes.push(0x08);
  writeSubBlocks(bytes, lzwEncodeIndexed(indexed, 8));
}

function finishGif(context) {
  const { bytes } = context;
  bytes.push(0x3b);
  return new Blob([new Uint8Array(bytes)], { type: "image/gif" });
}

function validateRgbaFrame(frame, width, height, format) {
  if (!frame || frame.width !== width || frame.height !== height) {
    throw new Error(`${format} frame dimensions must be constant`);
  }
  if (!frame.data || frame.data.length !== width * height * 4) {
    throw new Error(`${format} frame data must be RGBA for ${width}x${height}`);
  }
}

export function buildRgbCubePalette() {
  const bytes = [];
  const levels = [0, 51, 102, 153, 204, 255];
  for (const red of levels) {
    for (const green of levels) {
      for (const blue of levels) {
        bytes.push(red, green, blue);
      }
    }
  }
  for (let index = 0; index < 40; index += 1) {
    const value = Math.round((index / 39) * 255);
    bytes.push(value, value, value);
  }
  return bytes;
}

export function buildAdaptivePalette(frames, options = {}) {
  const maxColors = Math.max(2, Math.min(256, Math.trunc(options.maxColors || 256)));
  const background = Array.isArray(options.background)
    ? options.background.slice(0, 3).map((value) => Math.max(0, Math.min(255, Math.round(value))))
    : [12, 15, 20];
  const totalPixels = frames.reduce((total, frame) => total + Math.floor(frame.data.length / 4), 0);
  const sampleLimit = Math.max(200000, Math.trunc(options.sampleLimit || 1800000));
  const sampleStride = Math.max(1, Math.floor(totalPixels / sampleLimit));
  const histogram = new Map();
  addHistogramColor(histogram, background[0], background[1], background[2], 24000);

  let globalPixel = 0;
  for (const frame of frames) {
    const rgba = frame.data;
    for (let offset = 0; offset < rgba.length; offset += 4, globalPixel += 1) {
      if (globalPixel % sampleStride !== 0) {
        continue;
      }
      const alpha = rgba[offset + 3];
      if (alpha < 12) {
        addHistogramColor(histogram, background[0], background[1], background[2], 1);
        continue;
      }
      addHistogramColor(histogram, rgba[offset], rgba[offset + 1], rgba[offset + 2], 1);
    }
  }

  const reservedKey = colorKey(background[0], background[1], background[2]);
  const colors = [...histogram.entries()]
    .filter(([key]) => key !== reservedKey)
    .map(([, color]) => ({
      r: color.r / color.count,
      g: color.g / color.count,
      b: color.b / color.count,
      count: color.count,
    }));
  const boxes = medianCutColorBoxes(colors, maxColors - 1);
  const palette = [background[0], background[1], background[2]];
  for (const box of boxes) {
    const color = averageBoxColor(box);
    palette.push(color.r, color.g, color.b);
  }
  while (palette.length < 256 * 3) {
    const t = (palette.length / 3) / 255;
    const value = Math.round(t * 255);
    palette.push(value, value, value);
  }
  return palette.slice(0, 256 * 3);
}

function addHistogramColor(histogram, red, green, blue, weight) {
  const key = colorKey(red, green, blue);
  const current = histogram.get(key) || { r: 0, g: 0, b: 0, count: 0 };
  current.r += red * weight;
  current.g += green * weight;
  current.b += blue * weight;
  current.count += weight;
  histogram.set(key, current);
}

function colorKey(red, green, blue) {
  return `${red >> 3},${green >> 3},${blue >> 3}`;
}

function medianCutColorBoxes(colors, maxBoxes) {
  if (colors.length === 0) {
    return [];
  }
  const boxes = [colors];
  while (boxes.length < maxBoxes) {
    let splitIndex = -1;
    let splitScore = -1;
    for (let index = 0; index < boxes.length; index += 1) {
      const box = boxes[index];
      if (box.length < 2) {
        continue;
      }
      const bounds = boxBounds(box);
      const range = Math.max(bounds.rMax - bounds.rMin, bounds.gMax - bounds.gMin, bounds.bMax - bounds.bMin);
      const score = range * bounds.count;
      if (score > splitScore) {
        splitIndex = index;
        splitScore = score;
      }
    }
    if (splitIndex < 0) {
      break;
    }
    const box = boxes.splice(splitIndex, 1)[0];
    const bounds = boxBounds(box);
    const channel = largestRangeChannel(bounds);
    box.sort((a, b) => a[channel] - b[channel]);
    const halfWeight = bounds.count / 2;
    let weight = 0;
    let cut = 1;
    for (; cut < box.length - 1; cut += 1) {
      weight += box[cut].count;
      if (weight >= halfWeight) {
        break;
      }
    }
    boxes.push(box.slice(0, cut + 1), box.slice(cut + 1));
  }
  return boxes;
}

function boxBounds(box) {
  const bounds = {
    rMin: 255, rMax: 0,
    gMin: 255, gMax: 0,
    bMin: 255, bMax: 0,
    count: 0,
  };
  for (const color of box) {
    bounds.rMin = Math.min(bounds.rMin, color.r);
    bounds.rMax = Math.max(bounds.rMax, color.r);
    bounds.gMin = Math.min(bounds.gMin, color.g);
    bounds.gMax = Math.max(bounds.gMax, color.g);
    bounds.bMin = Math.min(bounds.bMin, color.b);
    bounds.bMax = Math.max(bounds.bMax, color.b);
    bounds.count += color.count;
  }
  return bounds;
}

function largestRangeChannel(bounds) {
  const r = bounds.rMax - bounds.rMin;
  const g = bounds.gMax - bounds.gMin;
  const b = bounds.bMax - bounds.bMin;
  if (r >= g && r >= b) {
    return "r";
  }
  return g >= b ? "g" : "b";
}

function averageBoxColor(box) {
  let r = 0;
  let g = 0;
  let b = 0;
  let count = 0;
  for (const color of box) {
    r += color.r * color.count;
    g += color.g * color.count;
    b += color.b * color.count;
    count += color.count;
  }
  return {
    r: Math.round(r / Math.max(1, count)),
    g: Math.round(g / Math.max(1, count)),
    b: Math.round(b / Math.max(1, count)),
  };
}

function rgbaToIndexed(rgba, palette, options = {}) {
  if (options.dither) {
    return rgbaToIndexedDithered(rgba, palette, options.width, options.height);
  }
  const indexed = new Uint8Array(rgba.length / 4);
  const lookup = new Map();
  for (let pixel = 0, offset = 0; pixel < indexed.length; pixel += 1, offset += 4) {
    const alpha = rgba[offset + 3];
    if (alpha < 12) {
      indexed[pixel] = 0;
      continue;
    }
    indexed[pixel] = nearestPaletteIndex(rgba[offset], rgba[offset + 1], rgba[offset + 2], palette, lookup);
  }
  return indexed;
}

function rgbaToIndexedDithered(rgba, palette, width, height) {
  const indexed = new Uint8Array(rgba.length / 4);
  const currentError = new Float32Array((width + 2) * 3);
  const nextError = new Float32Array((width + 2) * 3);
  const lookup = new Map();
  for (let y = 0; y < height; y += 1) {
    nextError.fill(0);
    for (let x = 0; x < width; x += 1) {
      const pixel = y * width + x;
      const offset = pixel * 4;
      if (rgba[offset + 3] < 12) {
        indexed[pixel] = 0;
        continue;
      }
      const errorOffset = (x + 1) * 3;
      const red = clampByte(rgba[offset] + currentError[errorOffset]);
      const green = clampByte(rgba[offset + 1] + currentError[errorOffset + 1]);
      const blue = clampByte(rgba[offset + 2] + currentError[errorOffset + 2]);
      const paletteIndex = nearestPaletteIndex(red, green, blue, palette, lookup);
      indexed[pixel] = paletteIndex;
      const paletteOffset = paletteIndex * 3;
      const errorRed = red - palette[paletteOffset];
      const errorGreen = green - palette[paletteOffset + 1];
      const errorBlue = blue - palette[paletteOffset + 2];
      diffuseError(currentError, errorOffset + 3, errorRed, errorGreen, errorBlue, 7 / 16);
      diffuseError(nextError, errorOffset - 3, errorRed, errorGreen, errorBlue, 3 / 16);
      diffuseError(nextError, errorOffset, errorRed, errorGreen, errorBlue, 5 / 16);
      diffuseError(nextError, errorOffset + 3, errorRed, errorGreen, errorBlue, 1 / 16);
    }
    currentError.set(nextError);
  }
  return indexed;
}

function diffuseError(buffer, offset, red, green, blue, scale) {
  if (offset < 0 || offset + 2 >= buffer.length) {
    return;
  }
  buffer[offset] += red * scale;
  buffer[offset + 1] += green * scale;
  buffer[offset + 2] += blue * scale;
}

function nearestPaletteIndex(red, green, blue, palette, lookup) {
  const key = ((red >> 3) << 10) | ((green >> 3) << 5) | (blue >> 3);
  const cached = lookup.get(key);
  if (cached !== undefined) {
    return cached;
  }
  let bestIndex = 0;
  let bestDistance = Infinity;
  for (let index = 0; index < palette.length; index += 3) {
    const dr = red - palette[index];
    const dg = green - palette[index + 1];
    const db = blue - palette[index + 2];
    const distance = dr * dr + dg * dg + db * db;
    if (distance < bestDistance) {
      bestDistance = distance;
      bestIndex = index / 3;
      if (distance === 0) {
        break;
      }
    }
  }
  lookup.set(key, bestIndex);
  return bestIndex;
}

function clampByte(value) {
  return Math.max(0, Math.min(255, Math.round(value)));
}

function lzwEncodeIndexed(indices, minCodeSize) {
  const clearCode = 1 << minCodeSize;
  const endCode = clearCode + 1;
  let nextCode = endCode + 1;
  let codeSize = minCodeSize + 1;
  let dictionary = initialDictionary(clearCode);
  const writer = new BitWriter();

  writer.write(clearCode, codeSize);
  let prefix = String(indices[0]);
  for (let index = 1; index < indices.length; index += 1) {
    const value = indices[index];
    const key = `${prefix},${value}`;
    if (dictionary.has(key)) {
      prefix = key;
      continue;
    }
    writer.write(dictionary.get(prefix), codeSize);
    if (nextCode < 4096) {
      dictionary.set(key, nextCode);
      nextCode += 1;
      // The encoder's table leads the decoder by one emitted code.
      if (nextCode > (1 << codeSize) && codeSize < 12) {
        codeSize += 1;
      }
    } else {
      writer.write(clearCode, codeSize);
      dictionary = initialDictionary(clearCode);
      nextCode = endCode + 1;
      codeSize = minCodeSize + 1;
    }
    prefix = String(value);
  }
  writer.write(dictionary.get(prefix), codeSize);
  writer.write(endCode, codeSize);
  return writer.finish();
}

function initialDictionary(clearCode) {
  const dictionary = new Map();
  for (let index = 0; index < clearCode; index += 1) {
    dictionary.set(String(index), index);
  }
  return dictionary;
}

class BitWriter {
  constructor() {
    this.bytes = [];
    this.bitBuffer = 0;
    this.bitCount = 0;
  }

  write(code, size) {
    this.bitBuffer |= code << this.bitCount;
    this.bitCount += size;
    while (this.bitCount >= 8) {
      this.bytes.push(this.bitBuffer & 0xff);
      this.bitBuffer >>= 8;
      this.bitCount -= 8;
    }
  }

  finish() {
    if (this.bitCount > 0) {
      this.bytes.push(this.bitBuffer & 0xff);
      this.bitBuffer = 0;
      this.bitCount = 0;
    }
    return this.bytes;
  }
}

function writeSubBlocks(bytes, payload) {
  for (let offset = 0; offset < payload.length; offset += 255) {
    const chunk = payload.slice(offset, offset + 255);
    bytes.push(chunk.length);
    writeBytes(bytes, chunk);
  }
  bytes.push(0x00);
}

function writeAscii(bytes, value) {
  writeBytes(bytes, asciiBytes(value));
}

function asciiBytes(value) {
  return [...value].map((char) => char.charCodeAt(0));
}

function writeU16(bytes, value) {
  bytes.push(value & 0xff, (value >> 8) & 0xff);
}

function writeBytes(bytes, values) {
  for (const value of values) {
    bytes.push(value);
  }
}

function yieldToBrowser() {
  return new Promise((resolve) => globalThis.setTimeout(resolve, 0));
}
