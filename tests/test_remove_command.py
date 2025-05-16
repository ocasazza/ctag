#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
End-to-end tests for the remove command.
"""

import os
import pytest
from tests.conftest import run_ctag_command, random_string


def test_remove_single_tag(confluence_client, test_page):
    """Test removing a single tag from a page."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Add the tag first
    confluence_client.set_page_label(page_id, tag)

    # Verify the tag was added
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]
    assert tag in label_names, f"Tag {tag} was not added to the page"

    # Remove the tag directly using the Confluence API
    confluence_client.remove_page_label(page_id, tag)

    # Verify the tag was removed
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]

    assert tag not in label_names, f"Tag {tag} was not removed from the page"


def test_remove_multiple_tags(confluence_client, test_page):
    """Test removing multiple tags from a page."""
    page_id, title, space_key = test_page
    tags = [f"test-tag-{random_string()}" for _ in range(3)]

    # Add the tags first
    for tag in tags:
        confluence_client.set_page_label(page_id, tag)

    # Run the command
    cmd = f'ctag remove "contentId = {page_id}" {" ".join(tags)}'
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the tags were removed
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]

    for tag in tags:
        assert tag not in label_names, f"Tag {tag} was not removed from the page"

    assert returncode == 0, f"Command failed with return code {returncode}"


def test_remove_tag_dry_run(confluence_client, test_page):
    """Test removing a tag with dry run mode."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Add the tag first
    confluence_client.set_page_label(page_id, tag)

    # Verify the tag exists
    labels_before = confluence_client.get_page_labels(page_id)
    label_names_before = [label["name"] for label in labels_before.get("results", [])]
    assert tag in label_names_before, f"Tag {tag} was not added to the page"

    # Run the command with dry run flag
    cmd = f'ctag --dry-run remove "contentId = {page_id}" {tag}'
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the tag was NOT removed (dry run)
    labels_after = confluence_client.get_page_labels(page_id)
    label_names_after = [label["name"] for label in labels_after.get("results", [])]

    assert tag in label_names_after, f"Tag {tag} was removed despite dry run mode"
    assert returncode == 0, f"Command failed with return code {returncode}"

    # Clean up
    confluence_client.remove_page_label(page_id, tag)


def test_remove_tag_with_cql_exclude(confluence_client, test_page, cleanup_tags):
    """Test removing a tag with CQL exclude expression."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Add the tag to the page
    confluence_client.set_page_label(page_id, tag)

    # Add an exclusion tag to the page
    exclude_tag = f"exclude-{random_string()}"
    confluence_client.set_page_label(page_id, exclude_tag)
    cleanup_tags.append((page_id, exclude_tag))

    # Verify the tag exists
    labels_before = confluence_client.get_page_labels(page_id)
    label_names_before = [label["name"] for label in labels_before.get("results", [])]
    assert tag in label_names_before, f"Tag {tag} was not added to the page"

    # Run the command with exclude
    cmd = f'ctag remove "contentId = {page_id}" {tag} --cql-exclude "label = \'{exclude_tag}\'"'
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the tag was NOT removed (excluded)
    labels_after = confluence_client.get_page_labels(page_id)
    label_names_after = [label["name"] for label in labels_after.get("results", [])]

    assert tag in label_names_after, f"Tag {tag} was removed despite exclusion"
    assert returncode == 0, f"Command failed with return code {returncode}"

    # Clean up
    confluence_client.remove_page_label(page_id, tag)


def test_remove_nonexistent_tag(confluence_client, test_page):
    """Test removing a tag that doesn't exist on the page."""
    page_id, title, space_key = test_page
    tag = f"nonexistent-tag-{random_string()}"

    # Run the command
    cmd = f'ctag remove "contentId = {page_id}" {tag}'
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the command completed successfully
    assert returncode == 0, f"Command failed with return code {returncode}"
