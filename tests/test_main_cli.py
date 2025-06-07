#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Tests for the main CLI functionality, including global options and argument parsing.
"""

import os
import subprocess

import pytest

from tests.conftest import random_string, run_ctag_command


class TestMainCLI:
    """Test the main CLI interface and global options."""

    def test_cli_version(self):
        """Test that the CLI version displays correctly."""
        cmd = "python -m src.main --version"
        stdout, stderr, returncode = run_ctag_command(cmd)

        assert returncode == 0, f"Version command failed with return code {returncode}"
        assert "0.1.0" in stdout

    def test_global_dry_run_option_position(self):
        """Test that --dry-run works when placed before the subcommand."""
        # This should work (correct position)
        cmd = 'python -m src.main --dry-run add "id = 999999" test-tag'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # Should succeed (even with invalid page ID, dry-run should work)
        # Note: May fail with missing env vars, but that's expected in some test environments
        if returncode != 0 and "Missing required environment variables" in stderr:
            # Skip this test if environment variables are not available
            pytest.skip("Environment variables not available for this test")

        assert returncode == 0, f"Dry-run command failed with return code {returncode}"
        assert "DRY RUN" in stdout or "No pages found" in stdout

    def test_invalid_global_option_position(self):
        """Test that global options after subcommand are handled correctly."""
        # This tests the current behavior - Click will treat --dry-run as an unknown option
        # for the add subcommand since it's not defined locally
        cmd = 'python -m src.main add "id = 999999" test-tag --dry-run'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # Should fail with unknown option error
        assert returncode != 0, "Command should fail with unknown option"
        assert "no such option" in stderr.lower() or "unrecognized" in stderr.lower()

    def test_missing_environment_variables(self):
        """Test behavior when required environment variables are missing."""
        # Create a clean environment without the required variables
        clean_env = os.environ.copy()
        env_vars = ["ATLASSIAN_URL", "ATLASSIAN_USERNAME", "ATLASSIAN_TOKEN"]

        for var in env_vars:
            clean_env.pop(var, None)

        # Temporarily rename .env file to prevent load_dotenv from loading it
        env_file_exists = os.path.exists(".env")
        if env_file_exists:
            os.rename(".env", ".env.backup")

        try:
            cmd = 'python -m src.main add "space = TEST" test-tag'
            stdout, stderr, returncode = run_ctag_command(cmd, env=clean_env)

            # Should fail with missing environment variables error
            assert (
                returncode != 0
            ), "Command should fail with missing environment variables"
            assert "Missing required environment variables" in stderr

        finally:
            # Restore .env file
            if env_file_exists:
                os.rename(".env.backup", ".env")

    def test_progress_option(self):
        """Test that the --progress option is accepted."""
        cmd = 'python -m src.main --progress false --dry-run add "id = 999999" test-tag'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # May fail with missing env vars, but that's expected in some test environments
        if returncode != 0 and "Missing required environment variables" in stderr:
            pytest.skip("Environment variables not available for this test")

        assert (
            returncode == 0
        ), f"Progress option command failed with return code {returncode}"

    def test_multiple_global_options(self):
        """Test using multiple global options together."""
        cmd = 'python -m src.main --dry-run --progress true add "id = 999999" test-tag'
        stdout, stderr, returncode = run_ctag_command(cmd)

        # May fail with missing env vars, but that's expected in some test environments
        if returncode != 0 and "Missing required environment variables" in stderr:
            pytest.skip("Environment variables not available for this test")

        assert (
            returncode == 0
        ), f"Multiple global options command failed with return code {returncode}"
        assert "DRY RUN" in stdout or "No pages found" in stdout


class TestCLIArgumentValidation:
    """Test CLI argument validation and error handling."""

    def test_missing_required_arguments(self):
        """Test that missing required arguments are handled correctly."""
        # Missing CQL expression
        cmd = "python -m src.main add"
        stdout, stderr, returncode = run_ctag_command(cmd)

        assert returncode != 0, "Command should fail with missing arguments"
        assert "Missing argument" in stderr or "Usage:" in stderr

    def test_invalid_subcommand(self):
        """Test that invalid subcommands are handled correctly."""
        cmd = "python -m src.main invalid-command"
        stdout, stderr, returncode = run_ctag_command(cmd)

        assert returncode != 0, "Command should fail with invalid subcommand"
        assert "No such command" in stderr or "Usage:" in stderr

    def test_empty_tag_list(self):
        """Test that empty tag lists are handled correctly."""
        cmd = 'python -m src.main add "id = 999999"'
        stdout, stderr, returncode = run_ctag_command(cmd)

        assert returncode != 0, "Command should fail with empty tag list"
        assert "Missing argument" in stderr or "Usage:" in stderr


@pytest.mark.integration
class TestCLIIntegration:
    """Integration tests for CLI functionality."""

    def test_dry_run_with_real_command(self, confluence_client, test_page):
        """Test dry-run mode with a real command and page."""
        page_id, title, space_key = test_page
        tag = f"test-tag-{random_string()}"

        # Verify the tag doesn't exist initially
        labels_before = confluence_client.get_page_labels(page_id)
        label_names_before = [
            label["name"] for label in labels_before.get("results", [])
        ]
        assert tag not in label_names_before

        # Run with dry-run
        cmd = f'python -m src.main --dry-run add "id = {page_id}" {tag}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        assert returncode == 0, f"Dry-run command failed: {stderr}"
        assert "DRY RUN" in stdout
        assert f"Would add tags ['{tag}']" in stdout

        # Verify the tag was NOT actually added
        labels_after = confluence_client.get_page_labels(page_id)
        label_names_after = [label["name"] for label in labels_after.get("results", [])]
        assert tag not in label_names_after

    def test_progress_option_with_real_command(
        self, confluence_client, test_page, cleanup_tags
    ):
        """Test progress option with a real command."""
        page_id, title, space_key = test_page
        tag = f"test-tag-{random_string()}"

        # Run with progress enabled and dry-run
        cmd = f'python -m src.main --progress true --dry-run add "id = {page_id}" {tag}'
        stdout, stderr, returncode = run_ctag_command(cmd)

        assert returncode == 0, f"Progress command failed: {stderr}"
        assert "DRY RUN" in stdout
