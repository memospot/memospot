#[cfg(test)]
use anyhow::bail;
#[cfg(test)]
use {crate::Config, std::fs};

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

    assert_eq!(parsed_config.memos.addr, Some("127.0.0.1".to_string()));
    assert_eq!(parsed_config.memos.port, Some(0));
    assert_eq!(parsed_config.memos.mode, Some("demo".to_string()));
    assert!(parsed_config.memospot.log.enabled.unwrap());
}

#[test]
fn test_malformed() -> Result<(), anyhow::Error> {
    static MALFORMED_YAML: &str = r#"
memospot:
mode: prod
"#;

    let tmp_dir = tempfile::tempdir().unwrap();
    let partial_yaml_path = tmp_dir.path().join("memospot_partial.yaml");
    fs::write(&partial_yaml_path, MALFORMED_YAML).unwrap();

    let Err(e) = Config::init(&partial_yaml_path) else {
        bail!("init must fail with malformed YAML");
    };

    assert!(e
        .to_string()
        .contains("invalid type: found unit, expected struct Memospot for key"));
    Ok(())
}

#[test]
fn test_gibberish() -> Result<(), anyhow::Error> {
    static INVALID_YAML: &str = r#"
\0\xqwefdklsaj
"#;

    let tmp_dir = tempfile::tempdir().unwrap();
    let partial_yaml_path = tmp_dir.path().join("memospot_gibberish.yaml");
    fs::write(&partial_yaml_path, INVALID_YAML).unwrap();

    let Err(_) = Config::init(&partial_yaml_path) else {
        bail!("init must fail with invalid YAML");
    };

    Ok(())
}

#[tokio::test]
async fn test_parse() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let tmp_yaml = tmp_dir.path().join("memospot.yaml");

    let mut config = Config::init(&tmp_yaml).unwrap();
    config.memos.addr = Some("0.0.0.0".to_string());

    config.save_to_file(&tmp_yaml).await.unwrap();

    let parsed = Config::parse_file(&tmp_yaml).unwrap();

    assert_eq!(parsed, config);
    assert_ne!(parsed, Config::default());
}

#[tokio::test]
async fn test_edit() {
    let mut default_config = Config::default();
    default_config.memos.addr = Some("0.0.0.0".to_string());

    let tmp_dir = tempfile::tempdir().unwrap();
    let tmp_yaml = tmp_dir.path().join("memospot.yaml");

    default_config.save_to_file(&tmp_yaml).await.unwrap();
    let parsed = Config::parse_file(&tmp_yaml).unwrap();

    assert_eq!(parsed, default_config);
    assert_ne!(parsed, Config::default());
}

#[tokio::test]
async fn test_reset() {
    let default_config = Config::default();
    let default_yaml = serde_yaml::to_string(&default_config).unwrap();

    let tmp_dir = tempfile::tempdir().unwrap();
    let tmp_yaml = tmp_dir.path().join("memospot.yaml");

    Config::reset_file(&tmp_yaml).await.ok();

    let cfg = Config::parse_file(&tmp_yaml).unwrap();
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    assert_eq!(yaml, default_yaml);
}

#[test]
fn test_show() {
    let default_config = Config::default();
    let default_yaml = serde_yaml::to_string(&default_config).unwrap();
    println!("{default_yaml}");
}
