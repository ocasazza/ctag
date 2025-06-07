#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Pytest configuration and fixtures for ctag tests.
"""

import json
import os
import random
import string
import subprocess

import pytest
from atlassian import Confluence

# Test space key for creating test pages
TEST_SPACE_KEY = "ITLC"


def random_string(length=8):
    """Generate a random string for unique test identifiers."""
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=length))


@pytest.fixture(scope="session")
def confluence_client():
    """Create a Confluence client for verification and cleanup."""
    # Load environment variables from .env file if present
    if os.path.exists(".env"):
        from dotenv import load_dotenv

        load_dotenv()

    # Check if required environment variables are set
    required_vars = [
        "CONFLUENCE_URL",
        "CONFLUENCE_USERNAME",
        "ATLASSIAN_TOKEN",
    ]
    missing_vars = [var for var in required_vars if not os.environ.get(var)]

    if missing_vars:
        pytest.skip(
            f"Missing required environment variables: {
        ', '.join(missing_vars)}"
        )

    confluence = Confluence(
        url=os.environ["CONFLUENCE_URL"],
        username=os.environ["CONFLUENCE_USERNAME"],
        password=os.environ["ATLASSIAN_TOKEN"],
        cloud=True,
    )
    return confluence


@pytest.fixture
def test_page(confluence_client):
    """Create a test page for tag operations."""
    # Create a unique title for the test page
    title = f"Test Page {random_string()}"
    body = "<p>This is a test page for ctag testing.</p>"

    try:
        # Create the page
        page = confluence_client.create_page(space=TEST_SPACE_KEY, title=title, body=body)

        page_id = page["id"]

        yield page_id, title, TEST_SPACE_KEY

        # Cleanup: Delete the test page
        confluence_client.remove_page(page_id)
    except Exception as e:
        pytest.skip(f"Failed to create test page: {str(e)}")


@pytest.fixture
def cleanup_tags(confluence_client):
    """Fixture to track and clean up tags added during tests."""
    tags_to_clean = []  # List of (page_id, tag) tuples

    yield tags_to_clean

    # Cleanup phase
    for page_id, tag in tags_to_clean:
        try:
            confluence_client.remove_page_label(page_id, tag)
        except Exception as e:
            print(f"Failed to clean up tag {tag} on page {page_id}: {str(e)}")


def run_ctag_command(command):
    """Run a ctag command and return the output."""
    result = subprocess.run(command, shell=True, capture_output=True, text=True)
    return result.stdout, result.stderr, result.returncode
