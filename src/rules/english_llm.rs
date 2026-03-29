use crate::cli::ZarcAuthConfig;
use crate::rules::english::EnglishPredicate;
use miette::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const ENGLISH_COMPILER_PROMPT_VERSION: u32 = 1;
pub const ENGLISH_COMPILER_RESPONSE_SCHEMA_VERSION: u32 = 1;
const ZARC_COMPILER_TIMEOUT_SECS: u64 = 20;
const ZARC_COMPILER_ENDPOINT: &str = "http://localhost:8787/v1/english/compile";

pub trait EnglishRuleCompiler: Send + Sync {
    fn compile_rule(&self, rule_text: &str) -> Result<Option<EnglishPredicate>>;
    fn fingerprint_material(&self) -> String;
}

pub fn build_compiler(auth: Option<&ZarcAuthConfig>) -> Result<Option<Box<dyn EnglishRuleCompiler>>> {
    let Some(auth) = auth else {
        return Ok(None);
    };

    Ok(Some(Box::new(ZarcBackendCompiler::new(auth)?)))
}

pub struct ZarcBackendCompiler {
    client: Client,
    api_key: String,
}

impl ZarcBackendCompiler {
    fn new(auth: &ZarcAuthConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(ZARC_COMPILER_TIMEOUT_SECS))
            .build()
            .map_err(|error| miette::miette!("Failed to build english compiler HTTP client: {}", error))?;

        Ok(Self {
            client,
            api_key: auth.api_key.clone(),
        })
    }
}

impl EnglishRuleCompiler for ZarcBackendCompiler {
    fn compile_rule(&self, rule_text: &str) -> Result<Option<EnglishPredicate>> {
        let request = HostedCompileRequest {
            rule_text: rule_text.trim().to_string(),
            prompt_version: ENGLISH_COMPILER_PROMPT_VERSION,
            response_schema_version: ENGLISH_COMPILER_RESPONSE_SCHEMA_VERSION,
        };

        let response = self
            .client
            .post(ZARC_COMPILER_ENDPOINT)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .map_err(|error| miette::miette!("English rule compiler request failed: {}", error))?
            .error_for_status()
            .map_err(|error| miette::miette!("English rule compiler returned an error: {}", error))?;

        let structured = response
            .json::<StructuredCompilerResponse>()
            .map_err(|error| miette::miette!("Failed to decode english compiler response: {}", error))?;

        match structured.outcome {
            CompileOutcome::Compiled => structured
                .predicate
                .ok_or_else(|| miette::miette!("English rule compiler omitted predicate data"))
                .map(Some),
            CompileOutcome::Unsupported => Ok(None),
        }
    }

    fn fingerprint_material(&self) -> String {
        format!(
            "provider=zarc-hosted;endpoint={};prompt={};schema={}",
            ZARC_COMPILER_ENDPOINT,
            ENGLISH_COMPILER_PROMPT_VERSION,
            ENGLISH_COMPILER_RESPONSE_SCHEMA_VERSION
        )
    }
}

#[derive(Serialize)]
struct HostedCompileRequest {
    rule_text: String,
    prompt_version: u32,
    response_schema_version: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CompileOutcome {
    Compiled,
    Unsupported,
}

#[derive(Debug, Deserialize)]
struct StructuredCompilerResponse {
    outcome: CompileOutcome,
    #[serde(default)]
    predicate: Option<EnglishPredicate>,
}
