#![allow(deprecated)]

mod common;
use anyhow::Result;
use assert_cmd::prelude::*;
use common::{get_tags, with_test_page, SandboxConfig, TestConfluenceClient};
use predicates::prelude::*;
use std::process::Command;

#[test]
#[ignore]
fn e2e_json_and_csv_summary_verification() -> Result<()> {
    with_test_page(|cfg, page_id| {
        let cql = format!("id = {}", page_id);

        // Verify JSON summary on ADD
        let mut add_json_cmd = Command::cargo_bin("ctag")?;
        add_json_cmd
            .arg("--format")
            .arg("json")
            .arg("add")
            .arg(&cql)
            .arg(&cfg.old_tag);

        let output = add_json_cmd.output()?;
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout)?;
        let json: serde_json::Value = serde_json::from_str(&stdout)?;
        assert_eq!(json["processed"], 1);
        assert_eq!(json["success"], 1);

        // Verify it actually worked
        assert!(get_tags(&cql)?.contains(&cfg.old_tag));

        // Verify CSV summary on REMOVE
        let mut remove_csv_cmd = Command::cargo_bin("ctag")?;
        remove_csv_cmd
            .arg("--format")
            .arg("csv")
            .arg("remove")
            .arg(&cql)
            .arg(&cfg.old_tag);

        let output = remove_csv_cmd.output()?;
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout)?;
        // CSV should contain total,processed,skipped,success,failed,aborted
        assert!(stdout.contains("1,1,0,1,0,false"));

        // Verify it actually worked
        assert!(!get_tags(&cql)?.contains(&cfg.old_tag));

        Ok(())
    })
}

#[test]
#[ignore]
fn e2e_get_format_verification() -> Result<()> {
    with_test_page(|cfg, page_id| {
        let cql = format!("id = {}", page_id);
        // Add a tag first
        let mut add_cmd = Command::cargo_bin("ctag")?;
        add_cmd
            .arg("add")
            .arg(&cql)
            .arg(&cfg.old_tag)
            .assert()
            .success();
        // Verify GET CSV
        let mut get_csv_cmd = Command::cargo_bin("ctag")?;
        get_csv_cmd.arg("get").arg(&cql).arg("--format").arg("csv");
        let output = get_csv_cmd.output()?;
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout)?;
        // ID,Title,Space,Tags
        assert!(stdout.contains(page_id));
        assert!(stdout.contains(&cfg.old_tag));
        // Verify GET JSON (full data)
        let mut get_json_cmd = Command::cargo_bin("ctag")?;
        get_json_cmd
            .arg("get")
            .arg(&cql)
            .arg("--format")
            .arg("json");

        let output = get_json_cmd.output()?;
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout)?;
        let json: serde_json::Value = serde_json::from_str(&stdout)?;
        assert!(json.is_array());
        assert_eq!(json[0]["id"], page_id);
        assert!(json[0]["tags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t == &cfg.old_tag));

        Ok(())
    })
}

#[test]
#[ignore]
fn e2e_dry_run_verification() -> Result<()> {
    with_test_page(|cfg, page_id| {
        let cql = format!("id = {}", page_id);

        // ADD with dry-run
        let mut dry_add_cmd = Command::cargo_bin("ctag")?;
        dry_add_cmd
            .arg("--dry-run")
            .arg("--verbose")
            .arg("add")
            .arg(&cql)
            .arg(&cfg.old_tag);

        dry_add_cmd.assert().success().stderr(
            predicate::str::contains("Would add tags").and(predicate::str::contains(&cfg.old_tag)),
        );

        // Verify NO tag was added
        assert!(!get_tags(&cql)?.contains(&cfg.old_tag));

        // Add for real
        let mut add_cmd = Command::cargo_bin("ctag")?;
        add_cmd
            .arg("add")
            .arg(&cql)
            .arg(&cfg.old_tag)
            .assert()
            .success();

        // REPLACE with dry-run
        let mut dry_replace_cmd = Command::cargo_bin("ctag")?;
        dry_replace_cmd
            .arg("--dry-run")
            .arg("--verbose")
            .arg("replace")
            .arg(&cql)
            .arg(format!("{}={}", &cfg.old_tag, &cfg.new_tag));

        dry_replace_cmd.assert().success().stderr(
            predicate::str::contains("Would replace tags")
                .and(predicate::str::contains(&cfg.new_tag)),
        );

        // Verify replace DID NOT happen
        let tags = get_tags(&cql)?;
        assert!(tags.contains(&cfg.old_tag));
        assert!(!tags.contains(&cfg.new_tag));

        Ok(())
    })
}

#[test]
#[ignore]
fn e2e_exclusion_and_multi_page_verification() -> Result<()> {
    let cfg = match SandboxConfig::from_env()? {
        Some(c) => c,
        None => return Ok(()),
    };
    let client = TestConfluenceClient::new(&cfg)?;

    // Create TWO pages
    let page_id_1 = client.create_test_page(&cfg.space_key, cfg.parent_page_id.as_deref())?;
    let page_id_2 = client.create_test_page(&cfg.space_key, cfg.parent_page_id.as_deref())?;

    let cql_both = format!("id in ({}, {})", page_id_1, page_id_2);
    let cql_1 = format!("id = {}", page_id_1);
    let cql_2 = format!("id = {}", page_id_2);

    // Wait for index
    std::thread::sleep(std::time::Duration::from_secs(15));

    // Add tag to BOTH
    let mut add_cmd = Command::cargo_bin("ctag")?;
    add_cmd
        .arg("add")
        .arg(&cql_both)
        .arg(&cfg.old_tag)
        .assert()
        .success();

    assert!(get_tags(&cql_1)?.contains(&cfg.old_tag));
    assert!(get_tags(&cql_2)?.contains(&cfg.old_tag));

    // Remove tag from BOTH BUT EXCLUDE page 2
    let mut remove_cmd = Command::cargo_bin("ctag")?;
    remove_cmd
        .arg("remove")
        .arg(&cql_both)
        .arg(&cfg.old_tag)
        .arg("--cql-exclude")
        .arg(&cql_2)
        .assert()
        .success();

    // Page 1 should be clean
    assert!(!get_tags(&cql_1)?.contains(&cfg.old_tag));
    // Page 2 should STILL have the tag
    assert!(get_tags(&cql_2)?.contains(&cfg.old_tag));

    // Cleanup
    let _ = client.delete_page(&page_id_1);
    let _ = client.delete_page(&page_id_2);

    Ok(())
}
