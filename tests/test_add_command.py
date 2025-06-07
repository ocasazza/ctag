#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
End-to-end tests for the add command.
"""

import os

import pytest

from tests.conftest import random_string, run_ctag_command


def test_add_single_tag(confluence_client, test_page, cleanup_tags):
    """Test adding a single tag to a page using the CLI."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Verify the tag doesn't exist initially
    labels_before = confluence_client.get_page_labels(page_id)
    label_names_before = [label["name"] for label in labels_before.get("results", [])]
    assert tag not in label_names_before, f"Tag {tag} already exists on the page"

    # Run the add command
    cmd = f'python -m src.main add "id = {page_id}" {tag}'
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Add to cleanup list
    cleanup_tags.append((page_id, tag))

    # Verify the command succeeded
    assert returncode == 0, f"Command failed with return code {returncode}: {stderr}"

    # Verify the tag was added
    labels_after = confluence_client.get_page_labels(page_id)
    label_names_after = [label["name"] for label in labels_after.get("results", [])]

    assert tag in label_names_after, f"Tag {tag} was not added to the page"


def test_add_multiple_tags(confluence_client, test_page, cleanup_tags):
    """Test adding multiple tags to a page using the CLI."""
    page_id, title, space_key = test_page
    tags = [f"test-tag-{random_string()}" for _ in range(3)]

    # Verify the tags don't exist initially
    labels_before = confluence_client.get_page_labels(page_id)
    label_names_before = [label["name"] for label in labels_before.get("results", [])]
    for tag in tags:
        assert tag not in label_names_before, f"Tag {tag} already exists on the page"

    # Run the add command with multiple tags
    tags_str = " ".join(tags)
    cmd = f'python -m src.main add "id = {page_id}" {tags_str}'
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Add to cleanup list
    for tag in tags:
        cleanup_tags.append((page_id, tag))

    # Verify the command succeeded
    assert returncode == 0, f"Command failed with return code {returncode}: {stderr}"

    # Verify the tags were added
    labels_after = confluence_client.get_page_labels(page_id)
    label_names_after = [label["name"] for label in labels_after.get("results", [])]

    for tag in tags:
        assert tag in label_names_after, f"Tag {tag} was not added to the page"


def test_add_tag_dry_run(confluence_client, test_page):
    """Test adding a tag with dry run mode."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Verify the tag doesn't exist initially
    labels_before = confluence_client.get_page_labels(page_id)
    label_names_before = [label["name"] for label in labels_before.get("results", [])]
    assert tag not in label_names_before, f"Tag {tag} already exists on the page"

    # Run the command with dry run flag
    cmd = f'python -m src.main --dry-run add "id = {page_id}" {tag}'
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the tag was NOT added (dry run)
    labels_after = confluence_client.get_page_labels(page_id)
    label_names_after = [label["name"] for label in labels_after.get("results", [])]

    assert tag not in label_names_after, f"Tag {tag} was added despite dry run mode"
    assert returncode == 0, f"Command failed with return code {returncode}"


def test_add_tag_with_cql_exclude(confluence_client, test_page, cleanup_tags):
    """Test adding a tag with CQL exclude expression."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"
    exclude_tag = f"exclude-{random_string()}"

    # Add an exclusion tag to the page
    confluence_client.set_page_label(page_id, exclude_tag)
    cleanup_tags.append((page_id, exclude_tag))

    # Verify the tag doesn't exist initially
    labels_before = confluence_client.get_page_labels(page_id)
    label_names_before = [label["name"] for label in labels_before.get("results", [])]
    assert tag not in label_names_before, f"Tag {tag} already exists on the page"

    # Run the command with exclude
    cmd = f'python -m src.main add "id = {page_id}" {tag} --cql-exclude "label = \'{exclude_tag}\'"'
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the tag was NOT added (excluded)
    labels_after = confluence_client.get_page_labels(page_id)
    label_names_after = [label["name"] for label in labels_after.get("results", [])]

    assert tag not in label_names_after, f"Tag {tag} was added despite exclusion"
    assert returncode == 0, f"Command failed with return code {returncode}"


def test_add_tag_nonexistent_page(confluence_client):
    """Test adding a tag to a nonexistent page."""
    nonexistent_id = f"999999{random_string()}"
    tag = f"test-tag-{random_string()}"

    # Run the command
    cmd = f'python -m src.main add "id = {nonexistent_id}" {tag}'
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the command completed but found no pages
    assert returncode == 0, f"Command failed with return code {returncode}"
    assert "No pages found" in stdout, "No pages found message not found in output"
