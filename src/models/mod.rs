use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub content: Option<Content>,
    pub title: Option<String>,
    #[serde(rename = "resultGlobalContainer")]
    pub result_global_container: Option<GlobalContainer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub status: Option<String>,
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

#[derive(Debug, Clone)]
pub struct ProcessResults {
    pub total: usize,
    pub processed: usize,
    pub skipped: usize,
    pub success: usize,
    pub failed: usize,
    pub aborted: bool,
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
    }
}
