#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
End-to-end tests for the from-json command.
"""

import os
import json
import tempfile
import pytest
from tests.conftest import run_ctag_command, random_string


def test_from_json_add(confluence_client, test_page, cleanup_tags):
    """Test executing add commands from a JSON file."""
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


def test_from_json_remove(confluence_client, test_page):
    """Test executing remove commands from a JSON file."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"
    
    # Add the tag first
    confluence_client.set_page_label(page_id, tag)
    
    # Create a temporary JSON file with remove command
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json_file = f.name
        json.dump({
            "description": "Test remove command",
            "commands": [
                {
                    "action": "remove",
                    "cql_expression": f"contentId = {page_id}",
                    "tags": [tag],
                    "interactive": False,
                    "cql_exclude": None
                }
            ]
        }, f)
    
    try:
        # Run the command
        cmd = f'ctag from-json {json_file}'
        stdout, stderr, returncode = run_ctag_command(cmd)
        
        # Verify the tag was removed
        labels = confluence_client.get_page_labels(page_id)
        label_names = [label["name"] for label in labels.get("results", [])]
        
        assert tag not in label_names, f"Tag {tag} was not removed from the page"
        assert returncode == 0, f"Command failed with return code {returncode}"
    finally:
        # Clean up the temporary file
        os.unlink(json_file)


def test_from_json_replace(confluence_client, test_page, cleanup_tags):
    """Test executing replace commands from a JSON file."""
    page_id, title, space_key = test_page
    old_tag = f"old-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"
    
    # Add the old tag first
    confluence_client.set_page_label(page_id, old_tag)
    
    # Create a temporary JSON file with replace command
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json_file = f.name
        json.dump({
            "description": "Test replace command",
            "commands": [
                {
                    "action": "replace",
                    "cql_expression": f"contentId = {page_id}",
                    "tags": {
                        old_tag: new_tag
                    },
                    "interactive": False,
                    "cql_exclude": None
                }
            ]
        }, f)
    
    try:
        # Run the command
        cmd = f'ctag from-json {json_file}'
        stdout, stderr, returncode = run_ctag_command(cmd)
        
        # Add to cleanup list
        cleanup_tags.append((page_id, new_tag))
        
        # Verify the tag was replaced
        labels = confluence_client.get_page_labels(page_id)
        label_names = [label["name"] for label in labels.get("results", [])]
        
        assert old_tag not in label_names, f"Old tag {old_tag} was not removed from the page"
        assert new_tag in label_names, f"New tag {new_tag} was not added to the page"
        assert returncode == 0, f"Command failed with return code {returncode}"
    finally:
        # Clean up the temporary file
        os.unlink(json_file)


def test_from_json_multiple_commands(confluence_client, test_page, cleanup_tags):
    """Test executing multiple commands from a JSON file."""
    page_id, title, space_key = test_page
    add_tag = f"add-tag-{random_string()}"
    remove_tag = f"remove-tag-{random_string()}"
    old_tag = f"old-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"
    
    # Add the remove_tag and old_tag first
    confluence_client.set_page_label(page_id, remove_tag)
    confluence_client.set_page_label(page_id, old_tag)
    
    # Create a temporary JSON file with multiple commands
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json_file = f.name
        json.dump({
            "description": "Test multiple commands",
            "commands": [
                {
                    "action": "add",
                    "cql_expression": f"contentId = {page_id}",
                    "tags": [add_tag],
                    "interactive": False,
                    "cql_exclude": None
                },
                {
                    "action": "remove",
                    "cql_expression": f"contentId = {page_id}",
                    "tags": [remove_tag],
                    "interactive": False,
                    "cql_exclude": None
                },
                {
                    "action": "replace",
                    "cql_expression": f"contentId = {page_id}",
                    "tags": {
                        old_tag: new_tag
                    },
                    "interactive": False,
                    "cql_exclude": None
                }
            ]
        }, f)
    
    try:
        # Run the command
        cmd = f'ctag from-json {json_file}'
        stdout, stderr, returncode = run_ctag_command(cmd)
        
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
        assert returncode == 0, f"Command failed with return code {returncode}"
    finally:
        # Clean up the temporary file
        os.unlink(json_file)


def test_from_json_dry_run(confluence_client, test_page):
    """Test executing commands from a JSON file with dry run mode."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"
    
    # Create a temporary JSON file with add command
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json_file = f.name
        json.dump({
            "description": "Test dry run",
            "commands": [
                {
                    "action": "add",
                    "cql_expression": f"contentId = {page_id}",
                    "tags": [tag],
                    "interactive": False,
                    "cql_exclude": None
                }
            ]
        }, f)
    
    try:
        # Run the command with dry run flag
        cmd = f'ctag --dry-run from-json {json_file}'
        stdout, stderr, returncode = run_ctag_command(cmd)
        
        # Verify the tag was NOT added (dry run)
        labels = confluence_client.get_page_labels(page_id)
        label_names = [label["name"] for label in labels.get("results", [])]
        
        assert tag not in label_names, f"Tag {tag} was added despite dry run mode"
        assert returncode == 0, f"Command failed with return code {returncode}"
        assert "DRY RUN" in stdout, "Dry run message not found in output"
    finally:
        # Clean up the temporary file
        os.unlink(json_file)
