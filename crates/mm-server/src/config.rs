use config::{Config as ConfigBuilder, ConfigError, File, FileFormat};
use mm_memory::MemoryConfig;
use mm_memory_neo4j::Neo4jConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for mm-server
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Neo4j configuration
    pub neo4j: Neo4jConfig,

    /// Memory related configuration
    pub memory: MemoryConfig,
}

impl Config {
    /// Load configuration from environment variables and specified config files
    ///
    /// # Arguments
    ///
    /// * `config_paths` - Paths to configuration files to load
    pub fn load<P: AsRef<Path>>(config_paths: &[P]) -> Result<Self, ConfigError> {
        let mut builder =
            ConfigBuilder::builder().add_source(config::Environment::with_prefix("MM"));

        // Add each config path to the builder
        for path in config_paths {
            builder = builder.add_source(File::from(path.as_ref()).required(false));
        }

        builder.build()?.try_deserialize()
    }

    /// Load configuration from a string source (useful for testing)
    ///
    /// # Arguments
    ///
    /// * `config_str` - Configuration string in TOML format
    pub fn load_from_string(config_str: &str) -> Result<Self, ConfigError> {
        let source = config::File::from_str(config_str, FileFormat::Toml);

        let config = ConfigBuilder::builder()
            .add_source(source)
            .build()?
            .try_deserialize()?;

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            neo4j: Neo4jConfig {
                uri: "neo4j://localhost:7687".to_string(),
                username: "neo4j".to_string(),
                password: "password".to_string(),
            },
            memory: MemoryConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_string() {
        let config_content = r#"
[memory]
default_label = "TestTag"
agent_name = "tester"
[neo4j]
uri = "neo4j://testhost:7687"
username = "test_user"
password = "test_password"
"#;

        // Load the config from string
        let config =
            Config::load_from_string(config_content).expect("Failed to load config from string");

        // Verify the loaded values
        assert_eq!(config.neo4j.uri, "neo4j://testhost:7687");
        assert_eq!(config.neo4j.username, "test_user");
        assert_eq!(config.neo4j.password, "test_password");
        assert_eq!(config.memory.default_label, Some("TestTag".to_string()));
        assert_eq!(config.memory.agent_name, "tester".to_string());
        assert!(config.memory.default_relationships);
    }

    #[test]
    fn test_neo4j_config_exposed() {
        let config = Config {
            neo4j: Neo4jConfig {
                uri: "neo4j://testconversion:7687".to_string(),
                username: "test_conversion_user".to_string(),
                password: "test_conversion_password".to_string(),
            },
            memory: MemoryConfig {
                default_label: None,
                default_relationships: true,
                additional_relationships: std::collections::HashSet::default(),
                default_labels: true,
                additional_labels: std::collections::HashSet::default(),
                default_project: None,
                agent_name: "test".to_string(),
            },
        };

        assert_eq!(config.neo4j.uri, "neo4j://testconversion:7687");
        assert_eq!(config.neo4j.username, "test_conversion_user");
        assert_eq!(config.neo4j.password, "test_conversion_password");
    }
}
