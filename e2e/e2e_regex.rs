#![allow(deprecated)]

mod common;
use anyhow::Result;
use assert_cmd::prelude::*;
use common::{get_tags, with_test_page};
use predicates::prelude::*;
use std::process::Command;

#[test]
#[ignore]
fn e2e_regex_remove_verification() -> Result<()> {
    with_test_page(|_cfg, page_id| {
        let cql = format!("id = {}", page_id);

        // 1. Add some specific tags
        let mut add_cmd = Command::cargo_bin("ctag")?;
        add_cmd
            .arg("add")
            .arg(&cql)
            .arg("test-tag-1")
            .arg("test-tag-2")
            .arg("other-tag")
            .assert()
            .success();

        // 2. Dry run check
        let mut dry_remove_cmd = Command::cargo_bin("ctag")?;
        dry_remove_cmd
            .arg("--dry-run")
            .arg("--verbose")
            .arg("remove")
            .arg(&cql)
            .arg("test-tag-.*")
            .arg("--regex")
            .assert()
            .success()
            .stderr(
                predicate::str::contains("Would remove tags")
                    .and(predicate::str::contains("[\"test-tag-1\", \"test-tag-2\"]")),
            );

        // 3. Remove using regex
        let mut remove_cmd = Command::cargo_bin("ctag")?;
        remove_cmd
            .arg("remove")
            .arg(&cql)
            .arg("test-tag-.*")
            .arg("--regex")
            .arg("--verbose")
            .assert()
            .success();

        // 4. Verify specific tags are gone but others remain
        let tags = get_tags(&cql)?;
        assert!(!tags.contains(&"test-tag-1".to_string()));
        assert!(!tags.contains(&"test-tag-2".to_string()));
        assert!(tags.contains(&"other-tag".to_string()));

        Ok(())
    })
}

#[test]
#[ignore]
fn e2e_regex_replace_verification() -> Result<()> {
    with_test_page(|_, page_id| {
        let cql = format!("id = {}", page_id);

        // 1. Add some specific tags
        let mut add_cmd = Command::cargo_bin("ctag")?;
        add_cmd
            .arg("add")
            .arg(&cql)
            .arg("id-123")
            .arg("id-456")
            .arg("unrelated")
            .assert()
            .success();

        // 2. Replace using regex
        let mut replace_cmd = Command::cargo_bin("ctag")?;
        replace_cmd
            .arg("replace")
            .arg(&cql)
            .arg("id-.*=matched-id")
            .arg("--regex")
            .arg("--verbose")
            .assert()
            .success();

        // 3. Verify replacements
        let tags = get_tags(&cql)?;
        assert!(!tags.contains(&"id-123".to_string()));
        assert!(!tags.contains(&"id-456".to_string()));
        assert!(tags.contains(&"matched-id".to_string()));
        assert!(tags.contains(&"unrelated".to_string()));

        Ok(())
    })
}

#[test]
#[ignore]
fn e2e_regex_from_json_verification() -> Result<()> {
    with_test_page(|_, page_id| {
        let cql = format!("id = {}", page_id);

        // 1. Add tags
        let mut add_cmd = Command::cargo_bin("ctag")?;
        add_cmd
            .arg("add")
            .arg(&cql)
            .arg("foo-1")
            .arg("bar-1")
            .assert()
            .success();

        // 2. Create JSON file
        let json_file = "e2e_regex_test.json";
        let json_content = serde_json::json!({
            "commands": [
                {
                    "action": "replace",
                    "cql_expression": cql,
                    "tags": { "foo-.*": "replaced-foo" },
                    "regex": true
                },
                {
                    "action": "remove",
                    "cql_expression": cql,
                    "tags": ["bar-.*"],
                    "regex": true
                }
            ]
        });
        std::fs::write(json_file, serde_json::to_string(&json_content)?)?;

        // 3. Run from-json
        let mut from_json_cmd = Command::cargo_bin("ctag")?;
        from_json_cmd
            .arg("from-json")
            .arg(json_file)
            .arg("--verbose")
            .assert()
            .success();

        // 4. Verify results
        let tags = get_tags(&cql)?;
        assert!(tags.contains(&"replaced-foo".to_string()));
        assert!(!tags.contains(&"foo-1".to_string()));
        assert!(!tags.contains(&"bar-1".to_string()));

        // Cleanup
        let _ = std::fs::remove_file(json_file);

        Ok(())
    })
}
