#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
End-to-end tests for the get command.
"""

import json
import os

import pytest

from tests.conftest import random_string, run_ctag_command


def test_get_tags_table_format(confluence_client, test_page, cleanup_tags):
    """Test getting tags in table format."""
    page_id, title, space_key = test_page
    tags = [f"test-tag-{random_string()}" for _ in range(2)]

    # Add tags to the page
    for tag in tags:
        confluence_client.set_page_label(page_id, tag)
        cleanup_tags.append((page_id, tag))

    # Run the get command
    cmd = f"ctag get \"space = {space_key} AND title = '{title}'\""
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the output contains the tags
    for tag in tags:
        assert tag in stdout, f"Tag {tag} not found in output"

    assert title in stdout, f"Page title {title} not found in output"
    assert returncode == 0, f"Command failed with return code {returncode}"


def test_get_tags_json_format(confluence_client, test_page, cleanup_tags):
    """Test getting tags in JSON format."""
    page_id, title, space_key = test_page
    tags = [f"test-tag-{random_string()}" for _ in range(2)]

    # Add tags to the page
    for tag in tags:
        confluence_client.set_page_label(page_id, tag)
        cleanup_tags.append((page_id, tag))

    # Run the get command with JSON format
    cmd = f"ctag get \"space = {space_key} AND title = '{title}'\" --format json"
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Parse the JSON output
    try:
        # Extract JSON from stdout (skip the initial messages)
        lines = stdout.strip().split("\n")
        json_start = -1
        for i, line in enumerate(lines):
            if line.strip().startswith("["):
                json_start = i
                break

        if json_start >= 0:
            json_output = "\n".join(lines[json_start:])
            data = json.loads(json_output)

            # Verify the data structure
            assert isinstance(data, list), "JSON output should be a list"
            assert len(data) > 0, "JSON output should contain at least one page"

            page_data = data[0]
            assert "tags" in page_data, "Page data should contain tags"
            assert "title" in page_data, "Page data should contain title"

            # Verify tags are present
            page_tags = page_data["tags"]
            for tag in tags:
                assert tag in page_tags, f"Tag {tag} not found in JSON output"
    except json.JSONDecodeError as e:
        pytest.fail(f"Failed to parse JSON output: {e}\nOutput: {stdout}")

    assert returncode == 0, f"Command failed with return code {returncode}"


def test_get_tags_only(confluence_client, test_page, cleanup_tags):
    """Test getting only unique tags."""
    page_id, title, space_key = test_page
    tags = [f"test-tag-{random_string()}" for _ in range(2)]

    # Add tags to the page
    for tag in tags:
        confluence_client.set_page_label(page_id, tag)
        cleanup_tags.append((page_id, tag))

    # Run the get command with tags-only flag
    cmd = f"ctag get \"space = {space_key} AND title = '{title}'\" --tags-only"
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the output contains the tags but not page details
    for tag in tags:
        assert tag in stdout, f"Tag {tag} not found in output"

    # Should not contain page title in tags-only mode
    assert "Tags found:" in stdout, "Tags found header not in output"
    assert returncode == 0, f"Command failed with return code {returncode}"


def test_get_tags_with_cql_exclude(confluence_client, test_page, cleanup_tags):
    """Test getting tags with CQL exclude expression."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"
    exclude_tag = f"exclude-{random_string()}"

    # Add tags to the page
    confluence_client.set_page_label(page_id, tag)
    confluence_client.set_page_label(page_id, exclude_tag)
    cleanup_tags.append((page_id, tag))
    cleanup_tags.append((page_id, exclude_tag))

    # Run the get command with exclude
    cmd = f"ctag get \"space = {space_key} AND title = '{title}'\" --cql-exclude \"label = '{exclude_tag}'\""
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the page was excluded
    assert "Excluded" in stdout, "Exclusion message not found in output"
    assert "0 pages remaining" in stdout or "No pages found" in stdout, "Page should have been excluded"
    assert returncode == 0, f"Command failed with return code {returncode}"


def test_get_tags_no_pages_found(confluence_client):
    """Test getting tags when no pages match the CQL."""
    nonexistent_title = f"Nonexistent Page {random_string()}"

    # Run the get command
    cmd = f"ctag get \"title = '{nonexistent_title}'\""
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the command completed but found no pages
    assert returncode == 0, f"Command failed with return code {returncode}"
    assert "No pages found" in stdout, "No pages found message not found in output"


def test_get_tags_output_file(confluence_client, test_page, cleanup_tags, tmp_path):
    """Test getting tags and saving to output file."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Add tag to the page
    confluence_client.set_page_label(page_id, tag)
    cleanup_tags.append((page_id, tag))

    # Create output file path
    output_file = tmp_path / "tags_output.json"

    # Run the get command with output file
    cmd = f"ctag get \"space = {space_key} AND title = '{title}'\" --format json --output-file {output_file}"
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the file was created
    assert output_file.exists(), "Output file was not created"

    # Verify the file contains the expected data
    with open(output_file, "r") as f:
        data = json.load(f)
        assert isinstance(data, list), "JSON output should be a list"
        assert len(data) > 0, "JSON output should contain at least one page"

        page_data = data[0]
        assert tag in page_data["tags"], f"Tag {tag} not found in output file"

    assert "Results saved to" in stdout, "Save confirmation not found in output"
    assert returncode == 0, f"Command failed with return code {returncode}"


def test_get_tags_no_show_pages(confluence_client, test_page, cleanup_tags):
    """Test getting tags without showing page information."""
    page_id, title, space_key = test_page
    tag = f"test-tag-{random_string()}"

    # Add tag to the page
    confluence_client.set_page_label(page_id, tag)
    cleanup_tags.append((page_id, tag))

    # Run the get command without showing pages
    cmd = f"ctag get \"space = {space_key} AND title = '{title}'\" --no-show-pages"
    stdout, stderr, returncode = run_ctag_command(cmd)

    # Verify the tag is shown but not page details
    assert tag in stdout, f"Tag {tag} not found in output"
    assert "Tags found:" in stdout, "Tags found header not in output"
    assert returncode == 0, f"Command failed with return code {returncode}"
