use mm_core::Root;

/// Convert an MCP SDK [`Root`] into a domain [`Root`].
pub fn from_sdk_root(root: rust_mcp_sdk::schema::Root) -> Root {
    Root {
        name: root.name,
        uri: root.uri,
    }
}
