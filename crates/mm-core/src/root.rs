#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Root {
    pub name: Option<String>,
    pub uri: String,
}

impl Root {
    pub fn new(name: Option<String>, uri: String) -> Self {
        Self { name, uri }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RootCollection {
    roots: Vec<Root>,
}

impl RootCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_roots(roots: Vec<Root>) -> Self {
        Self { roots }
    }

    pub fn roots(&self) -> &[Root] {
        &self.roots
    }

    pub fn set_roots(&mut self, roots: Vec<Root>) {
        self.roots = roots;
    }

    pub fn add_root(&mut self, root: Root) {
        self.roots.push(root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_collection_basic() {
        let mut coll = RootCollection::new();
        assert!(coll.roots().is_empty());
        let r = Root::new(Some("test".to_string()), "file:///tmp".to_string());
        coll.add_root(r.clone());
        assert_eq!(coll.roots(), &[r]);
        coll.set_roots(vec![]);
        assert!(coll.roots().is_empty());
    }
}
