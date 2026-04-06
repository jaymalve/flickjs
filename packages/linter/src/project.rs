use serde::{de::DeserializeOwned, Deserialize};
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleResolutionConfig {
    pub config_dir: PathBuf,
    pub base_url: Option<PathBuf>,
    pub paths: Vec<PathAlias>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathAlias {
    pub pattern: String,
    pub targets: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
struct TsConfigFile {
    #[serde(default, rename = "compilerOptions")]
    compiler_options: CompilerOptions,
}

#[derive(Debug, Deserialize, Default)]
struct CompilerOptions {
    #[serde(default, rename = "baseUrl")]
    base_url: Option<String>,
    #[serde(default)]
    paths: HashMap<String, Vec<String>>,
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

impl ModuleResolutionConfig {
    pub fn resolve_non_relative(&self, source: &str) -> Vec<PathBuf> {
        let mut candidates = self.resolve_paths(source);
        if candidates.is_empty() {
            if let Some(base_url) = &self.base_url {
                candidates.push(base_url.join(source));
            }
        }
        candidates
    }

    fn resolve_paths(&self, source: &str) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        let base_dir = self.base_url.as_ref().unwrap_or(&self.config_dir);

        for alias in &self.paths {
            let Some(capture) = match_path_pattern(&alias.pattern, source) else {
                continue;
            };

            for target in &alias.targets {
                let expanded = match target.split_once('*') {
                    Some((prefix, suffix)) => format!("{prefix}{capture}{suffix}"),
                    None => target.clone(),
                };
                candidates.push(base_dir.join(expanded));
            }
        }

        candidates
    }
}

pub fn load_module_resolution_config(path: &Path) -> Option<ModuleResolutionConfig> {
    let config_path = nearest_tsconfig_or_jsconfig(path)?;
    let config_dir = config_path.parent()?.to_path_buf();
    let raw = fs::read_to_string(&config_path).ok()?;
    let parsed: TsConfigFile = parse_jsonc(&raw).ok()?;

    let base_url = parsed
        .compiler_options
        .base_url
        .filter(|value| !value.trim().is_empty())
        .map(|value| config_dir.join(value));

    let mut paths: Vec<PathAlias> = parsed
        .compiler_options
        .paths
        .into_iter()
        .filter_map(|(pattern, targets)| {
            let filtered_targets: Vec<String> = targets
                .into_iter()
                .filter(|target| !target.trim().is_empty())
                .collect();
            if filtered_targets.is_empty() {
                None
            } else {
                Some(PathAlias {
                    pattern,
                    targets: filtered_targets,
                })
            }
        })
        .collect();
    paths.sort_by(|left: &PathAlias, right: &PathAlias| left.pattern.cmp(&right.pattern));

    Some(ModuleResolutionConfig {
        config_dir,
        base_url,
        paths,
    })
}

fn nearest_package_json(path: &Path) -> Option<PathBuf> {
    let mut current = search_start_dir(path)?;

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

fn nearest_tsconfig_or_jsconfig(path: &Path) -> Option<PathBuf> {
    let mut current = search_start_dir(path)?;

    loop {
        let tsconfig = current.join("tsconfig.json");
        if tsconfig.exists() {
            return Some(tsconfig);
        }

        let jsconfig = current.join("jsconfig.json");
        if jsconfig.exists() {
            return Some(jsconfig);
        }

        if !current.pop() {
            return None;
        }
    }
}

fn search_start_dir(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        path.parent().map(Path::to_path_buf)
    } else {
        Some(path.to_path_buf())
    }
}

fn match_path_pattern<'a>(pattern: &'a str, source: &'a str) -> Option<&'a str> {
    match pattern.split_once('*') {
        Some((prefix, suffix)) => source
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(suffix)),
        None => (pattern == source).then_some(""),
    }
}

fn parse_jsonc<T: DeserializeOwned>(source: &str) -> Result<T, serde_json::Error> {
    let without_comments = strip_json_comments(source);
    let normalized = strip_trailing_commas(&without_comments);
    serde_json::from_str(&normalized)
}

fn strip_json_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;

    while let Some(ch) = chars.next() {
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                result.push('\n');
            }
            continue;
        }

        if block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                block_comment = false;
            } else if ch == '\n' {
                result.push('\n');
            }
            continue;
        }

        if in_string {
            result.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            result.push(ch);
            continue;
        }

        if ch == '/' {
            match chars.peek() {
                Some('/') => {
                    chars.next();
                    line_comment = true;
                    continue;
                }
                Some('*') => {
                    chars.next();
                    block_comment = true;
                    continue;
                }
                _ => {}
            }
        }

        result.push(ch);
    }

    result
}

fn strip_trailing_commas(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < chars.len() {
        let ch = chars[index];

        if in_string {
            result.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            result.push(ch);
            index += 1;
            continue;
        }

        if ch == ',' {
            let mut lookahead = index + 1;
            while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                lookahead += 1;
            }
            if lookahead < chars.len() && matches!(chars[lookahead], '}' | ']') {
                index += 1;
                continue;
            }
        }

        result.push(ch);
        index += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{load_module_resolution_config, parse_jsonc, PathAlias, ProjectInfo};
    use serde::Deserialize;
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

    #[test]
    fn loads_nearest_module_resolution_config() {
        let dir = tempdir().unwrap();
        let app = dir.path().join("apps/web");
        fs::create_dir_all(&app).unwrap();
        fs::write(
            app.join("tsconfig.json"),
            r#"{
                "compilerOptions": {
                    "baseUrl": "./src",
                    "paths": {
                        "@/*": ["./*"],
                        "components/*": ["./components/*"]
                    }
                }
            }"#,
        )
        .unwrap();

        let nested = app.join("src/features");
        fs::create_dir_all(&nested).unwrap();

        let config = load_module_resolution_config(&nested).unwrap();
        assert_eq!(config.base_url, Some(app.join("src")));
        assert_eq!(
            config.paths,
            vec![
                PathAlias {
                    pattern: "@/*".to_string(),
                    targets: vec!["./*".to_string()],
                },
                PathAlias {
                    pattern: "components/*".to_string(),
                    targets: vec!["./components/*".to_string()],
                },
            ]
        );
    }

    #[test]
    fn parses_jsonc_configs_with_comments_and_trailing_commas() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Fixture {
            #[serde(rename = "compilerOptions")]
            compiler_options: InnerFixture,
        }

        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct InnerFixture {
            #[serde(rename = "baseUrl")]
            base_url: String,
        }

        let parsed: Fixture = parse_jsonc(
            r#"{
                // keep aliases working
                "compilerOptions": {
                    "baseUrl": "./src",
                },
            }"#,
        )
        .unwrap();

        assert_eq!(
            parsed,
            Fixture {
                compiler_options: InnerFixture {
                    base_url: "./src".to_string(),
                },
            }
        );
    }
}
