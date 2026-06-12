use rusty_optics_stimulus::StimulusProfile;

/// Serializes the deterministic stimulus volume preview profile.
pub fn volume_interference_preview_profile_json() -> Result<String, serde_json::Error> {
    let mut json = serde_json::to_string_pretty(&StimulusProfile::volume_interference_preview(
        "stimulus.profile.volume_interference_preview",
    ))?;
    json.push('\n');
    Ok(json)
}
