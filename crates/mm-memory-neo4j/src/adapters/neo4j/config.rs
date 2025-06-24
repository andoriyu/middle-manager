use serde::{Deserialize, Serialize};

/// Configuration for connecting to Neo4j
#[derive(Clone, Deserialize, Serialize)]
pub struct Neo4jConfig {
    /// URI of the Neo4j server (e.g., "neo4j://localhost:7688")
    pub uri: String,

    /// Username for authentication
    pub username: String,

    /// Password for authentication
    #[serde(skip_serializing)]
    pub password: String,
}

impl std::fmt::Debug for Neo4jConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Neo4jConfig")
            .field("uri", &self.uri)
            .field("username", &self.username)
            .field("password", &"***")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Neo4jConfig;

    #[test]
    fn debug_redacts_password() {
        let cfg = Neo4jConfig {
            uri: "neo4j://localhost:7687".to_string(),
            username: "user".to_string(),
            password: "secret".to_string(),
        };

        let dbg = format!("{cfg:?}");
        assert!(!dbg.contains("secret"));
        assert!(dbg.contains("***"));
    }
}
