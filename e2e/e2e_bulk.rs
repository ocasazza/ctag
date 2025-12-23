#![allow(deprecated)]

mod common;
use anyhow::Result;
use assert_cmd::prelude::*;
use common::{get_tags, with_test_page};
use serde_json::json;
use std::env;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

/// E2E flow testing `from-json` and `from-stdin-json` commands.
#[test]
#[ignore]
fn e2e_bulk_commands_flow() -> Result<()> {
    with_test_page(|cfg, page_id| {
        let cql = format!("id = {}", page_id);

        // Prepare a JSON file for `from-json`
        // We will perform:
        // 1. ADD old_tag
        // 2. CHECK if it exists
        // 3. REPLACE old -> new using `from-stdin-json`
        // 4. CHECK if new exists
        // 5. REMOVE new using `from-json`

        // 1. Run `from-json` to ADD the old tag
        let add_json_ops = json!({
            "description": "Add old tag via bulk",
            "commands": [
                {
                    "action": "add",
                    "cql_expression": cql,
                    "tags": [cfg.old_tag],
                    "interactive": false
                }
            ]
        });

        let mut temp_file = env::temp_dir();
        temp_file.push(format!("ctag_e2e_add_{}.json", page_id));
        let mut f = fs::File::create(&temp_file)?;
        f.write_all(add_json_ops.to_string().as_bytes())?;
        f.sync_all()?;
        drop(f);

        let mut from_json_cmd = Command::cargo_bin("ctag")?;
        from_json_cmd
            .arg("from-json")
            .arg(temp_file.to_str().unwrap());

        from_json_cmd.assert().success();

        // Verify add worked
        let tags = get_tags(&cql)?;
        assert!(tags.contains(&cfg.old_tag), "from-json add failed");

        // 3. Run `from-stdin-json` to REPLACE old -> new
        let replace_json_ops = json!({
            "description": "Replace old with new tag via bulk stdin",
            "commands": [
                {
                    "action": "replace",
                    "cql_expression": cql,
                    "tags": {
                        &cfg.old_tag: &cfg.new_tag
                    },
                    "interactive": false
                }
            ]
        });

        let mut from_stdin_cmd = Command::cargo_bin("ctag")?;
        let mut child = from_stdin_cmd
            .arg("from-stdin-json")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        {
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(replace_json_ops.to_string().as_bytes())?;
        }

        let output = child.wait_with_output()?;
        assert!(output.status.success(), "from-stdin-json failed");

        // Verify replace worked
        let tags = get_tags(&cql)?;
        assert!(
            !tags.contains(&cfg.old_tag),
            "from-stdin-json replace failed (old tag still matches)"
        );
        assert!(
            tags.contains(&cfg.new_tag),
            "from-stdin-json replace failed (new tag missing)"
        );

        // 5. Run `from-json` again to REMOVE new tag
        let remove_json_ops = json!({
            "description": "Remove new tag via bulk",
            "commands": [
                {
                    "action": "remove",
                    "cql_expression": cql,
                    "tags": [cfg.new_tag],
                    "interactive": false
                }
            ]
        });

        let mut temp_file_rem = env::temp_dir();
        temp_file_rem.push(format!("ctag_e2e_remove_{}.json", page_id));
        let mut f = fs::File::create(&temp_file_rem)?;
        f.write_all(remove_json_ops.to_string().as_bytes())?;
        f.sync_all()?;
        drop(f);

        let mut from_json_rem_cmd = Command::cargo_bin("ctag")?;
        from_json_rem_cmd
            .arg("from-json")
            .arg(temp_file_rem.to_str().unwrap());

        from_json_rem_cmd.assert().success();

        // Verify remove worked
        let tags = get_tags(&cql)?;
        assert!(!tags.contains(&cfg.new_tag), "from-json remove failed");

        // Cleanup temp files
        let _ = fs::remove_file(temp_file);
        let _ = fs::remove_file(temp_file_rem);

        Ok(())
    })
}
