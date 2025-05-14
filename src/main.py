#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
atool - A command line program for Confluence page management.

This module serves as the entry point for the application.
"""

import sys
import os
import logging
import click
from dotenv import load_dotenv
from atlassian import Confluence
from src.engine import SyncEngine

# Required environment variables
REQUIRED_ENV_VARS = {
    'CONFLUENCE_URL': 'The base URL of your Confluence instance',
    'CONFLUENCE_USERNAME': 'Your Confluence username',
    'ATLASSIAN_TOKEN': 'Your Atlassian API token'
}

def check_environment():
    """Check if all required environment variables are set."""
    missing = []
    for var, desc in REQUIRED_ENV_VARS.items():
        if not os.environ.get(var):
            missing.append(f"{var} - {desc}")
    
    if missing:
        click.echo("Error: Missing required environment variables:", err=True)
        for var in missing:
            click.echo(f"  {var}", err=True)
        click.echo("\nCreate a .env file with these variables or set them in your environment.", err=True)
        sys.exit(1)


@click.group()
@click.version_option(version="0.1.0")
@click.option("--progress", default=True, 
              help="Show progress bars during operations")
@click.option("--recurse", default=True,
              help="Process child pages recursively")
@click.option("--dry-run", is_flag=True,
              help="Preview changes without making any modifications")
@click.pass_context
def cli(ctx, progress, recurse, dry_run):
    """
    atool - Synchronize Confluence pages with your local filesystem.

    This tool allows you to:
    - Pull Confluence pages to your local filesystem
    - Push local changes back to Confluence
    - Track page history and changes
    - Handle page renames and moves
    - Sync attachments

    Configuration:
    Create a .env file with:
    - CONFLUENCE_URL: Your Confluence instance URL
    - CONFLUENCE_USERNAME: Your username
    - ATLASSIAN_TOKEN: Your API token

    Example Usage:
    $ atool pull "https://<confluence-url>/wiki/spaces/SPACE/pages/123" ./docs
    $ atool push ./docs "https://<confluence-url>/wiki/spaces/SPACE/pages/123"
    """
    # Load environment variables from .env file
    load_dotenv()
    
    # Check environment variables
    check_environment()

    # Initialize the context object with our options
    ctx.ensure_object(dict)
    ctx.obj.update({
        "PROGRESS": progress,
        "RECURSE": recurse,
        "DRY_RUN": dry_run,
        "CONFLUENCE_URL": os.environ["CONFLUENCE_URL"],
        "CONFLUENCE_USERNAME": os.environ["CONFLUENCE_USERNAME"],
        "ATLASSIAN_TOKEN": os.environ["ATLASSIAN_TOKEN"]
    })

def main():
    """Main entry point for the application."""
    try:
        cli()
    except Exception as e:
        click.echo(f"Error: {str(e)}", err=True)
        sys.exit(1)


if __name__ == "__main__":
    sys.exit(main())
