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

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let auth = format!("{}:{}", self.username, self.token);
        let auth_header = format!("Basic {}", BASE64.encode(auth));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_header).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    /// Execute a CQL query and return matching pages
    pub fn execute_cql_query(
        &self,
        cql_expression: &str,
        start: usize,
        limit: usize,
        expand: Option<&str>,
    ) -> Result<Vec<SearchResultItem>> {
        let expand_str = expand.unwrap_or("space,metadata.labels,version");

        let url = format!(
            "{}/wiki/rest/api/content/search?cql={}&start={}&limit={}&expand={}",
            self.base_url,
            urlencoding::encode(cql_expression),
            start,
            limit,
            expand_str
        );

        info!("Executing CQL query: {}", cql_expression);

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
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
                Ok(page) => pages.push(page),
                Err(e) => {
                    warn!("Failed to parse search result item: {}", e);
                    // Try minimal parsing
                    if let Some(obj) = item.as_object() {
                        let minimal = SearchResultItem {
                            content: obj
                                .get("content")
                                .and_then(|c| serde_json::from_value(c.clone()).ok()),
                            title: obj.get("title").and_then(|t| t.as_str()).map(String::from),
                            result_global_container: obj
                                .get("resultGlobalContainer")
                                .and_then(|r| serde_json::from_value(r.clone()).ok()),
                        };
                        pages.push(minimal);
                    }
                }
            }
        }

        info!("CQL query returned {} results", pages.len());
        Ok(pages)
    }

    /// Get all results for a CQL query, handling pagination
    pub fn get_all_cql_results(
        &self,
        cql_expression: &str,
        batch_size: usize,
    ) -> Result<Vec<SearchResultItem>> {
        let mut all_pages = Vec::new();
        let mut start = 0;

        loop {
            let batch = self.execute_cql_query(cql_expression, start, batch_size, None)?;

            if batch.is_empty() {
                break;
            }

            let batch_len = batch.len();
            all_pages.extend(batch);

            if batch_len < batch_size {
                break;
            }

            start += batch_size;
        }

        Ok(all_pages)
    }

    /// Get all tags for a specific page
    pub fn get_page_tags(&self, page_id: &str) -> Result<Vec<String>> {
        let url = format!("{}/wiki/rest/api/content/{}/label", self.base_url, page_id);

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
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
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
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
            .client
            .delete(&url)
            .headers(self.headers())
            .send()
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

// Helper function to filter excluded pages
pub fn filter_excluded_pages(
    pages: Vec<SearchResultItem>,
    excluded_pages: &[SearchResultItem],
) -> Vec<SearchResultItem> {
    let excluded_ids: Vec<String> = excluded_pages
        .iter()
        .filter_map(|p| p.content.as_ref()?.id.clone())
        .collect();

    pages
        .into_iter()
        .filter(|page| {
            if let Some(content) = &page.content {
                if let Some(id) = &content.id {
                    return !excluded_ids.contains(id);
                }
            }
            true
        })
        .collect()
}

// Helper function to sanitize text for display
pub fn sanitize_text(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Content, GlobalContainer, SearchResultItem};

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
    fn filter_excluded_pages_filters_by_content_id() {
        fn page_with_id(id: &str) -> SearchResultItem {
            SearchResultItem {
                content: Some(Content {
                    id: Some(id.to_string()),
                    content_type: Some("page".to_string()),
                    status: Some("current".to_string()),
                }),
                title: Some(format!("Page {}", id)),
                result_global_container: Some(GlobalContainer {
                    title: Some("SPACE".to_string()),
                }),
            }
        }

        let pages = vec![page_with_id("1"), page_with_id("2"), page_with_id("3")];

        let excluded = vec![page_with_id("2")];

        let filtered = filter_excluded_pages(pages, &excluded);
        let ids: Vec<_> = filtered
            .iter()
            .filter_map(|p| p.content.as_ref()?.id.as_deref())
            .collect();

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"1"));
        assert!(ids.contains(&"3"));
        assert!(!ids.contains(&"2"));
    }
}
