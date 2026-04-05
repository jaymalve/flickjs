use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectInfo {
    pub has_react: bool,
    pub has_next: bool,
    pub has_expo: bool,
    pub has_react_native: bool,
    pub has_express: bool,
    pub has_fastify: bool,
    pub has_hono: bool,
    pub has_koa: bool,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: HashMap<String, serde_json::Value>,
}

impl ProjectInfo {
    pub fn detect(root: &Path) -> Self {
        let package_json = match nearest_package_json(root) {
            Some(path) => path,
            None => return Self::default(),
        };

        let raw = match fs::read_to_string(&package_json) {
            Ok(raw) => raw,
            Err(_) => return Self::default(),
        };
        let package_json: PackageJson = match serde_json::from_str(&raw) {
            Ok(package_json) => package_json,
            Err(_) => return Self::default(),
        };

        let has_dep = |name: &str| {
            package_json.dependencies.contains_key(name)
                || package_json.dev_dependencies.contains_key(name)
        };

        let has_next = has_dep("next");
        let has_expo = has_dep("expo");
        let has_react_native = has_dep("react-native");

        Self {
            has_react: has_dep("react")
                || has_dep("react-dom")
                || has_next
                || has_expo
                || has_react_native,
            has_next,
            has_expo,
            has_react_native,
            has_express: has_dep("express"),
            has_fastify: has_dep("fastify"),
            has_hono: has_dep("hono"),
            has_koa: has_dep("koa"),
        }
    }

    pub fn has_server_framework(&self) -> bool {
        self.has_express || self.has_fastify || self.has_hono || self.has_koa
    }

    pub fn fingerprint(&self) -> String {
        let value = format!(
            "has_express={};has_expo={};has_fastify={};has_hono={};has_koa={};has_next={};has_react={};has_react_native={}",
            self.has_express,
            self.has_expo,
            self.has_fastify,
            self.has_hono,
            self.has_koa,
            self.has_next,
            self.has_react,
            self.has_react_native,
        );
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    #[cfg(test)]
    pub fn test_react() -> Self {
        Self {
            has_react: true,
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub fn test_server() -> Self {
        Self {
            has_express: true,
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub fn test_all() -> Self {
        Self {
            has_react: true,
            has_next: true,
            has_expo: true,
            has_react_native: true,
            has_express: true,
            has_fastify: true,
            has_hono: true,
            has_koa: true,
        }
    }
}

fn nearest_package_json(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_file() {
        path.parent().map(Path::to_path_buf)?
    } else {
        path.to_path_buf()
    };

    loop {
        let candidate = current.join("package.json");
        if candidate.exists() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectInfo;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_nearest_package_json_only() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"express":"^1.0.0"}}"#,
        )
        .unwrap();

        let app = dir.path().join("apps/web");
        fs::create_dir_all(&app).unwrap();
        fs::write(
            app.join("package.json"),
            r#"{"dependencies":{"react":"^18.0.0","react-dom":"^18.0.0"}}"#,
        )
        .unwrap();

        let project = ProjectInfo::detect(&app);
        assert!(project.has_react);
        assert!(!project.has_express);
    }

    #[test]
    fn detects_dev_dependencies() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"devDependencies":{"next":"^15.0.0"}}"#,
        )
        .unwrap();

        let project = ProjectInfo::detect(dir.path());
        assert!(project.has_next);
        assert!(project.has_react);
    }

    #[test]
    fn returns_default_when_package_json_is_missing() {
        let dir = tempdir().unwrap();
        let project = ProjectInfo::detect(dir.path());
        assert_eq!(project, ProjectInfo::default());
    }

    #[test]
    fn fingerprint_changes_with_flags() {
        let react = ProjectInfo::test_react();
        let server = ProjectInfo::test_server();
        assert_ne!(react.fingerprint(), server.fingerprint());
    }
}
