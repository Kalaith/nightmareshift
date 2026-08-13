use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const RUNTIME_AUDIO: [&str; 9] = [
    "assets/sounds/brink.wav",
    "assets/sounds/engine_ambience.wav",
    "assets/sounds/meltdown.wav",
    "assets/sounds/rain_ambience.wav",
    "assets/sounds/success.wav",
    "assets/sounds/tension_pulse.wav",
    "assets/sounds/violation.wav",
    "assets/sounds/ward.wav",
    "assets/sounds/warning.wav",
];

#[test]
fn asset_registry_matches_external_runtime_audio() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(root.join("asset_registry.json"))
        .expect("asset_registry.json must be readable");
    let registry: Value = serde_json::from_str(&json).expect("asset registry must be valid JSON");
    assert_eq!(registry["version"], 1);

    let registered: BTreeSet<&str> = registry["assets"]
        .as_array()
        .expect("asset registry needs an assets array")
        .iter()
        .map(|entry| entry.as_str().expect("asset paths must be strings"))
        .collect();
    let expected: BTreeSet<&str> = RUNTIME_AUDIO.into_iter().collect();
    assert_eq!(registered, expected);

    for relative in registered {
        assert!(
            root.join(relative).is_file(),
            "registered runtime asset is missing: {relative}"
        );
    }
}
