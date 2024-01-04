#[cfg(test)]
use crate::*;

/// Test that provided config is merged with
/// default config in case of missing fields.
#[test]
fn test_init_partial() {
    static PARTIAL_YAML: &str = r#"
memos:
#   addr: 127.0.0.1
    mode: demo # default is prod
#   port: 0
memospot:
    log:
        enabled: true # default is false
"#;

    let tmp_dir = tempfile::tempdir().unwrap();
    let partial_yaml_path = tmp_dir.path().join("memospot_partial.yaml");
    fs::write(&partial_yaml_path, PARTIAL_YAML).unwrap();

    let parsed_config = Config::init(&partial_yaml_path).unwrap();

    assert!(parsed_config.memos.addr == "127.0.0.1");
    assert!(parsed_config.memos.port == 0);
    assert!(parsed_config.memos.mode == "demo");
    assert!(parsed_config.memos.log.rotation.amount == 5);
    assert!(parsed_config.memospot.log.enabled);
}

#[test]
fn test_malformed() {
    static MALFORMED_YAML: &str = r#"
memos:
mode: prod
"#;

    let tmp_dir = tempfile::tempdir().unwrap();
    let partial_yaml_path = tmp_dir.path().join("memospot_partial.yaml");
    fs::write(&partial_yaml_path, MALFORMED_YAML).unwrap();

    let Err(e) = Config::init(&partial_yaml_path) else {
        todo!()
    };

    assert!(e
        .to_string()
        .contains("invalid type: found unit, expected struct Memos for key"));

    assert_eq!(e.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn test_parse() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let tmp_yaml = tmp_dir.path().join("memospot.yaml");

    let mut config = Config::init(&tmp_yaml).unwrap();
    config.memos.addr = "0.0.0.0".to_string();

    Config::save_file(&tmp_yaml, &config).unwrap();
    let parsed = Config::parse_file(&tmp_yaml).unwrap();

    assert_eq!(parsed, config);
    assert_ne!(parsed, Config::default());
}

#[test]
fn test_edit() {
    let mut default_config = Config::default();
    default_config.memos.addr = "0.0.0.0".to_string();

    let tmp_dir = tempfile::tempdir().unwrap();
    let tmp_yaml = tmp_dir.path().join("memospot.yaml");

    Config::save_file(&tmp_yaml, &default_config).unwrap();
    let parsed = Config::parse_file(&tmp_yaml).unwrap();

    assert_eq!(parsed, default_config);
    assert_ne!(parsed, Config::default());
}

#[test]
fn test_reset() {
    let default_config = Config::default();
    let default_yaml = serde_yaml::to_string(&default_config).unwrap();

    let tmp_dir = tempfile::tempdir().unwrap();
    let tmp_yaml = tmp_dir.path().join("memospot.yaml");

    Config::reset_file(&tmp_yaml).unwrap();

    let cfg = Config::parse_file(&tmp_yaml).unwrap();
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    assert_eq!(yaml, default_yaml);
}
