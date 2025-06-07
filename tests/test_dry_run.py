#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Comprehensive tests for dry-run functionality across all commands.
"""

import pytest

from tests.conftest import random_string, run_ctag_command
from tests.test_config import (
    assert_command_success,
    assert_dry_run_output,
    assert_tags_absent,
)


@pytest.mark.dry_run
@pytest.mark.cli
class TestDryRunFunctionality:
    """Test dry-run functionality for all commands."""

    def test_add_dry_run(self, confluence_client, test_page):
        """Test dry-run mode for add command."""
        page_id, title, space_key = test_page
        tag = f"test-tag-{random_string()}"

        # Run with dry-run
        cmd = f'python -m src.main --dry-run add "id = {page_id}" {tag}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # Verify command succeeded
        assert_command_success(returncode, stderr, "dry-run add")

        # Verify dry-run output
        assert_dry_run_output(stdout, "add", [tag])

        # Verify no actual changes were made
        assert_tags_absent(confluence_client, page_id, [tag])

    def test_remove_dry_run(self, confluence_client, test_page, cleanup_tags):
        """Test dry-run mode for remove command."""
        page_id, title, space_key = test_page
        tag = f"test-tag-{random_string()}"

        # First add a tag to remove
        confluence_client.set_page_label(page_id, tag)
        cleanup_tags.append((page_id, tag))

        # Run remove with dry-run
        cmd = f'python -m src.main --dry-run remove "id = {page_id}" {tag}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # Verify command succeeded
        assert_command_success(returncode, stderr, "dry-run remove")

        # Verify dry-run output
        assert_dry_run_output(stdout, "remove", [tag])

        # Verify tag is still present (not actually removed)
        labels = confluence_client.get_page_labels(page_id)
        label_names = [label["name"] for label in labels.get("results", [])]
        assert tag in label_names, f"Tag {tag} should still be present after dry-run remove"

    def test_replace_dry_run(self, confluence_client, test_page, cleanup_tags):
        """Test dry-run mode for replace command."""
        page_id, title, space_key = test_page
        old_tag = f"old-tag-{random_string()}"
        new_tag = f"new-tag-{random_string()}"

        # First add the old tag
        confluence_client.set_page_label(page_id, old_tag)
        cleanup_tags.append((page_id, old_tag))

        # Run replace with dry-run
        cmd = f'python -m src.main --dry-run replace "id = {page_id}" {old_tag}={new_tag}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # Verify command succeeded
        assert_command_success(returncode, stderr, "dry-run replace")

        # Verify dry-run output
        assert "DRY RUN" in stdout
        assert "Would replace" in stdout
        assert old_tag in stdout
        assert new_tag in stdout

        # Verify no actual changes were made
        labels = confluence_client.get_page_labels(page_id)
        label_names = [label["name"] for label in labels.get("results", [])]
        assert old_tag in label_names, f"Old tag {old_tag} should still be present after dry-run replace"
        assert new_tag not in label_names, f"New tag {new_tag} should not be present after dry-run replace"

    def test_dry_run_with_multiple_tags(self, confluence_client, test_page):
        """Test dry-run mode with multiple tags."""
        page_id, title, space_key = test_page
        tags = [f"test-tag-{random_string()}" for _ in range(3)]

        # Run with dry-run and multiple tags
        tags_str = " ".join(tags)
        cmd = f'python -m src.main --dry-run add "id = {page_id}" {tags_str}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # Verify command succeeded
        assert_command_success(returncode, stderr, "dry-run add multiple tags")

        # Verify dry-run output mentions all tags
        assert_dry_run_output(stdout, "add", tags)

        # Verify no actual changes were made
        assert_tags_absent(confluence_client, page_id, tags)

    def test_dry_run_with_nonexistent_page(self):
        """Test dry-run mode with nonexistent page."""
        nonexistent_id = f"999999{random_string()}"
        tag = f"test-tag-{random_string()}"

        # Run with dry-run on nonexistent page
        cmd = f'python -m src.main --dry-run add "id = {nonexistent_id}" {tag}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # Should succeed but find no pages
        assert_command_success(returncode, stderr, "dry-run with nonexistent page")
        assert "No pages found" in stdout

    def test_dry_run_with_cql_exclude(self, confluence_client, test_page, cleanup_tags):
        """Test dry-run mode with CQL exclude."""
        page_id, title, space_key = test_page
        tag = f"test-tag-{random_string()}"
        exclude_tag = f"exclude-{random_string()}"

        # Add exclusion tag
        confluence_client.set_page_label(page_id, exclude_tag)
        cleanup_tags.append((page_id, exclude_tag))

        # Run with dry-run and exclude
        cmd = (
            f'python -m src.main --dry-run add "id = {page_id}" {tag} --cql-exclude "label = \'{exclude_tag}\'"'
        )
        stdout, stderr, returncode = run_ctag_command(cmd)

        # Should succeed but exclude the page
        assert_command_success(returncode, stderr, "dry-run with exclude")
        # Should show that pages were excluded
        assert "Excluded" in stdout or "No pages found" in stdout or "0 pages remaining" in stdout

    def test_dry_run_combined_with_progress(self, confluence_client, test_page):
        """Test dry-run mode combined with progress option."""
        page_id, title, space_key = test_page
        tag = f"test-tag-{random_string()}"

        # Run with both dry-run and progress
        cmd = f'python -m src.main --dry-run --progress true add "id = {page_id}" {tag}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # Verify command succeeded
        assert_command_success(returncode, stderr, "dry-run with progress")

        # Verify dry-run output
        assert_dry_run_output(stdout, "add", [tag])

        # Verify no actual changes were made
        assert_tags_absent(confluence_client, page_id, [tag])


@pytest.mark.dry_run
@pytest.mark.integration
class TestDryRunIntegration:
    """Integration tests for dry-run functionality."""

    def test_dry_run_preserves_existing_tags(self, confluence_client, test_page, cleanup_tags):
        """Test that dry-run doesn't affect existing tags."""
        page_id, title, space_key = test_page
        existing_tag = f"existing-{random_string()}"
        new_tag = f"new-{random_string()}"

        # Add an existing tag
        confluence_client.set_page_label(page_id, existing_tag)
        cleanup_tags.append((page_id, existing_tag))

        # Get initial state
        labels_before = confluence_client.get_page_labels(page_id)
        label_names_before = [label["name"] for label in labels_before.get("results", [])]

        # Run dry-run add
        cmd = f'python -m src.main --dry-run add "id = {page_id}" {new_tag}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        assert_command_success(returncode, stderr, "dry-run preserving existing tags")

        # Verify state is unchanged
        labels_after = confluence_client.get_page_labels(page_id)
        label_names_after = [label["name"] for label in labels_after.get("results", [])]

        assert label_names_before == label_names_after, "Existing tags should be unchanged after dry-run"
        assert existing_tag in label_names_after, "Existing tag should still be present"
        assert new_tag not in label_names_after, "New tag should not be added in dry-run"

    def test_dry_run_output_format_consistency(self, confluence_client, test_page):
        """Test that dry-run output format is consistent across commands."""
        page_id, title, space_key = test_page
        tag = f"test-tag-{random_string()}"

        # Test add command dry-run output
        cmd = f'python -m src.main --dry-run add "id = {page_id}" {tag}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        assert_command_success(returncode, stderr, "dry-run output format test")

        # Check for consistent dry-run indicators
        assert "DRY RUN" in stdout, "Should contain 'DRY RUN' indicator"
        assert "No changes will be made" in stdout, "Should contain no changes message"
        assert "Would add" in stdout, "Should indicate what would be done"
        assert tag in stdout, "Should mention the tag"
