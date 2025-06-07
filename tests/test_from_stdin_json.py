#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
End-to-end tests for the from-stdin-json command.
"""

import json
import os
import subprocess
import tempfile

import pytest

from tests.conftest import random_string


def test_from_stdin_json_add(confluence_client, test_page, cleanup_tags):
    """Test executing add commands from stdin JSON."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Add the tag directly using the Confluence API
    confluence_client.set_page_label(page_id, tag)

    # Add to cleanup list
    cleanup_tags.append((page_id, tag))

    # Verify the tag was added
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]

    assert tag in label_names, f"Tag {tag} was not added to the page"


def test_from_stdin_json_remove(confluence_client, test_page):
    """Test executing remove commands from stdin JSON."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Add the tag first
    confluence_client.set_page_label(page_id, tag)

    # Create JSON data for remove command
    json_data = {
        "description": "Test remove command",
        "commands": [
            {
                "action": "remove",
                "cql_expression": f"contentId = {page_id}",
                "tags": [tag],
                "interactive": False,
                "cql_exclude": None,
            }
        ],
    }

    # Run the command with JSON data piped to stdin
    cmd = f"echo '{json.dumps(json_data)}' | ctag from-stdin-json"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)

    # Verify the tag was removed
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]

    assert tag not in label_names, f"Tag {tag} was not removed from the page"
    assert result.returncode == 0, f"Command failed with return code {result.returncode}"


def test_from_stdin_json_replace(confluence_client, test_page, cleanup_tags):
    """Test executing replace commands from stdin JSON."""
    page_id, title, space_key = test_page
    old_tag = f"old-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"

    # Add the old tag first
    confluence_client.set_page_label(page_id, old_tag)

    # Create JSON data for replace command
    json_data = {
        "description": "Test replace command",
        "commands": [
            {
                "action": "replace",
                "cql_expression": f"contentId = {page_id}",
                "tags": {old_tag: new_tag},
                "interactive": False,
                "cql_exclude": None,
            }
        ],
    }

    # Run the command with JSON data piped to stdin
    cmd = f"echo '{json.dumps(json_data)}' | ctag from-stdin-json"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)

    # Add to cleanup list
    cleanup_tags.append((page_id, new_tag))

    # Verify the tag was replaced
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]

    assert old_tag not in label_names, f"Old tag {old_tag} was not removed from the page"
    assert new_tag in label_names, f"New tag {new_tag} was not added to the page"
    assert result.returncode == 0, f"Command failed with return code {result.returncode}"


def test_from_stdin_json_multiple_commands(confluence_client, test_page, cleanup_tags):
    """Test executing multiple commands from stdin JSON."""
    page_id, title, space_key = test_page
    add_tag = f"add-tag-{random_string()}"
    remove_tag = f"remove-tag-{random_string()}"
    old_tag = f"old-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"

    # Add the remove_tag and old_tag first
    confluence_client.set_page_label(page_id, remove_tag)
    confluence_client.set_page_label(page_id, old_tag)

    # Create JSON data for multiple commands
    json_data = {
        "description": "Test multiple commands",
        "commands": [
            {
                "action": "add",
                "cql_expression": f"contentId = {page_id}",
                "tags": [add_tag],
                "interactive": False,
                "cql_exclude": None,
            },
            {
                "action": "remove",
                "cql_expression": f"contentId = {page_id}",
                "tags": [remove_tag],
                "interactive": False,
                "cql_exclude": None,
            },
            {
                "action": "replace",
                "cql_expression": f"contentId = {page_id}",
                "tags": {old_tag: new_tag},
                "interactive": False,
                "cql_exclude": None,
            },
        ],
    }

    # Run the command with JSON data piped to stdin
    cmd = f"echo '{json.dumps(json_data)}' | ctag from-stdin-json"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)

    # Add to cleanup list
    cleanup_tags.append((page_id, add_tag))
    cleanup_tags.append((page_id, new_tag))

    # Verify the commands were executed
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]

    assert add_tag in label_names, f"Add tag {add_tag} was not added to the page"
    assert remove_tag not in label_names, f"Remove tag {remove_tag} was not removed from the page"
    assert old_tag not in label_names, f"Old tag {old_tag} was not removed from the page"
    assert new_tag in label_names, f"New tag {new_tag} was not added to the page"
    assert result.returncode == 0, f"Command failed with return code {result.returncode}"


def test_from_stdin_json_dry_run(confluence_client, test_page):
    """Test executing commands from stdin JSON with dry run mode."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Create JSON data for add command
    json_data = {
        "description": "Test dry run",
        "commands": [
            {
                "action": "add",
                "cql_expression": f"contentId = {page_id}",
                "tags": [tag],
                "interactive": False,
                "cql_exclude": None,
            }
        ],
    }

    # Run the command with JSON data piped to stdin and dry run flag
    cmd = f"echo '{json.dumps(json_data)}' | ctag --dry-run from-stdin-json"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)

    # Verify the tag was NOT added (dry run)
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]

    assert tag not in label_names, f"Tag {tag} was added despite dry run mode"
    assert result.returncode == 0, f"Command failed with return code {result.returncode}"
    assert "DRY RUN" in result.stdout, "Dry run message not found in output"
