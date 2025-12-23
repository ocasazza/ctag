#![allow(deprecated)]

mod common;
use anyhow::Result;
use assert_cmd::prelude::*;
use common::{get_tags, with_test_page};
use predicates::prelude::*;
use std::process::Command;

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

        // add the old tag
        let mut add_cmd = Command::cargo_bin("ctag")?;
        add_cmd
            .arg("--verbose")
            .arg("add")
            .arg(&cql)
            .arg(&cfg.old_tag);

        add_cmd.assert().success().stderr(
            predicate::str::contains("Found").and(predicate::str::contains("matching pages")),
        );

        // verify old tag present
        let tags = get_tags(&cql)?;
        assert!(
            tags.contains(&cfg.old_tag),
            "Expected old tag `{}` to be present after add; tags: {:?}",
            cfg.old_tag,
            tags
        );

        // replace old -> new
        let mut replace_cmd = Command::cargo_bin("ctag")?;
        replace_cmd
            .arg("--verbose")
            .arg("replace")
            .arg(&cql)
            .arg(format!("{}={}", &cfg.old_tag, &cfg.new_tag));

        replace_cmd.assert().success().stderr(
            predicate::str::contains("Found").and(predicate::str::contains("matching pages")),
        );

        // verify only new tag present
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

        // remove the new tag
        let mut remove_cmd = Command::cargo_bin("ctag")?;
        remove_cmd
            .arg("--verbose")
            .arg("remove")
            .arg(&cql)
            .arg(&cfg.new_tag);

        remove_cmd.assert().success().stderr(
            predicate::str::contains("Found").and(predicate::str::contains("matching pages")),
        );

        // verify both old/new tags are absent
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
