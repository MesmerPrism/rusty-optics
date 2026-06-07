const RealtimeHandSdf = (() => {
  const SURFACE_SEQUENCE_SCHEMA_ID = "rusty.matter.tools.glb_mesh_surface_sequence.v1";
  const DEFAULT_PARTICLE_COUNT = 1000;
  const meshLineColor = { r: 1.0, g: 0.63, b: 0.10, a: 1.0 };
  const colliderLineColor = { r: 0.98, g: 0.68, b: 0.24, a: 0.78 };
  const coordinateColor = { r: 0.2, g: 0.74, b: 1.0, a: 0.90 };

  function isSurfaceSequence(payload) {
    return payload?.schema_id === SURFACE_SEQUENCE_SCHEMA_ID && Array.isArray(payload.frames);
  }

  function normalizeSurfaceSequence(payload) {
    if (!isSurfaceSequence(payload)) {
      throw new Error("payload is not a Matter surface sequence");
    }
    if (!Array.isArray(payload.triangles) || payload.triangles.length === 0) {
      throw new Error("surface sequence is missing triangle topology");
    }
    if (!payload.frames.length) {
      throw new Error("surface sequence has no frames");
    }
    const vertexCount = payload.vertex_count || payload.frames[0].positions.length;
    const edgePairs = uniqueEdgePairs(payload.triangles);
    const bounds = {
      min: payload.bounds_min || payload.frames[0].bounds_min,
      max: payload.bounds_max || payload.frames[0].bounds_max,
    };
    const size = subtract(bounds.max, bounds.min);
    const radius = Math.max(0.001, length(size) * 0.5);
    const durationSeconds = Math.max(0.001, payload.duration_seconds || payload.frames.length / 12);
    return {
      schema_id: payload.schema_id,
      sequence_id: payload.sequence_id || "mesh.surface_sequence.browser",
      mesh_name: payload.mesh_name || "mesh",
      animation_name: payload.animation_name || "animation",
      duration_seconds: durationSeconds,
      frame_count: payload.frames.length,
      frame_seconds: durationSeconds / payload.frames.length,
      vertex_count: vertexCount,
      triangle_count: payload.triangles.length,
      triangles: payload.triangles,
      edge_pairs: edgePairs,
      frames: payload.frames,
      bounds_min: bounds.min,
      bounds_max: bounds.max,
      center: scale(add(bounds.min, bounds.max), 0.5),
      radius,
      cloud_radius: radius * 2.45,
      particle_count: DEFAULT_PARTICLE_COUNT,
    };
  }

  function buildRuntimeFrame(sequence, frameIndex, metrics = null) {
    const startedAt = nowMs();
    const source = sequence.frames[frameIndex % sequence.frames.length];
    const positions = source.positions;
    const boundsMin = source.bounds_min || boundsForPositions(positions).min;
    const boundsMax = source.bounds_max || boundsForPositions(positions).max;
    const triangleCache = sequence.triangles.map((triangle) => ({
      indices: triangle,
      a: positions[triangle[0]],
      b: positions[triangle[1]],
      c: positions[triangle[2]],
    }));
    const edges = sequence.edge_pairs.map(([startIndex, endIndex]) => ({
      start: positions[startIndex],
      end: positions[endIndex],
      role: "SurfaceEdge",
      color: meshLineColor,
    }));
    const colliderEdges = sequence.edge_pairs.map(([startIndex, endIndex]) => ({
      start: positions[startIndex],
      end: positions[endIndex],
      role: "ColliderShell",
      color: colliderLineColor,
    }));
    const coordinates = coordinateVisual(sequence, positions, boundsMin, boundsMax);
    const runtimeSurface = {
      positions,
      triangles: sequence.triangles,
      triangle_cache: triangleCache,
      bounds_min: boundsMin,
      bounds_max: boundsMax,
      center: scale(add(boundsMin, boundsMax), 0.5),
      radius: Math.max(0.001, length(subtract(boundsMax, boundsMin)) * 0.5),
      sequence_center: sequence.center,
      sequence_cloud_radius: sequence.cloud_radius,
    };
    const frame = {
      schema_id: "rusty.optics.mesh.browser.realtime_frame.v1",
      frame_id: `${sequence.sequence_id}.browser_frame.${source.frame_index}`,
      source_frame_id: source.surface_id,
      source_surface_id: source.surface_id,
      realtime_surface_sequence: true,
      realtime_frame_index: source.frame_index,
      realtime_time_seconds: source.time_seconds,
      mesh: {
        schema_id: "rusty.optics.mesh.debug.frame.v1",
        frame_id: `${source.surface_id}.mesh_debug`,
        source_surface_id: source.surface_id,
        source_schema_id: "rusty.matter.mesh.surface.v1",
        topology_index_hash: 0,
        vertices: positions.map((position, index) => ({ index, position })),
        triangles: sequence.triangles,
        edges,
        bounds_min: boundsMin,
        bounds_max: boundsMax,
      },
      coordinates,
      collider: {
        schema_id: "rusty.optics.mesh.collider.visual.v1",
        visual_id: `${source.surface_id}.runtime_collider`,
        source_surface_id: source.surface_id,
        update_status: "runtime_recomputed",
        collider_vertex_count: positions.length,
        collider_triangle_count: sequence.triangles.length,
        shell_edges: colliderEdges,
        contact_points: [],
        contact_normals: [],
      },
      sdf_slice: sdfSliceVisual(runtimeSurface, metrics),
      runtime_surface: runtimeSurface,
    };
    addMetric(metrics, "runtimeFrameBuildMs", nowMs() - startedAt);
    return frame;
  }

  function resetParticles(sequence, runtimeFrame, seed) {
    const count = sequence.particle_count || DEFAULT_PARTICLE_COUNT;
    const sphereRadius = sequence.cloud_radius;
    const particleRadius = Math.max(sequence.radius * 0.009, 0.0012);
    const particles = [];
    for (let index = 0; index < count; index += 1) {
      const direction = randomUnitDirection(index, seed);
      const radial = Math.cbrt(unitHash(index, seed + 113)) * sphereRadius;
      const position = add(sequence.center, scale(direction, radial));
      particles.push({
        position,
        velocity: scale(direction, -sequence.radius * 0.025),
        radius: particleRadius,
        color: { r: 0.20, g: 0.84, b: 1.0, a: 0.86 },
        trail: [position],
      });
    }
    return particles;
  }

  function stepParticles(particles, runtimeFrame, deltaSeconds, options = {}) {
    const startedAt = nowMs();
    const trailsEnabled = Boolean(options.trailsEnabled);
    const metrics = options.metrics || null;
    const surface = runtimeFrame.runtime_surface;
    const targetDistance = Math.max(particles[0]?.radius || surface.radius * 0.01, 0.0008) * 0.65;
    const maxSpeed = surface.radius * 1.9;
    const substeps = Math.max(1, Math.ceil(deltaSeconds / (1 / 45)));
    const subDelta = deltaSeconds / substeps;
    for (let substep = 0; substep < substeps; substep += 1) {
      for (const particle of particles) {
        const sample = closestSurfaceSample(surface, particle.position);
        const outward = sample.distance > 1.0e-7
          ? normalize(subtract(particle.position, sample.point))
          : sample.normal;
        const error = sample.distance - targetDistance;
        let acceleration = scale(outward, -error * 19.0);
        const cloudOffset = subtract(particle.position, surface.sequence_center);
        const cloudDistance = length(cloudOffset);
        if (cloudDistance > surface.sequence_cloud_radius * 1.12) {
          acceleration = add(
            acceleration,
            scale(normalize(cloudOffset), -(cloudDistance - surface.sequence_cloud_radius) * 7.0),
          );
        }
        particle.velocity = add(particle.velocity, scale(acceleration, subDelta));
        particle.velocity = scale(particle.velocity, Math.max(0, 1.0 - 1.55 * subDelta));
        particle.velocity = clampVectorLength(particle.velocity, maxSpeed);
        particle.position = add(particle.position, scale(particle.velocity, subDelta));
      }
    }

    for (const particle of particles) {
      const sample = closestSurfaceSample(surface, particle.position);
      const distance01 = clamp(sample.distance / Math.max(surface.radius * 0.22, 0.001), 0, 1);
      const speed01 = clamp(length(particle.velocity) / maxSpeed, 0, 1);
      particle.color = {
        r: 0.10 + speed01 * 0.48,
        g: 0.70 + (1 - distance01) * 0.24,
        b: 1.0,
        a: 0.72 + (1 - distance01) * 0.24,
      };
      if (trailsEnabled) {
        particle.trail.push({ ...particle.position });
        while (particle.trail.length > 10) {
          particle.trail.shift();
        }
      } else {
        particle.trail = [particle.position];
      }
    }
    const sampleCount = particles.length * substeps + particles.length;
    addMetric(metrics, "particleStepMs", nowMs() - startedAt);
    setMetric(metrics, "particleSubsteps", substeps);
    setMetric(metrics, "particleClosestSamples", sampleCount);
    setMetric(metrics, "particleTriangleChecks", sampleCount * surface.triangle_cache.length);
    setMetric(metrics, "runtimeTriangleCount", surface.triangle_cache.length);
  }

  function coordinateVisual(sequence, positions, boundsMin, boundsMax) {
    const axisLength = Math.max(length(subtract(boundsMax, boundsMin)) * 0.035, 0.004);
    const step = Math.max(1, Math.floor(positions.length / 48));
    const anchors = [];
    const axes = [];
    for (let sourceIndex = 0; sourceIndex < positions.length && anchors.length < 48; sourceIndex += step) {
      const position = positions[sourceIndex];
      anchors.push({
        point_id: `mesh.coordinate.runtime.${anchors.length.toString().padStart(4, "0")}`,
        position,
        radius: axisLength * 0.18,
        role: "CoordinateAnchor",
        color: coordinateColor,
      });
      axes.push({
        start: position,
        end: add(position, { x: axisLength, y: 0, z: 0 }),
        role: "CoordinateAxisX",
        color: { r: 0.25, g: 0.82, b: 1.0, a: 0.70 },
      });
    }
    return {
      schema_id: "rusty.optics.mesh.coordinate.visual.v1",
      visual_id: `${sequence.sequence_id}.runtime_coordinates`,
      source_surface_id: sequence.sequence_id,
      anchors,
      axes,
    };
  }

  function sdfSliceVisual(surface, metrics = null) {
    const startedAt = nowMs();
    const boundsMin = surface.bounds_min;
    const boundsMax = surface.bounds_max;
    const size = subtract(boundsMax, boundsMin);
    const pad = Math.max(surface.radius * 0.12, 0.01);
    const min = subtract(boundsMin, { x: pad, y: pad, z: pad });
    const max = add(boundsMax, { x: pad, y: pad, z: pad });
    const width = clampInt(Math.round(size.x / Math.max(surface.radius * 0.035, 0.004)), 24, 42);
    const height = clampInt(Math.round(size.y / Math.max(surface.radius * 0.035, 0.004)), 16, 32);
    const z = surface.center.z;
    const cells = [];
    let minDistance = Number.POSITIVE_INFINITY;
    let maxDistance = 0;
    for (let y = 0; y < height; y += 1) {
      for (let x = 0; x < width; x += 1) {
        const point = {
          x: min.x + (x + 0.5) * ((max.x - min.x) / width),
          y: min.y + (y + 0.5) * ((max.y - min.y) / height),
          z,
        };
        const sample = closestSurfaceSample(surface, point);
        minDistance = Math.min(minDistance, sample.distance);
        maxDistance = Math.max(maxDistance, sample.distance);
        cells.push({
          grid: [x, y, 0],
          plane: [x, y],
          position: point,
          distance: sample.distance,
          normalized_distance: 0.5,
        });
      }
    }
    const range = Math.max(maxDistance - minDistance, 1.0e-6);
    for (const cell of cells) {
      cell.normalized_distance = clamp((cell.distance - minDistance) / range, 0, 1);
    }
    addMetric(metrics, "runtimeSdfSliceMs", nowMs() - startedAt);
    setMetric(metrics, "runtimeSdfCells", cells.length);
    setMetric(metrics, "runtimeSdfTriangleChecks", cells.length * surface.triangle_cache.length);
    return {
      schema_id: "rusty.optics.sdf.slice.visual.v1",
      visual_id: `${surface.sequence_id || "runtime"}.sdf_slice`,
      source_grid_id: "runtime.recomputed.sdf_slice",
      source_schema_id: "runtime.triangle_distance",
      axis: "Z",
      slice_index: 0,
      width,
      height,
      min_distance: minDistance,
      max_distance: maxDistance,
      cells,
    };
  }

  function closestSurfaceSample(surface, point) {
    let bestDistanceSquared = Number.POSITIVE_INFINITY;
    let bestPoint = surface.triangle_cache[0]?.a || surface.center;
    let bestNormal = { x: 0, y: 1, z: 0 };
    for (const triangle of surface.triangle_cache) {
      const closest = closestPointOnTriangle(point, triangle.a, triangle.b, triangle.c);
      const delta = subtract(point, closest);
      const distanceSquared = dot(delta, delta);
      if (distanceSquared < bestDistanceSquared) {
        bestDistanceSquared = distanceSquared;
        bestPoint = closest;
        bestNormal = triangleNormal(triangle.a, triangle.b, triangle.c);
      }
    }
    return {
      point: bestPoint,
      normal: bestNormal,
      distance: Math.sqrt(bestDistanceSquared),
    };
  }

  function closestPointOnTriangle(point, a, b, c) {
    const ab = subtract(b, a);
    const ac = subtract(c, a);
    const ap = subtract(point, a);
    const d1 = dot(ab, ap);
    const d2 = dot(ac, ap);
    if (d1 <= 0 && d2 <= 0) {
      return a;
    }

    const bp = subtract(point, b);
    const d3 = dot(ab, bp);
    const d4 = dot(ac, bp);
    if (d3 >= 0 && d4 <= d3) {
      return b;
    }

    const vc = d1 * d4 - d3 * d2;
    if (vc <= 0 && d1 >= 0 && d3 <= 0) {
      const v = d1 / (d1 - d3);
      return add(a, scale(ab, v));
    }

    const cp = subtract(point, c);
    const d5 = dot(ab, cp);
    const d6 = dot(ac, cp);
    if (d6 >= 0 && d5 <= d6) {
      return c;
    }

    const vb = d5 * d2 - d1 * d6;
    if (vb <= 0 && d2 >= 0 && d6 <= 0) {
      const w = d2 / (d2 - d6);
      return add(a, scale(ac, w));
    }

    const va = d3 * d6 - d5 * d4;
    if (va <= 0 && d4 - d3 >= 0 && d5 - d6 >= 0) {
      const w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
      return add(b, scale(subtract(c, b), w));
    }

    const denom = 1 / (va + vb + vc);
    const v = vb * denom;
    const w = vc * denom;
    return add(a, add(scale(ab, v), scale(ac, w)));
  }

  function triangleNormal(a, b, c) {
    return normalize(cross(subtract(b, a), subtract(c, a)));
  }

  function uniqueEdgePairs(triangles) {
    const seen = new Set();
    const edges = [];
    for (const triangle of triangles) {
      for (const [left, right] of [[triangle[0], triangle[1]], [triangle[1], triangle[2]], [triangle[2], triangle[0]]]) {
        const start = Math.min(left, right);
        const end = Math.max(left, right);
        const key = `${start}:${end}`;
        if (!seen.has(key)) {
          seen.add(key);
          edges.push([start, end]);
        }
      }
    }
    return edges;
  }

  function boundsForPositions(positions) {
    const min = { ...positions[0] };
    const max = { ...positions[0] };
    for (const position of positions.slice(1)) {
      min.x = Math.min(min.x, position.x);
      min.y = Math.min(min.y, position.y);
      min.z = Math.min(min.z, position.z);
      max.x = Math.max(max.x, position.x);
      max.y = Math.max(max.y, position.y);
      max.z = Math.max(max.z, position.z);
    }
    return { min, max };
  }

  function randomUnitDirection(index, seed) {
    const z = unitHash(index, seed) * 2 - 1;
    const angle = unitHash(index, seed + 41) * Math.PI * 2;
    const radius = Math.sqrt(Math.max(0, 1 - z * z));
    return {
      x: Math.cos(angle) * radius,
      y: Math.sin(angle) * radius,
      z,
    };
  }

  function unitHash(index, seed) {
    let value = Math.imul(index + 1, 2654435761) ^ Math.imul(seed + 1, 2246822519);
    value ^= value >>> 16;
    value = Math.imul(value, 2246822507);
    value ^= value >>> 13;
    value = Math.imul(value, 3266489909);
    value ^= value >>> 16;
    return (value >>> 8) / 16777215;
  }

  function add(left, right) {
    return { x: left.x + right.x, y: left.y + right.y, z: left.z + right.z };
  }

  function subtract(left, right) {
    return { x: left.x - right.x, y: left.y - right.y, z: left.z - right.z };
  }

  function scale(vector, scalar) {
    return { x: vector.x * scalar, y: vector.y * scalar, z: vector.z * scalar };
  }

  function length(vector) {
    return Math.sqrt(dot(vector, vector));
  }

  function normalize(vector) {
    const vectorLength = length(vector);
    if (!Number.isFinite(vectorLength) || vectorLength <= 1.0e-8) {
      return { x: 0, y: 1, z: 0 };
    }
    return scale(vector, 1 / vectorLength);
  }

  function dot(left, right) {
    return left.x * right.x + left.y * right.y + left.z * right.z;
  }

  function cross(left, right) {
    return {
      x: left.y * right.z - left.z * right.y,
      y: left.z * right.x - left.x * right.z,
      z: left.x * right.y - left.y * right.x,
    };
  }

  function clampVectorLength(vector, maxLength) {
    const vectorLength = length(vector);
    if (!Number.isFinite(vectorLength) || vectorLength <= maxLength) {
      return vector;
    }
    return scale(vector, maxLength / vectorLength);
  }

  function clamp(value, min, max) {
    return Math.min(max, Math.max(min, value));
  }

  function clampInt(value, min, max) {
    return Math.round(clamp(value, min, max));
  }

  function nowMs() {
    return globalThis.performance?.now?.() ?? Date.now();
  }

  function addMetric(metrics, key, value) {
    if (!metrics || !Number.isFinite(value)) {
      return;
    }
    metrics[key] = (metrics[key] || 0) + value;
  }

  function setMetric(metrics, key, value) {
    if (!metrics || !Number.isFinite(value)) {
      return;
    }
    metrics[key] = value;
  }

  return {
    DEFAULT_PARTICLE_COUNT,
    isSurfaceSequence,
    normalizeSurfaceSequence,
    buildRuntimeFrame,
    resetParticles,
    stepParticles,
  };
})();
