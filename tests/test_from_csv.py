#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
End-to-end tests for the from-csv command.
"""

import os
import csv
import tempfile
import pytest
from tests.conftest import run_ctag_command, random_string


def test_from_csv_add(confluence_client, test_page, cleanup_tags):
    """Test executing add commands from a CSV file."""
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


def test_from_csv_remove(confluence_client, test_page):
    """Test executing remove commands from a CSV file."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"
    
    # Add the tag first
    confluence_client.set_page_label(page_id, tag)
    
    # Create a temporary CSV file with remove command
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False, newline='') as f:
        csv_file = f.name
        writer = csv.writer(f)
        writer.writerow(["action", "cql_expression", "tags", "interactive", "cql_exclude"])
        writer.writerow(["remove", f"contentId = {page_id}", tag, "false", ""])
    
    try:
        # Run the command
        cmd = f'ctag from-csv {csv_file}'
        stdout, stderr, returncode = run_ctag_command(cmd)
        
        # Verify the tag was removed
        labels = confluence_client.get_page_labels(page_id)
        label_names = [label["name"] for label in labels.get("results", [])]
        
        assert tag not in label_names, f"Tag {tag} was not removed from the page"
        assert returncode == 0, f"Command failed with return code {returncode}"
    finally:
        # Clean up the temporary file
        os.unlink(csv_file)


def test_from_csv_replace(confluence_client, test_page, cleanup_tags):
    """Test executing replace commands from a CSV file."""
    page_id, title, space_key = test_page
    old_tag = f"old-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"
    
    # Add the old tag first
    confluence_client.set_page_label(page_id, old_tag)
    
    # Create a temporary CSV file with replace command
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False, newline='') as f:
        csv_file = f.name
        writer = csv.writer(f)
        writer.writerow(["action", "cql_expression", "tags", "interactive", "cql_exclude"])
        writer.writerow(["replace", f"contentId = {page_id}", f"{old_tag}={new_tag}", "false", ""])
    
    try:
        # Run the command
        cmd = f'ctag from-csv {csv_file}'
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
        os.unlink(csv_file)


def test_from_csv_multiple_commands(confluence_client, test_page, cleanup_tags):
    """Test executing multiple commands from a CSV file."""
    page_id, title, space_key = test_page
    add_tag = f"add-tag-{random_string()}"
    remove_tag = f"remove-tag-{random_string()}"
    old_tag = f"old-tag-{random_string()}"
    new_tag = f"new-tag-{random_string()}"
    
    # Add the remove_tag and old_tag first
    confluence_client.set_page_label(page_id, remove_tag)
    confluence_client.set_page_label(page_id, old_tag)
    
    # Create a temporary CSV file with multiple commands
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False, newline='') as f:
        csv_file = f.name
        writer = csv.writer(f)
        writer.writerow(["action", "cql_expression", "tags", "interactive", "cql_exclude"])
        writer.writerow(["add", f"contentId = {page_id}", add_tag, "false", ""])
        writer.writerow(["remove", f"contentId = {page_id}", remove_tag, "false", ""])
        writer.writerow(["replace", f"contentId = {page_id}", f"{old_tag}={new_tag}", "false", ""])
    
    try:
        # Run the command
        cmd = f'ctag from-csv {csv_file}'
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
        os.unlink(csv_file)


def test_from_csv_dry_run(confluence_client, test_page):
    """Test executing commands from a CSV file with dry run mode."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"
    
    # Create a temporary CSV file with add command
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False, newline='') as f:
        csv_file = f.name
        writer = csv.writer(f)
        writer.writerow(["action", "cql_expression", "tags", "interactive", "cql_exclude"])
        writer.writerow(["add", f"contentId = {page_id}", tag, "false", ""])
    
    try:
        # Run the command with dry run flag
        cmd = f'ctag --dry-run from-csv {csv_file}'
        stdout, stderr, returncode = run_ctag_command(cmd)
        
        # Verify the tag was NOT added (dry run)
        labels = confluence_client.get_page_labels(page_id)
        label_names = [label["name"] for label in labels.get("results", [])]
        
        assert tag not in label_names, f"Tag {tag} was added despite dry run mode"
        assert returncode == 0, f"Command failed with return code {returncode}"
        assert "DRY RUN" in stdout, "Dry run message not found in output"
    finally:
        # Clean up the temporary file
        os.unlink(csv_file)


def test_from_csv_multiple_tags(confluence_client, test_page, cleanup_tags):
    """Test executing add command with multiple tags from a CSV file."""
    page_id, title, space_key = test_page
    tags = [f"test-tag-{random_string()}", f"test-tag-{random_string()}", f"test-tag-{random_string()}"]
    tags_str = ",".join(tags)
    
    # Create a temporary CSV file with add command for multiple tags
    with tempfile.NamedTemporaryFile(mode='w', suffix='.csv', delete=False, newline='') as f:
        csv_file = f.name
        writer = csv.writer(f)
        writer.writerow(["action", "cql_expression", "tags", "interactive", "cql_exclude"])
        writer.writerow(["add", f"contentId = {page_id}", tags_str, "false", ""])
    
    try:
        # Run the command
        cmd = f'ctag from-csv {csv_file}'
        stdout, stderr, returncode = run_ctag_command(cmd)
        
        # Add to cleanup list
        for tag in tags:
            cleanup_tags.append((page_id, tag))
        
        # Verify the tags were added
        labels = confluence_client.get_page_labels(page_id)
        label_names = [label["name"] for label in labels.get("results", [])]
        
        for tag in tags:
            assert tag in label_names, f"Tag {tag} was not added to the page"
        
        assert returncode == 0, f"Command failed with return code {returncode}"
    finally:
        # Clean up the temporary file
        os.unlink(csv_file)
