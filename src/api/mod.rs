use anyhow::{Context, Result};
use log::{error, info, warn};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::collections::HashMap;

use crate::models::{CqlResponse, LabelsResponse, SearchResultItem};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

pub struct ConfluenceClient {
    client: Client,
    base_url: String,
    username: String,
    token: String,
}

impl ConfluenceClient {
    pub fn new(base_url: String, username: String, token: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            username,
            token,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let auth = format!("{}:{}", self.username, self.token);
        let auth_header = format!("Basic {}", BASE64.encode(auth));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_header).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    fn send_request<F>(&self, build_request: F) -> Result<reqwest::blocking::Response>
    where
        F: Fn() -> reqwest::blocking::RequestBuilder,
    {
        const MAX_RETRIES: u32 = 5;
        let mut attempt = 0;
        let mut delay = std::time::Duration::from_secs(1);

        loop {
            attempt += 1;
            let request = build_request();
            match request.send() {
                Ok(response) => {
                    let status = response.status();
                    if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS
                    {
                        if attempt > MAX_RETRIES {
                            return Ok(response);
                        }
                        let mut wait_duration = delay;
                        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            if let Some(retry_after) =
                                response.headers().get(reqwest::header::RETRY_AFTER)
                            {
                                if let Ok(retry_str) = retry_after.to_str() {
                                    if let Ok(seconds) = retry_str.parse::<u64>() {
                                        wait_duration = std::time::Duration::from_secs(seconds);
                                    }
                                }
                            }
                        }
                        // Add jitter
                        let jitter_ms = fastrand::u64(..1000);
                        wait_duration += std::time::Duration::from_millis(jitter_ms);
                        warn!(
                            "Request failed with status {}, retrying in {:?} (attempt {}/{})",
                            status, wait_duration, attempt, MAX_RETRIES
                        );
                        std::thread::sleep(wait_duration);
                        delay = std::cmp::min(delay * 2, std::time::Duration::from_secs(30));
                        continue;
                    } else {
                        return Ok(response);
                    }
                }
                Err(e) => {
                    if attempt > MAX_RETRIES {
                        return Err(e.into());
                    }
                    let jitter_ms = fastrand::u64(..1000);
                    let wait_duration = delay + std::time::Duration::from_millis(jitter_ms);
                    warn!(
                        "Request failed: {}, retrying in {:?} (attempt {}/{})",
                        e, wait_duration, attempt, MAX_RETRIES
                    );
                    std::thread::sleep(wait_duration);
                    delay = std::cmp::min(delay * 2, std::time::Duration::from_secs(30));
                }
            }
        }
    }

    /// Execute a CQL query and return matching pages
    /// Returns (pages, next_url) where next_url is the cursor-based URL for the next page
    pub fn execute_cql_query(
        &self,
        cql_expression: &str,
        limit: usize,
        next_url: Option<&str>,
    ) -> Result<(Vec<SearchResultItem>, Option<String>)> {
        let _expand_str = "content.space,content.metadata.labels,content.version";

        // If we have a next_url, use it directly; otherwise build the initial URL
        let url = if let Some(next) = next_url {
            format!("{}/wiki{}", self.base_url, next)
        } else {
            format!(
                "{}/wiki/rest/api/search?cql={}&limit={}&expand=content.space,content.metadata.labels,content.version",
                self.base_url,
                urlencoding::encode(cql_expression),
                limit
            )
        };

        info!("Executing CQL query: {} (limit: {})", cql_expression, limit);
        let response = self
            .send_request(|| self.client.get(&url).headers(self.headers()))
            .context("Failed to execute CQL query")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!("CQL query failed with status {}: {}", status, error_text);
        }
        let cql_response: CqlResponse = response.json().context("Failed to parse CQL response")?;
        let mut pages = Vec::new();
        for item in cql_response.results {
            match serde_json::from_value::<SearchResultItem>(item.clone()) {
                Ok(mut page) => {
                    if page.content.is_none() {
                        if let Ok(c) =
                            serde_json::from_value::<crate::models::Content>(item.clone())
                        {
                            page.content = Some(c);
                        }
                    }
                    pages.push(page);
                }
                Err(e) => {
                    warn!("Failed to parse search result item: {}", e);
                    if let Ok(c) = serde_json::from_value::<crate::models::Content>(item.clone()) {
                        let minimal = SearchResultItem {
                            title: c.title.clone(),
                            content: Some(c),
                            space: None,
                            result_global_container: None,
                        };
                        pages.push(minimal);
                    }
                }
            }
        }
        let result_count = pages.len();

        // Extract the next link for cursor-based pagination
        let next_link = cql_response
            .links
            .as_ref()
            .and_then(|links| links.get("next"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        info!(
            "CQL query returned {} results (totalSize: {:?}, has_next: {})",
            result_count,
            cql_response.total_size,
            next_link.is_some()
        );
        Ok((pages, next_link))
    }

    /// Get all results for a CQL query, handling pagination
    /// Optional callback receives (current_count, batch_size) after each batch
    pub fn get_all_cql_results(
        &self,
        cql_expression: &str,
        batch_size: usize,
    ) -> Result<Vec<SearchResultItem>> {
        self.get_all_cql_results_with_progress(cql_expression, batch_size, None::<fn(usize, usize)>)
    }

    /// Get all results for a CQL query with progress callback
    pub fn get_all_cql_results_with_progress<F>(
        &self,
        cql_expression: &str,
        batch_size: usize,
        mut progress_callback: Option<F>,
    ) -> Result<Vec<SearchResultItem>>
    where
        F: FnMut(usize, usize),
    {
        let mut all_pages = Vec::new();
        let mut next_url: Option<String> = None;

        loop {
            let (batch, next) =
                self.execute_cql_query(cql_expression, batch_size, next_url.as_deref())?;

            if batch.is_empty() {
                break;
            }

            let batch_len = batch.len();
            all_pages.extend(batch);

            // Call progress callback with current total
            if let Some(ref mut callback) = progress_callback {
                callback(all_pages.len(), batch_len);
            }

            // Break if no more pages
            if next.is_none() {
                break;
            }

            next_url = next;
        }

        Ok(all_pages)
    }

    /// Get all tags for a specific page
    pub fn get_page_tags(&self, page_id: &str) -> Result<Vec<String>> {
        let url = format!("{}/wiki/rest/api/content/{}/label", self.base_url, page_id);

        let response = self
            .send_request(|| self.client.get(&url).headers(self.headers()))
            .context("Failed to get page labels")?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let labels_response: LabelsResponse =
            response.json().context("Failed to parse labels response")?;

        Ok(labels_response
            .results
            .into_iter()
            .map(|l| l.name)
            .collect())
    }

    /// Add a tag to a Confluence page
    pub fn add_tag(&self, page_id: &str, tag: &str) -> Result<()> {
        let url = format!("{}/wiki/rest/api/content/{}/label", self.base_url, page_id);

        let body = json!([{"name": tag}]);

        let response = self
            .send_request(|| self.client.post(&url).headers(self.headers()).json(&body))
            .context("Failed to add tag")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!(
                "Failed to add tag '{}' to page {}: {} - {}",
                tag,
                page_id,
                status,
                error_text
            );
        }

        info!("Added tag '{}' to page {}", tag, page_id);
        Ok(())
    }

    /// Remove a tag from a Confluence page
    pub fn remove_tag(&self, page_id: &str, tag: &str) -> Result<()> {
        let url = format!(
            "{}/wiki/rest/api/content/{}/label?name={}",
            self.base_url,
            page_id,
            urlencoding::encode(tag)
        );

        let response = self
            .send_request(|| self.client.delete(&url).headers(self.headers()))
            .context("Failed to remove tag")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!(
                "Failed to remove tag '{}' from page {}: {} - {}",
                tag,
                page_id,
                status,
                error_text
            );
        }

        info!("Removed tag '{}' from page {}", tag, page_id);
        Ok(())
    }

    /// Add multiple tags to a page
    pub fn add_tags(&self, page_id: &str, tags: &[String]) -> bool {
        let mut success = true;
        for tag in tags {
            if let Err(e) = self.add_tag(page_id, tag) {
                error!("Error adding tag '{}' to page {}: {}", tag, page_id, e);
                success = false;
            }
        }
        success
    }

    /// Remove multiple tags from a page
    pub fn remove_tags(&self, page_id: &str, tags: &[String]) -> bool {
        let mut success = true;
        for tag in tags {
            if let Err(e) = self.remove_tag(page_id, tag) {
                error!("Error removing tag '{}' from page {}: {}", tag, page_id, e);
                success = false;
            }
        }
        success
    }

    /// Replace tags on a page
    pub fn replace_tags(&self, page_id: &str, tag_mapping: &HashMap<String, String>) -> bool {
        let current_tags = match self.get_page_tags(page_id) {
            Ok(tags) => tags,
            Err(e) => {
                error!("Failed to get current tags for page {}: {}", page_id, e);
                return false;
            }
        };

        let mut success = true;
        for (old_tag, new_tag) in tag_mapping {
            if current_tags.contains(old_tag) {
                if let Err(e) = self.remove_tag(page_id, old_tag) {
                    error!(
                        "Error removing tag '{}' from page {}: {}",
                        old_tag, page_id, e
                    );
                    success = false;
                    continue;
                }
                if let Err(e) = self.add_tag(page_id, new_tag) {
                    error!("Error adding tag '{}' to page {}: {}", new_tag, page_id, e);
                    success = false;
                } else {
                    info!(
                        "Replaced tag '{}' with '{}' on page {}",
                        old_tag, new_tag, page_id
                    );
                }
            }
        }
        success
    }
}

pub use crate::models::sanitize_text;

/// Filter tags that match any of the provided regexes
pub fn filter_tags_by_regex(tags: Vec<String>, regexes: &[regex::Regex]) -> Vec<String> {
    tags.into_iter()
        .filter(|tag| regexes.iter().any(|re| re.is_match(tag)))
        .collect()
}

/// Compute a mapping of old tags to new tags based on regex matches
pub fn compute_replacements_by_regex(
    tags: Vec<String>,
    regex_pairs: &[(regex::Regex, String)],
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for tag in tags {
        for (re, new_tag) in regex_pairs {
            if re.is_match(&tag) {
                map.insert(tag, new_tag.clone());
                break;
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_text_removes_control_chars_but_keeps_whitespace() {
        let input = "Hello\u{7} World\nNext\tLine";
        let output = sanitize_text(input);

        // Bell (\u{7}) should be removed, but spaces, newline and tab remain
        assert!(!output.contains('\u{7}'));
        assert!(output.contains(' '));
        assert!(output.contains('\n'));
        assert!(output.contains('\t'));
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
    }

    #[test]
    fn sanitize_text_decodes_html_entities() {
        // Test HTML entity decoding (Confluence may return these)
        let input = "Lock &#128274; Page"; // &#128274; is the lock emoji 🔒
        let output = sanitize_text(input);
        assert!(
            output.contains("🔒"),
            "Expected emoji in output: {}",
            output
        );
        // Test named entities
        let input2 = "Hello &amp; World";
        let output2 = sanitize_text(input2);
        assert!(output2.contains("&"), "Expected & in output: {}", output2);
    }

    #[test]
    fn filter_tags_by_regex_works() {
        let tags = vec![
            "test-1".to_string(),
            "test-2".to_string(),
            "other".to_string(),
            "TEST-3".to_string(),
        ];
        // Test multiple regexes and case sensitivity
        let regexes = vec![
            regex::Regex::new("test-.*").unwrap(),
            regex::Regex::new("^other$").unwrap(),
        ];
        let filtered = filter_tags_by_regex(tags, &regexes);
        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains(&"test-1".to_string()));
        assert!(filtered.contains(&"test-2".to_string()));
        assert!(filtered.contains(&"other".to_string()));
        assert!(!filtered.contains(&"TEST-3".to_string())); // Case sensitive
    }

    #[test]
    fn filter_tags_by_regex_empty() {
        let tags = vec!["a".into(), "b".into()];
        let regexes = vec![regex::Regex::new("z").unwrap()];
        let filtered = filter_tags_by_regex(tags, &regexes);
        assert!(filtered.is_empty());
    }

    #[test]
    fn compute_replacements_by_regex_works() {
        let tags = vec![
            "id-123".to_string(),
            "id-456".to_string(),
            "other".to_string(),
            "special-1".to_string(),
        ];
        let regex_pairs = vec![
            (
                regex::Regex::new("id-.*").unwrap(),
                "matched-id".to_string(),
            ),
            (
                regex::Regex::new("special-.*").unwrap(),
                "matched-special".to_string(),
            ),
        ];
        let replacements = compute_replacements_by_regex(tags, &regex_pairs);
        assert_eq!(replacements.len(), 3);
        assert_eq!(replacements.get("id-123"), Some(&"matched-id".to_string()));
        assert_eq!(replacements.get("id-456"), Some(&"matched-id".to_string()));
        assert_eq!(
            replacements.get("special-1"),
            Some(&"matched-special".to_string())
        );
        assert!(!replacements.contains_key("other"));
    }

    #[test]
    fn compute_replacements_by_regex_priority() {
        let tags = vec!["match-both".to_string()];
        // First match wins
        let regex_pairs = vec![
            (regex::Regex::new("match-.*").unwrap(), "first".to_string()),
            (regex::Regex::new(".*-both").unwrap(), "second".to_string()),
        ];
        let replacements = compute_replacements_by_regex(tags, &regex_pairs);
        assert_eq!(replacements.get("match-both"), Some(&"first".to_string()));
    }
}
