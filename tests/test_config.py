#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Test configuration and utilities for the ctag test suite.
"""

import pytest
import os
from typing import Dict, Any


class TestConfig:
    """Configuration class for test settings."""
    
    # Test environment settings
    TEST_SPACE_KEY = "ITLC"
    DEFAULT_TIMEOUT = 30
    
    # Test markers
    MARKERS = {
        'unit': 'Unit tests that don\'t require external dependencies',
        'integration': 'Integration tests that require Confluence API access',
        'slow': 'Tests that take longer than 5 seconds to run',
        'cli': 'Tests that exercise the CLI interface',
        'api': 'Tests that directly use the Confluence API',
        'dry_run': 'Tests that verify dry-run functionality',
        'exclude': 'Tests that verify CQL exclude functionality',
    }
    
    @classmethod
    def get_required_env_vars(cls) -> Dict[str, str]:
        """Get required environment variables for testing."""
        return {
            'CONFLUENCE_URL': 'The base URL of your Confluence instance',
            'CONFLUENCE_USERNAME': 'Your Confluence username',
            'ATLASSIAN_TOKEN': 'Your Atlassian API token'
        }
    
    @classmethod
    def check_test_environment(cls) -> bool:
        """Check if the test environment is properly configured."""
        missing_vars = []
        for var, desc in cls.get_required_env_vars().items():
            if not os.environ.get(var):
                missing_vars.append(f"{var} - {desc}")
        
        if missing_vars:
            pytest.skip(f"Missing required environment variables: {', '.join(missing_vars)}")
            return False
        
        return True
    
    @classmethod
    def get_test_command_prefix(cls) -> str:
        """Get the command prefix for running ctag commands in tests."""
        return "python -m src.main"


def pytest_configure(config):
    """Configure pytest with custom markers."""
    for marker, description in TestConfig.MARKERS.items():
        config.addinivalue_line("markers", f"{marker}: {description}")


def pytest_collection_modifyitems(config, items):
    """Modify test collection to add markers based on test names and locations."""
    for item in items:
        # Add markers based on test file names
        if "test_main_cli" in item.nodeid:
            item.add_marker(pytest.mark.cli)
        
        if "integration" in item.name.lower():
            item.add_marker(pytest.mark.integration)
        
        if "dry_run" in item.name.lower():
            item.add_marker(pytest.mark.dry_run)
        
        if "exclude" in item.name.lower():
            item.add_marker(pytest.mark.exclude)
        
        # Add slow marker for tests that might take longer
        if any(keyword in item.name.lower() for keyword in ['multiple', 'batch', 'large']):
            item.add_marker(pytest.mark.slow)


# Utility functions for tests
def assert_command_success(returncode: int, stderr: str, context: str = ""):
    """Assert that a command succeeded."""
    assert returncode == 0, f"Command failed{' ' + context if context else ''} with return code {returncode}: {stderr}"


def assert_command_failure(returncode: int, context: str = ""):
    """Assert that a command failed as expected."""
    assert returncode != 0, f"Command should have failed{' ' + context if context else ''}"


def assert_dry_run_output(stdout: str, action: str, tags: list, page_title: str = None):
    """Assert that dry-run output contains expected information."""
    assert "DRY RUN" in stdout, "Dry-run output should contain 'DRY RUN'"
    assert f"Would {action}" in stdout, f"Dry-run output should indicate it would {action}"
    
    for tag in tags:
        assert tag in stdout, f"Tag '{tag}' should be mentioned in dry-run output"
    
    if page_title:
        assert page_title in stdout, f"Page title '{page_title}' should be mentioned in dry-run output"


def assert_tags_present(confluence_client, page_id: str, expected_tags: list):
    """Assert that specific tags are present on a page."""
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]
    
    for tag in expected_tags:
        assert tag in label_names, f"Tag '{tag}' was not found on page {page_id}. Found tags: {label_names}"


def assert_tags_absent(confluence_client, page_id: str, expected_absent_tags: list):
    """Assert that specific tags are NOT present on a page."""
    labels = confluence_client.get_page_labels(page_id)
    label_names = [label["name"] for label in labels.get("results", [])]
    
    for tag in expected_absent_tags:
        assert tag not in label_names, f"Tag '{tag}' was found on page {page_id} but should be absent. Found tags: {label_names}"
