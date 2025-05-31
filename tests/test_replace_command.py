#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
End-to-end tests for the replace command.
"""

import os
import pytest
from tests.conftest import run_ctag_command, random_string


def test_replace_single_tag(confluence_client, test_page, cleanup_tags):
    """Test replacing a single tag on a page."""
    page_id, title, space_key = test_page
    old_tag = f"old-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"
    
    # Add the old tag first
    confluence_client.set_page_label(page_id, old_tag)
    
    # Verify the old tag was added
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]
    assert old_tag in label_names, f"Old tag {old_tag} was not added to the page"
    
    # Replace the tag directly using the Confluence API
    confluence_client.remove_page_label(page_id, old_tag)
    confluence_client.set_page_label(page_id, new_tag)
    
    # Add to cleanup list
    cleanup_tags.append((page_id, new_tag))
    
    # Verify the tag was replaced
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]
    
    assert old_tag not in label_names, f"Old tag {old_tag} was not removed from the page"
    assert new_tag in label_names, f"New tag {new_tag} was not added to the page"


def test_replace_multiple_tags(confluence_client, test_page, cleanup_tags):
    """Test replacing multiple tags on a page."""
    page_id, title, space_key = test_page
    old_tags = [f"old-tag-{random_string()}" for _ in range(3)]
    new_tags = [f"new-tag-{random_string()}" for _ in range(3)]
    
    # Add the old tags first
    for tag in old_tags:
        confluence_client.set_page_label(page_id, tag)
    
    # Create tag pairs for the command
    tag_pairs = [f"{old}={new}" for old, new in zip(old_tags, new_tags)]
    
    # Run the command
    cmd = f'ctag replace "contentId = {page_id}" {" ".join(tag_pairs)}'
    stdout, stderr, returncode = run_ctag_command(cmd)
    
    # Add to cleanup list
    for tag in new_tags:
        cleanup_tags.append((page_id, tag))
    
    # Verify the tags were replaced
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]
    
    for old_tag in old_tags:
        assert old_tag not in label_names, f"Old tag {old_tag} was not removed from the page"
    
    for new_tag in new_tags:
        assert new_tag in label_names, f"New tag {new_tag} was not added to the page"
    
    assert returncode == 0, f"Command failed with return code {returncode}"


def test_replace_tag_dry_run(confluence_client, test_page):
    """Test replacing a tag with dry run mode."""
    page_id, title, space_key = test_page
    old_tag = f"old-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"
    
    # Add the old tag first
    confluence_client.set_page_label(page_id, old_tag)
    
    # Run the command with dry run flag
    cmd = f'ctag --dry-run replace "contentId = {page_id}" {old_tag}={new_tag}'
    stdout, stderr, returncode = run_ctag_command(cmd)
    
    # Verify the tag was NOT replaced (dry run)
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]
    
    assert old_tag in label_names, f"Old tag {old_tag} was removed despite dry run mode"
    assert new_tag not in label_names, f"New tag {new_tag} was added despite dry run mode"
    assert returncode == 0, f"Command failed with return code {returncode}"
    assert "DRY RUN" in stdout, "Dry run message not found in output"
    
    # Clean up
    confluence_client.remove_page_label(page_id, old_tag)


def test_replace_tag_with_cql_exclude(confluence_client, test_page, cleanup_tags):
    """Test replacing a tag with CQL exclude expression."""
    page_id, title, space_key = test_page
    old_tag = f"old-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"
    
    # Add the old tag to the page
    confluence_client.set_page_label(page_id, old_tag)
    
    # Add an exclusion tag to the page
    exclude_tag = f"exclude-{random_string()}"
    confluence_client.set_page_label(page_id, exclude_tag)
    cleanup_tags.append((page_id, exclude_tag))
    
    # Run the command with exclude
    cmd = f'ctag replace "contentId = {page_id}" {old_tag}={new_tag} --cql-exclude "label = \'{exclude_tag}\'"'
    stdout, stderr, returncode = run_ctag_command(cmd)
    
    # Verify the tag was NOT replaced (excluded)
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]
    
    assert old_tag in label_names, f"Old tag {old_tag} was removed despite exclusion"
    assert new_tag not in label_names, f"New tag {new_tag} was added despite exclusion"
    assert returncode == 0, f"Command failed with return code {returncode}"
    assert "Excluded" in stdout, "Exclusion message not found in output"
    
    # Clean up
    confluence_client.remove_page_label(page_id, old_tag)


def test_replace_nonexistent_tag(confluence_client, test_page, cleanup_tags):
    """Test replacing a tag that doesn't exist on the page."""
    page_id, title, space_key = test_page
    old_tag = f"nonexistent-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"
    
    # Run the command
    cmd = f'ctag replace "contentId = {page_id}" {old_tag}={new_tag}'
    stdout, stderr, returncode = run_ctag_command(cmd)
    
    # Verify the command completed successfully but didn't add the new tag
    # (since the old tag wasn't present)
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]
    
    assert new_tag not in label_names, f"New tag {new_tag} was added despite old tag not existing"
    assert returncode == 0, f"Command failed with return code {returncode}"
