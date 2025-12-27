use serde::{Deserialize, Serialize};

#[derive(clap::ValueEnum, Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    Simple,
    Verbose,
    Json,
    Csv,
}

impl OutputFormat {
    /// Check if format is verbose (detailed human-readable output)
    pub fn is_verbose(&self) -> bool {
        *self == OutputFormat::Verbose
    }

    /// Check if format is structured (JSON or CSV - machine readable)
    pub fn is_structured(&self) -> bool {
        *self == OutputFormat::Json || *self == OutputFormat::Csv
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub content: Option<Content>,
    pub title: Option<String>,
    pub space: Option<Space>,
    #[serde(rename = "resultGlobalContainer")]
    pub result_global_container: Option<GlobalContainer>,
}

impl SearchResultItem {
    pub fn space_name(&self) -> &str {
        self.content
            .as_ref()
            .and_then(|c| c.space.as_ref())
            .and_then(|s| s.name.as_deref())
            .or_else(|| self.space.as_ref().and_then(|s| s.name.as_deref()))
            .or_else(|| {
                self.result_global_container
                    .as_ref()
                    .and_then(|c| c.title.as_deref())
            })
            .unwrap_or("Unknown")
    }

    pub fn page_id(&self) -> Option<&str> {
        self.content.as_ref().and_then(|c| c.id.as_deref())
    }

    pub fn printable_clickable_title(&self, base_url: &str) -> String {
        let title = self.title.as_deref().unwrap_or("Unknown");
        let sanitized = sanitize_text(title);
        if let Some(id) = self.page_id() {
            format!(
                "\x1b]8;;{}/wiki/pages/viewpage.action?pageId={}\x1b\\{}\x1b]8;;\x1b\\",
                base_url.trim_end_matches('/'),
                id,
                sanitized
            )
        } else {
            sanitized
        }
    }
}

/// Sanitize text by decoding HTML entities and removing control characters (except whitespace)
pub fn sanitize_text(text: &str) -> String {
    // First decode HTML entities (e.g., &#128274; -> 🔒)
    let decoded = html_escape::decode_html_entities(text);
    // Then filter control characters but keep whitespace
    decoded
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ancestor {
    pub id: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub space: Option<Space>,
    #[serde(default)]
    pub ancestors: Vec<Ancestor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub id: Option<i64>,
    pub key: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalContainer {
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CqlResponse {
    pub results: Vec<serde_json::Value>,
    pub start: Option<i32>,
    pub limit: Option<i32>,
    pub size: Option<i32>,
    #[serde(rename = "totalSize")]
    pub total_size: Option<i32>,
    #[serde(rename = "_links")]
    pub links: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelsResponse {
    pub results: Vec<Label>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDetail {
    pub page_id: String,
    pub title: String,
    pub space: String,
    pub url: String,
    pub tags_added: Vec<String>,
    pub tags_removed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResults {
    pub total: usize,
    pub processed: usize,
    pub skipped: usize,
    pub success: usize,
    pub failed: usize,
    pub aborted: bool,
    #[serde(default)]
    pub tags_added: usize,
    #[serde(default)]
    pub tags_removed: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub details: Vec<ActionDetail>,
}

impl ProcessResults {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            processed: 0,
            skipped: 0,
            success: 0,
            failed: 0,
            aborted: false,
            tags_added: 0,
            tags_removed: 0,
            details: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessResults;

    #[test]
    fn process_results_new_initializes_counts_correctly() {
        let pr = ProcessResults::new(42);
        assert_eq!(pr.total, 42);
        assert_eq!(pr.processed, 0);
        assert_eq!(pr.skipped, 0);
        assert_eq!(pr.success, 0);
        assert_eq!(pr.failed, 0);
        assert!(!pr.aborted);
        assert_eq!(pr.tags_added, 0);
        assert_eq!(pr.tags_removed, 0);
    }
}
