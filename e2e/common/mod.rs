use anyhow::{Context, Result};
use assert_cmd::prelude::*;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::env;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SandboxConfig {
    pub base_url: String,
    pub username: String,
    pub token: String,
    pub space_key: String,
    pub parent_page_id: Option<String>,
    pub old_tag: String,
    pub new_tag: String,
}

impl SandboxConfig {
    pub fn from_env() -> Result<Option<Self>> {
        // Load .env first (if present) to support standard local dev
        dotenvy::dotenv().ok();
        // Load .sandbox.env if present, overriding .env; ignore errors so CI etc. can opt out.
        let _ = dotenvy::from_filename(".sandbox.env");

        // Helper to check var and return None if missing
        let get_var = |key| -> Option<String> { env::var(key).ok() };

        let base_url = match get_var("ATLASSIAN_URL") {
            Some(v) => v,
            None => return Ok(None),
        };
        let username = match get_var("ATLASSIAN_USERNAME") {
            Some(v) => v,
            None => return Ok(None),
        };
        let token = match get_var("ATLASSIAN_TOKEN") {
            Some(v) => v,
            None => return Ok(None),
        };
        let space_key = match get_var("SANDBOX_SPACE_KEY") {
            Some(v) => v,
            None => return Ok(None),
        };
        let parent_page_id = get_var("SANDBOX_PARENT_PAGE_ID");
        let old_tag = match get_var("SANDBOX_OLD_TAG") {
            Some(v) => v,
            None => return Ok(None),
        };
        let new_tag = match get_var("SANDBOX_NEW_TAG") {
            Some(v) => v,
            None => return Ok(None),
        };

        Ok(Some(SandboxConfig {
            base_url,
            username,
            token,
            space_key,
            parent_page_id,
            old_tag,
            new_tag,
        }))
    }
}

pub struct TestConfluenceClient {
    pub client: Client,
    pub base_url: String,
    pub auth_header: String,
}

impl TestConfluenceClient {
    pub fn new(cfg: &SandboxConfig) -> Result<Self> {
        let client = Client::new();
        let auth_raw = format!("{}:{}", cfg.username, cfg.token);
        let auth_header = format!("Basic {}", BASE64.encode(auth_raw.as_bytes()));
        Ok(Self {
            client,
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
            auth_header,
        })
    }

    pub fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&self.auth_header)
                .context("failed to build Authorization header for test client")?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// Create a temporary test page in the sandbox space and return its page ID.
    pub fn create_test_page(&self, space_key: &str, parent_id: Option<&str>) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let title = format!("ctag-e2e-test-{}", now);

        let mut body = json!({
            "type": "page",
            "title": title,
            "space": { "key": space_key },
            "body": {
                "storage": {
                    "value": "<p>ctag e2e test page</p>",
                    "representation": "storage"
                }
            }
        });

        if let Some(pid) = parent_id {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("ancestors".to_string(), json!([{"id": pid}]));
            }
        }

        let url = format!("{}/wiki/rest/api/content", self.base_url,);

        let resp = self
            .client
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .context("failed to create test page via Confluence REST API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            anyhow::bail!(
                "Failed to create test page: status={} body={}",
                status,
                text
            );
        }

        let v: serde_json::Value = resp
            .json()
            .context("failed to parse create page response JSON")?;
        let id = v
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| anyhow::anyhow!("create page response did not contain string 'id'"))?;

        Ok(id.to_string())
    }

    /// Delete a page by ID (moves it to trash in Confluence).
    pub fn delete_page(&self, page_id: &str) -> Result<()> {
        let url = format!("{}/wiki/rest/api/content/{}", self.base_url, page_id);

        let resp = self
            .client
            .delete(&url)
            .headers(self.headers()?)
            .send()
            .context("failed to delete test page via Confluence REST API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            eprintln!(
                "Warning: failed to delete test page {}: status={} body={}",
                page_id, status, text
            );
        }

        Ok(())
    }
}

/// Use the CLI itself to remove labels from a specific page using a CQL like
/// `id = <page_id>`. This is idempotent.
#[allow(deprecated)]
pub fn cleanup_labels_for_page(cql: &str, old_tag: &str, new_tag: &str) -> Result<()> {
    let mut cmd = Command::cargo_bin("ctag")?;
    let output = cmd
        .arg("remove")
        .arg(cql)
        .arg(old_tag)
        .arg(new_tag)
        .arg(new_tag)
        .output()
        .context("failed to run cleanup `ctag remove` command")?;

    if !output.status.success() {
        eprintln!(
            "Warning: cleanup `ctag remove` exited with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    Ok(())
}

/// Helper: run `ctag get <CQL> --tags-only --format json` and parse the result
/// as a list of tags.
#[allow(deprecated)]
pub fn get_tags(cql: &str) -> Result<Vec<String>> {
    let mut cmd = Command::cargo_bin("ctag")?;
    let output = cmd
        .arg("get")
        .arg(cql)
        .arg("--tags-only")
        .arg("--format")
        .arg("json")
        .output()
        .context("failed to run `ctag get` command")?;

    if !output.status.success() {
        anyhow::bail!(
            "`ctag get` failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tags: Vec<String> =
        serde_json::from_str(&stdout).context("failed to parse JSON output from `ctag get`")?;
    Ok(tags)
}

/// Run a function with a freshly-created test page, guaranteeing best-effort
/// cleanup of labels and deletion of the page.
pub fn with_test_page<F>(f: F) -> Result<()>
where
    F: FnOnce(&SandboxConfig, &str) -> Result<()>,
{
    let cfg = match SandboxConfig::from_env()? {
        Some(c) => c,
        None => {
            println!("Skipping E2E test: Missing environment variables.");
            return Ok(());
        }
    };
    let client = TestConfluenceClient::new(&cfg)?;

    // Create a new test page
    let page_id = client
        .create_test_page(&cfg.space_key, cfg.parent_page_id.as_deref())
        .context("failed to create test page")?;

    // CQL that targets only this page
    let cql = format!("id = {}", page_id);

    // Wait for Confluence search index to catch up
    std::thread::sleep(std::time::Duration::from_secs(15));

    // Ensure labels are clean before running the test
    let _ = cleanup_labels_for_page(&cql, &cfg.old_tag, &cfg.new_tag);

    // Run the actual test logic, catching any panic to ensure cleanup happens
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&cfg, &page_id)));

    // Best-effort cleanup: remove labels and delete the page
    let _ = cleanup_labels_for_page(&cql, &cfg.old_tag, &cfg.new_tag);
    let _ = client.delete_page(&page_id);

    // If the test panicked, resume unwinding; otherwise return the Result
    match result {
        Ok(r) => r,
        Err(e) => std::panic::resume_unwind(e),
    }
}
