//! End-to-end tests that exercise the compiled `ctag` binary against a real
//! Confluence instance.
//!
//! These tests are intentionally marked `#[ignore]` so they do not run as part
//! of the default `cargo test` invocation. To run them, you must:
//!
//! 1. Create a `.sandbox.env` file in the project root with at least:
//!    - `ATLASSIAN_URL`       - Base URL for your Confluence Cloud (e.g. https://your-instance.atlassian.net)
//!    - `ATLASSIAN_USERNAME`  - Email/username for API auth
//!    - `ATLASSIAN_TOKEN`     - API token
//!    - `SANDBOX_SPACE_KEY`   - Space key where temporary test pages may be created
//!    - `SANDBOX_OLD_TAG`     - A label used as the "old" tag in replace tests
//!    - `SANDBOX_NEW_TAG`     - A label used as the "new" tag in replace tests
//!
//!    Example `.sandbox.env`:
//!    ```env
//!    ATLASSIAN_URL=https://your-instance.atlassian.net
//!    ATLASSIAN_USERNAME=you@example.com
//!    ATLASSIAN_TOKEN=your-api-token
//!    SANDBOX_SPACE_KEY=SANDBOX
//!    SANDBOX_OLD_TAG=ctag-e2e-old
//!    SANDBOX_NEW_TAG=ctag-e2e-new
//!    ```
//!
//! 2. Run the tests explicitly:
//!    ```bash
//!    cargo test --test e2e_sandbox -- --ignored
//!    ```
//!
//! Startup & cleanup behavior
//! --------------------------
//! Each test uses `with_test_page`, which:
//! - Loads `.sandbox.env`
//! - Creates a temporary Confluence page in `SANDBOX_SPACE_KEY` via the REST API
//! - Runs `ctag` commands against that specific page using a CQL like `id = <page_id>`
//! - Removes test labels from the page using the `ctag remove` CLI command
//! - Deletes the test page via the REST API at the end (best-effort)
//!
//! This ensures that each test runs against its own fresh page and leaves no
//! persistent pages or labels behind.

use anyhow::{Context, Result};
use assert_cmd::prelude::*;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use predicates::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::env;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct SandboxConfig {
    base_url: String,
    username: String,
    token: String,
    space_key: String,
    old_tag: String,
    new_tag: String,
}

impl SandboxConfig {
    fn from_env() -> Result<Self> {
        // Load .sandbox.env if present; ignore errors so CI etc. can opt out.
        let _ = dotenvy::from_filename(".sandbox.env");

        let base_url = env::var("ATLASSIAN_URL")
            .context("ATLASSIAN_URL must be set in .sandbox.env for e2e tests")?;
        let username = env::var("ATLASSIAN_USERNAME")
            .context("ATLASSIAN_USERNAME must be set in .sandbox.env for e2e tests")?;
        let token = env::var("ATLASSIAN_TOKEN")
            .context("ATLASSIAN_TOKEN must be set in .sandbox.env for e2e tests")?;
        let space_key = env::var("SANDBOX_SPACE_KEY")
            .context("SANDBOX_SPACE_KEY must be set in .sandbox.env for e2e tests")?;
        let old_tag = env::var("SANDBOX_OLD_TAG")
            .context("SANDBOX_OLD_TAG must be set in .sandbox.env for e2e tests")?;
        let new_tag = env::var("SANDBOX_NEW_TAG")
            .context("SANDBOX_NEW_TAG must be set in .sandbox.env for e2e tests")?;

        Ok(SandboxConfig {
            base_url,
            username,
            token,
            space_key,
            old_tag,
            new_tag,
        })
    }
}

struct TestConfluenceClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl TestConfluenceClient {
    fn new(cfg: &SandboxConfig) -> Result<Self> {
        let client = Client::new();
        let auth_raw = format!("{}:{}", cfg.username, cfg.token);
        let auth_header = format!("Basic {}", BASE64.encode(auth_raw.as_bytes()));
        Ok(Self {
            client,
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
            auth_header,
        })
    }

    fn headers(&self) -> Result<HeaderMap> {
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
    fn create_test_page(&self, space_key: &str) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let title = format!("ctag-e2e-test-{}", now);

        let body = json!({
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
    fn delete_page(&self, page_id: &str) -> Result<()> {
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
fn cleanup_labels_for_page(cql: &str, old_tag: &str, new_tag: &str) -> Result<()> {
    let mut cmd = Command::cargo_bin("ctag")?;
    let output = cmd
        .arg("remove")
        .arg(cql)
        .arg(old_tag)
        .arg(new_tag)
        .arg("--no-progress")
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
fn get_tags(cql: &str) -> Result<Vec<String>> {
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
fn with_test_page<F>(f: F) -> Result<()>
where
    F: FnOnce(&SandboxConfig, &str) -> Result<()>,
{
    let cfg = SandboxConfig::from_env()?;
    let client = TestConfluenceClient::new(&cfg)?;

    // Create a new test page
    let page_id = client
        .create_test_page(&cfg.space_key)
        .context("failed to create test page")?;

    // CQL that targets only this page
    let cql = format!("id = {}", page_id);

    // Ensure labels are clean before running the test
    let _ = cleanup_labels_for_page(&cql, &cfg.old_tag, &cfg.new_tag);

    // Run the actual test logic
    let result = f(&cfg, &page_id);

    // Best-effort cleanup: remove labels and delete the page
    let _ = cleanup_labels_for_page(&cql, &cfg.old_tag, &cfg.new_tag);
    let _ = client.delete_page(&page_id);

    result
}

/// Full e2e flow on a freshly-created test page:
/// 1. Ensure page is clean (no old/new tags).
/// 2. Add the old tag via `ctag add`.
/// 3. Verify the old tag appears in `ctag get ... --tags-only`.
/// 4. Replace old -> new via `ctag replace`.
/// 5. Verify only the new tag appears (and old is absent).
/// 6. Remove the new tag via `ctag remove`.
/// 7. Verify both test tags are absent again.
/// 8. Delete the test page.
#[test]
#[ignore]
fn e2e_add_replace_remove_flow_on_new_page() -> Result<()> {
    with_test_page(|cfg, page_id| {
        let cql = format!("id = {}", page_id);

        // Step 2: add the old tag
        let mut add_cmd = Command::cargo_bin("ctag")?;
        add_cmd
            .arg("add")
            .arg(&cql)
            .arg(&cfg.old_tag)
            .arg("--no-progress");

        add_cmd.assert().success().stdout(
            predicate::str::contains("Found").and(predicate::str::contains("matching pages")),
        );

        // Step 3: verify old tag present
        let tags = get_tags(&cql)?;
        assert!(
            tags.contains(&cfg.old_tag),
            "Expected old tag `{}` to be present after add; tags: {:?}",
            cfg.old_tag,
            tags
        );

        // Step 4: replace old -> new
        let mut replace_cmd = Command::cargo_bin("ctag")?;
        replace_cmd
            .arg("replace")
            .arg(&cql)
            .arg(format!("{}={}", &cfg.old_tag, &cfg.new_tag))
            .arg("--no-progress");

        replace_cmd.assert().success().stdout(
            predicate::str::contains("Found").and(predicate::str::contains("matching pages")),
        );

        // Step 5: verify only new tag present
        let tags = get_tags(&cql)?;
        assert!(
            !tags.contains(&cfg.old_tag),
            "Did not expect old tag `{}` after replace; tags: {:?}",
            cfg.old_tag,
            tags
        );
        assert!(
            tags.contains(&cfg.new_tag),
            "Expected new tag `{}` after replace; tags: {:?}",
            cfg.new_tag,
            tags
        );

        // Step 6: remove the new tag
        let mut remove_cmd = Command::cargo_bin("ctag")?;
        remove_cmd
            .arg("remove")
            .arg(&cql)
            .arg(&cfg.new_tag)
            .arg("--no-progress");

        remove_cmd.assert().success().stdout(
            predicate::str::contains("Found").and(predicate::str::contains("matching pages")),
        );

        // Step 7: verify both old/new tags are absent
        let tags = get_tags(&cql)?;
        assert!(
            !tags.contains(&cfg.old_tag),
            "Did not expect old tag `{}` after final remove; tags: {:?}",
            cfg.old_tag,
            tags
        );
        assert!(
            !tags.contains(&cfg.new_tag),
            "Did not expect new tag `{}` after final remove; tags: {:?}",
            cfg.new_tag,
            tags
        );

        Ok(())
    })
}
