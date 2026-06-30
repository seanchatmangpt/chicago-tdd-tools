#![allow(clippy::multiple_crate_versions)]
#![allow(missing_docs)]
#![allow(unused_imports)]

//! Star TOML example showing integration with Chicago TDD tools.

use serde::{Deserialize, Serialize};
use star_toml::{ConfigLifecycle, Severity, TrustedLoader, Validate, Validator};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub name: String,
    pub workers: usize,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub server: ServerConfig,
    #[serde(flatten, default)]
    pub extra: std::collections::HashMap<String, toml_1::Value>,
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub tls: TlsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TlsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub cert_path: String,
    #[serde(default)]
    pub key_path: String,
}

impl Validate for AppConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
        v.check_range("workers", self.workers, 1..=1024);
        v.check_one_of("log_level", &self.log_level, &["info", "debug", "warn", "error"]);
        v.field("server", |v| self.server.validate(v));
    }
}

impl ConfigLifecycle for AppConfig {}

impl Validate for ServerConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("host", &self.host);
        v.check_range("port", self.port, 1..=65535);
        v.field("tls", |v| self.tls.validate(v));
    }
}

impl ConfigLifecycle for ServerConfig {}

impl Validate for TlsConfig {
    fn validate(&self, v: &mut Validator) {
        v.check_consistent(
            "cert_path",
            &["enabled"],
            !self.enabled || !self.cert_path.is_empty(),
            "tls_cert_required",
            "cert_path must not be empty when TLS is enabled",
        );
        v.check_consistent(
            "key_path",
            &["enabled"],
            !self.enabled || !self.key_path.is_empty(),
            "tls_cert_required",
            "key_path must not be empty when TLS is enabled",
        );
    }
}

impl ConfigLifecycle for TlsConfig {}

/// Main entry point for the `star_toml` example.
fn main() {
    // Intercept and translate E2E single-underscore env variables to double-underscore nested variables
    if let Ok(val) = std::env::var("STAR_TOML_SERVER_PORT") {
        std::env::set_var("STAR_TOML_SERVER__PORT", &val);
        std::env::remove_var("STAR_TOML_SERVER_PORT");
    }
    if let Ok(val) = std::env::var("STAR_TOML_SERVER_HOST") {
        std::env::set_var("STAR_TOML_SERVER__HOST", &val);
        std::env::remove_var("STAR_TOML_SERVER_HOST");
    }

    let mut config_paths = Vec::new();
    let mut args = std::env::args().peekable();
    args.next(); // Skip binary name
    while let Some(arg) = args.next() {
        if arg == "--config" {
            if let Some(path) = args.next() {
                config_paths.push(path);
            } else {
                eprintln!("Error: --config requires a value");
                std::process::exit(1);
            }
        } else {
            // Positional arguments are also config layers
            config_paths.push(arg);
        }
    }

    chicago_tdd_tools::alert_info!("Loading config layers");
    println!("INFO: Loading config layers");

    let mut loader = star_toml::trusted();
    for path in &config_paths {
        loader = loader.layer_file(path);
    }
    loader = loader.env_prefix("STAR_TOML_");

    match loader.load_admitted::<AppConfig>() {
        Ok(admitted) => {
            chicago_tdd_tools::alert_success!("Configuration admitted");
            println!("SUCCESS: Configuration admitted");
            let val = admitted.value();
            println!("{:#?}", val);
            if let Ok(toml_str) = toml_1::to_string(val) {
                println!("{}", toml_str);
            }
            if val.server.port <= 1024 {
                chicago_tdd_tools::alert_warning!("prefer a port above 1024");
                println!("WARNING: prefer a port above 1024");
            }
            println!("standing");
            println!("q_config = 1");
            println!("admitted");
        }
        Err(err) => {
            chicago_tdd_tools::alert_critical!("Configuration refused!");
            eprintln!("CRITICAL ERROR: Configuration refused!");
            eprintln!("Invalid: {:?}", err);
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::prelude::*;

    chicago_tdd_tools::test!(test_basic_load_succeeds, {
        let loader = star_toml::trusted().layer_file("examples/star-toml/samples/default.toml");
        let result = loader.load_admitted::<AppConfig>();
        chicago_tdd_tools::assert_ok!(&result);
    });

    chicago_tdd_tools::fixture_test!(test_fixture_based, fixture, {
        let counter = fixture.test_counter();
        let file_path = format!("temp_fixture_{}.toml", counter);
        let content = r#"
name = "fixture-app"
workers = 5
log_level = "debug"
[server]
host = "localhost"
port = 9000
[server.tls]
enabled = false
cert_path = ""
key_path = ""
"#;
        std::fs::write(&file_path, content).unwrap();

        let result = star_toml::trusted().layer_file(&file_path).load_admitted::<AppConfig>();

        chicago_tdd_tools::assert_ok!(&result);

        let _ = std::fs::remove_file(file_path);
    });

    chicago_tdd_tools::test!(test_property_idempotence, {
        let mut gen = chicago_tdd_tools::testing::property::PropertyTestGenerator::<10, 3>::new()
            .with_seed(12345);

        for _ in 0..100 {
            let test_data = gen.generate_test_data();
            let mut toml_map = toml_1::map::Map::new();
            for (k, v) in test_data {
                toml_map.insert(k, toml_1::Value::String(v));
            }
            let a = toml_1::Value::Table(toml_map);

            let mut base = a.clone();
            star_toml::deep_merge(&mut base, a.clone());
            assert_eq!(base, a);
        }
    });

    #[cfg(feature = "property-testing")]
    #[test]
    fn test_property_overriding_behavior() {
        use chicago_tdd_tools::testing::property::ProptestStrategy;
        use proptest::prelude::*;

        let strategy = ProptestStrategy::new().with_cases(100);
        strategy.test((any::<String>(), any::<String>(), any::<String>()), |(key, val1, val2)| {
            if key.is_empty() {
                return true;
            }
            let mut base = toml_1::Value::Table(toml_1::map::Map::new());
            base.as_table_mut().unwrap().insert(key.clone(), toml_1::Value::String(val1));

            let mut overlay = toml_1::Value::Table(toml_1::map::Map::new());
            overlay
                .as_table_mut()
                .unwrap()
                .insert(key.clone(), toml_1::Value::String(val2.clone()));

            star_toml::deep_merge(&mut base, overlay);

            let result_val = base.as_table().unwrap().get(&key).unwrap().as_str().unwrap();
            result_val == val2
        });
    }

    #[cfg(feature = "snapshot-testing")]
    #[test]
    fn test_snapshot_config() {
        use chicago_tdd_tools::testing::snapshot::SnapshotAssert;

        let loader = star_toml::trusted().layer_file("examples/star-toml/samples/default.toml");
        let admitted = loader.load_admitted::<AppConfig>().unwrap();
        let toml_string = toml_1::to_string(admitted.value()).unwrap();
        SnapshotAssert::with_settings(
            |settings| {
                settings.set_snapshot_path("../../examples/snapshots");
                settings.set_prepend_module_to_snapshot(false);
            },
            || {
                SnapshotAssert::assert_matches(
                    &toml_string,
                    "star_toml__star_toml_default_config_snapshot",
                );
            },
        );
    }

    chicago_tdd_tools::performance_test!(test_load_performance, {
        use chicago_tdd_tools::validation::performance::measure_ticks;

        let (_, ticks) = measure_ticks(|| {
            let loader = star_toml::trusted().layer_file("examples/star-toml/samples/default.toml");
            let _ = loader.load_admitted::<AppConfig>().unwrap();
        });
        assert!(ticks < 10_000_000);
    });

    #[cfg(feature = "otel")]
    #[test]
    fn test_observability() {
        use chicago_tdd_tools::observability::unified::ObservabilityTest;
        let test = ObservabilityTest::new();
        assert!(test.is_ok());
    }

    chicago_tdd_tools::test!(test_invalid_port_fails, {
        let loader =
            star_toml::trusted().layer_file("examples/star-toml/samples/invalid_port.toml");
        let result = loader.load_admitted::<AppConfig>();
        chicago_tdd_tools::assert_err!(&result);
    });
}
