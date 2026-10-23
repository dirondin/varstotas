use varstotas_kernel::{Model, ModelMetadata, io};

#[test]
fn metadata_and_unicode_license_round_trip_in_current_version() {
    let mut model = Model::new();
    model.metadata = ModelMetadata {
        text: "Верстак\nАвтор: Žilvinas\nРазмеры в мм 🪚".into(),
        license: "CC BY 4.0\nhttps://creativecommons.org/licenses/by/4.0/".into(),
    };
    let text = io::to_json(&model).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["version"], io::NATIVE_VERSION);
    assert_eq!(value["model"]["metadata"]["text"], model.metadata.text);
    assert_eq!(io::from_json(&text).unwrap().metadata, model.metadata);
}

#[test]
fn version_one_without_metadata_migrates_and_saves_as_current_version() {
    let mut value: serde_json::Value = serde_json::from_str(&io::to_json(&Model::new()).unwrap()).unwrap();
    value["version"] = 1.into();
    value["model"].as_object_mut().unwrap().remove("metadata");
    let model = io::from_json(&value.to_string()).unwrap();
    assert_eq!(model.metadata, ModelMetadata::default());
    let migrated: serde_json::Value = serde_json::from_str(&io::to_json(&model).unwrap()).unwrap();
    assert_eq!(migrated["version"], io::NATIVE_VERSION);
    assert_eq!(migrated["model"]["metadata"]["license"], "");
}

#[test]
fn unsupported_version_is_rejected_before_model_schema_is_read() {
    for version in [0, io::NATIVE_VERSION + 1] {
        // An incompatible schema must report its version, not missing model fields.
        let text = format!(r#"{{"format":"varstotas","version":{version},"model":null}}"#);
        let error = io::from_json(&text).unwrap_err().to_string();
        assert!(error.contains(&version.to_string()), "{error}");
        assert!(!error.contains("missing field"), "{error}");
    }
    assert!(io::from_json(r#"{"format":"varstotas","model":{}}"#).is_err());
}
