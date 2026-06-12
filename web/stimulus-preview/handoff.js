import {
  applyTuningToProfile,
  tuningToHash,
} from "./tuning.js";

const BROWSER_PRESET_SCHEMA = "rusty.optics.stimulus.browser_preset.v1";
const QUEST_HANDOFF_SCHEMA = "rusty.optics.stimulus.quest_handoff.v1";
const EFFECTIVE_SETTINGS_SCHEMA = "rusty.gui.makepad.effective_settings.v1";
const SETTINGS_SURFACE_SCHEMA = "rusty.gui.makepad.app_settings_surface.v1";
const QUEST_CAMERA_SHELL_APP_ID = "rusty-quest-makepad.camera-shell";
const QUEST_SETTINGS_SURFACE_VERSION = 10;
const PRESET_STORE_KEY = "rusty.optics.stimulus.presets.v1";
const PRESET_LIMIT = 24;
const STIMULUS_PROFILE_PATH = "stimulus/stimulus-profile.json";
const STIMULUS_TUNING_PATH = "stimulus/stimulus-tuning.json";

const BASE_QUEST_SETTINGS = [
  setting("makepad.mesh_replay.enabled", false, "frame_safe", "profile_owned", "meshReplayEnabled"),
  setting("makepad.mesh_replay.speed", 1.0, "frame_safe", "session_hotload", "meshReplaySpeed"),
  setting("makepad.mesh_replay.source", "public-synthetic-hand-sequence", "startup_only", "profile_owned", "meshReplaySource"),
  setting("makepad.mesh_replay.opacity", 0.0, "frame_safe", "session_hotload", "meshReplayOpacity"),
  setting("makepad.render.scale", 1.0, "scene_rebuild", "profile_owned", "renderScale"),
  setting("makepad.camera.streaming.enabled", false, "scene_rebuild", "session_hotload", "cameraStreamingEnabled"),
  setting("makepad.collision.enabled", false, "frame_safe", "profile_owned", "collisionEnabled"),
  setting("makepad.sdf_adf.overlay_mode", "off", "scene_rebuild", "profile_owned", "sdfAdfOverlayMode"),
  setting("makepad.particles.enabled", false, "frame_safe", "profile_owned", "particlesEnabled"),
  setting("makepad.particles.render.draw_limit", 0, "frame_safe", "profile_owned", "particlesRenderDrawLimit"),
  setting("makepad.particles.render.animation_mode", "static-ring", "frame_safe", "profile_owned", "particlesRenderAnimationMode"),
  setting("makepad.particles.render.size_scale", 1.0, "frame_safe", "profile_owned", "particlesRenderSizeScale"),
];

export function loadStoredPresets() {
  try {
    const stored = JSON.parse(localStorage.getItem(PRESET_STORE_KEY) ?? "[]");
    if (!Array.isArray(stored)) {
      return [];
    }
    return stored.filter((preset) => preset?.schema === BROWSER_PRESET_SCHEMA);
  } catch {
    return [];
  }
}

export function saveStoredPreset(preset) {
  const presets = loadStoredPresets().filter((stored) => stored.preset_id !== preset.preset_id);
  presets.unshift(preset);
  localStorage.setItem(PRESET_STORE_KEY, JSON.stringify(presets.slice(0, PRESET_LIMIT)));
  return presets.slice(0, PRESET_LIMIT);
}

export async function createBrowserPreset({ profile, tuning, elapsedSeconds }) {
  const generatedAt = new Date().toISOString();
  const tunedProfile = applyTuningToProfile(profile, tuning, elapsedSeconds);
  const tuningJson = stableJson(tuning);
  const tuningSha256 = await sha256Hex(tuningJson);
  const profileId = tunedProfile.profile_id ?? profile?.profile_id ?? "stimulus.profile.unknown";
  return {
    schema: BROWSER_PRESET_SCHEMA,
    preset_id: `${profileId}.${generatedAt.replace(/[:.]/g, "-")}`,
    saved_at: generatedAt,
    base_profile_id: profile?.profile_id ?? "stimulus.profile.unknown",
    profile_id: profileId,
    profile_schema_id: tunedProfile.schema_id ?? profile?.schema_id ?? "rusty.optics.stimulus.profile.v1",
    elapsed_seconds: finite(elapsedSeconds, 0),
    tuning_url_hash: tuningToHash(tuning),
    tuning_sha256: tuningSha256,
    tuning,
    stimulus_profile: tunedProfile,
  };
}

export async function createQuestHandoff({ profile, tuning, elapsedSeconds }) {
  const generatedAt = new Date().toISOString();
  const stimulusProfile = applyTuningToProfile(profile, tuning, elapsedSeconds);
  const stimulusProfileJson = stableJson(stimulusProfile);
  const stimulusProfileSha256 = await sha256Hex(stimulusProfileJson);
  const stimulusTuningJson = stableJson({
    schema: "rusty.optics.stimulus.browser_tuning_export.v1",
    generated_at: generatedAt,
    source_profile_id: profile?.profile_id ?? "stimulus.profile.unknown",
    elapsed_seconds: finite(elapsedSeconds, 0),
    tuning_url_hash: tuningToHash(tuning),
    tuning,
  });
  const stimulusTuningSha256 = await sha256Hex(stimulusTuningJson);
  const profileSchema = stimulusProfile.schema_id ?? "rusty.optics.stimulus.profile.v1";
  const profileId = stimulusProfile.profile_id ?? "stimulus.profile.browser_export";
  const presentationMode = stimulusProfile.presentation?.mode ?? "StereoEyeField";
  const effectiveSettings = createEffectiveSettingsReport({
    generatedAt,
    profileSchema,
    stimulusProfileSha256,
    stimulusTuningSha256,
    presentationMode,
  });

  return {
    schema: QUEST_HANDOFF_SCHEMA,
    generated_at: generatedAt,
    source: {
      adapter: "rusty-optics.web.stimulus-preview",
      base_profile_id: profile?.profile_id ?? "stimulus.profile.unknown",
      profile_id: profileId,
      elapsed_seconds: finite(elapsedSeconds, 0),
    },
    files: {
      effective_settings: "effective-settings.json",
      stimulus_profile: STIMULUS_PROFILE_PATH,
      stimulus_tuning: STIMULUS_TUNING_PATH,
    },
    stimulus_profile_sha256: stimulusProfileSha256,
    stimulus_tuning_sha256: stimulusTuningSha256,
    stimulus_profile_json: stimulusProfileJson,
    stimulus_tuning_json: stimulusTuningJson,
    stimulus_profile: stimulusProfile,
    browser_tuning: {
      schema: "rusty.optics.stimulus.browser_tuning_ref.v1",
      tuning_url_hash: tuningToHash(tuning),
      tuning_sha256: stimulusTuningSha256,
      tuning,
    },
    quest_makepad: {
      app_id: QUEST_CAMERA_SHELL_APP_ID,
      presentation_mode: presentationMode,
      effective_settings_file: "effective-settings.json",
      profile_path: STIMULUS_PROFILE_PATH,
      tuning_path: STIMULUS_TUNING_PATH,
    },
    effective_settings: effectiveSettings,
    notes: [
      "Stimulus profile and browser tuning are staged as sibling data-plane files.",
      "Effective settings carry only low-rate enable/path/schema/hash/presentation controls.",
    ],
  };
}

export function downloadJson(value, filename) {
  const blob = new Blob([`${JSON.stringify(value, null, 2)}\n`], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function questHandoffFilename(profileId) {
  const safeId = String(profileId ?? "stimulus-profile")
    .replace(/[^a-z0-9_.-]+/gi, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 96) || "stimulus-profile";
  const stamp = new Date().toISOString().replace(/[:.]/g, "-");
  return `${safeId}.quest-handoff.${stamp}.json`;
}

function createEffectiveSettingsReport({
  generatedAt,
  profileSchema,
  stimulusProfileSha256,
  stimulusTuningSha256,
  presentationMode,
}) {
  const stimulusSettings = [
    setting("makepad.stimulus.enabled", true, "scene_rebuild", "profile_owned", "stimulusEnabled"),
    setting("makepad.stimulus.profile_path", STIMULUS_PROFILE_PATH, "scene_rebuild", "profile_owned", "stimulusProfilePath"),
    setting("makepad.stimulus.profile_sha256", stimulusProfileSha256, "scene_rebuild", "profile_owned", "stimulusProfileSha256"),
    setting("makepad.stimulus.profile_schema", profileSchema, "scene_rebuild", "profile_owned", "stimulusProfileSchema"),
    setting("makepad.stimulus.tuning_path", STIMULUS_TUNING_PATH, "scene_rebuild", "profile_owned", "stimulusTuningPath"),
    setting("makepad.stimulus.tuning_sha256", stimulusTuningSha256, "scene_rebuild", "profile_owned", "stimulusTuningSha256"),
    setting("makepad.stimulus.presentation_mode", presentationMode, "scene_rebuild", "profile_owned", "stimulusPresentationMode"),
  ];
  return {
    schema: EFFECTIVE_SETTINGS_SCHEMA,
    app_id: QUEST_CAMERA_SHELL_APP_ID,
    surface_schema: SETTINGS_SURFACE_SCHEMA,
    surface_version: QUEST_SETTINGS_SURFACE_VERSION,
    revision: 1,
    generated_at: generatedAt,
    settings: [...BASE_QUEST_SETTINGS, ...stimulusSettings],
  };
}

function setting(settingId, value, hotloadPolicy, writerPolicy, readbackField) {
  return {
    setting_id: settingId,
    value,
    winning_layer: "browser_export",
    winning_source_id: "profile.quest_makepad.browser_stimulus_export",
    rejected_layers: [],
    hotload_policy: hotloadPolicy,
    writer_policy: writerPolicy,
    readback_field: readbackField,
  };
}

function stableJson(value) {
  return JSON.stringify(sortJson(value));
}

function sortJson(value) {
  if (Array.isArray(value)) {
    return value.map(sortJson);
  }
  if (value && typeof value === "object") {
    return Object.keys(value)
      .sort()
      .reduce((object, key) => {
        object[key] = sortJson(value[key]);
        return object;
      }, {});
  }
  return value;
}

async function sha256Hex(text) {
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function finite(value, fallback) {
  return Number.isFinite(value) ? value : fallback;
}
