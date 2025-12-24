#![allow(deprecated)]

mod common;
use anyhow::Result;
use assert_cmd::prelude::*;
use common::{get_tags, with_test_page};
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
