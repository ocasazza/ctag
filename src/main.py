#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
CLI module for the ctag tool.

This module defines the main CLI interface for the ctag tool.
"""
import logging
import os
import sys

import click
from atlassian import Confluence
from dotenv import load_dotenv

# Import commands
from src.commands.add import add
from src.commands.from_json import from_json
from src.commands.from_stdin_json import from_stdin_json
from src.commands.get import get
from src.commands.remove import remove
from src.commands.replace import replace

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)

# Required environment variables
REQUIRED_ENV_VARS = {
    "ATLASSIAN_URL": "The base URL of your Confluence instance",
    "ATLASSIAN_USERNAME": "Your Confluence username",
    "ATLASSIAN_TOKEN": "Your Atlassian API token",
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
        click.echo(
            "\nCreate a .env file with these variables or set them in your environment.",
            err=True,
        )
        raise click.ClickException("Missing required environment variables")


@click.group()
@click.version_option(version="0.1.0")
@click.option("--progress", default=True, help="Show progress bars during operations")
@click.option(
    "--dry-run",
    is_flag=True,
    help="Preview changes without making any modifications",
)
@click.pass_context
def cli(ctx, progress, dry_run):
    """
    ctag - Manage Confluence page tags in bulk with a CLI.

    This tool allows you to:
    - Add, remove, or replace tags on Confluence pages in bulk
    - Use CQL queries to select pages based on various criteria
    - Interactively confirm each action before execution

    Configuration:
    Create a .env file with:
    - ATLASSIAN_URL: Your Confluence instance URL
    - ATLASSIAN_USERNAME: Your username
    - ATLASSIAN_TOKEN: Your API token

    Example Usage:
    $ ctag add "space = DOCS" tag1 tag2 tag3
    $ ctag remove "title ~ 'Project*'" tag1 tag2 --interactive
    $ ctag replace "lastmodified > -7d" old1=new1 old2=new2
    """
    # Load environment variables from .env file
    load_dotenv(dotenv_path=".env")

    # Check environment variables
    check_environment()

    # Initialize the context object with our options
    ctx.ensure_object(dict)

    # Create Confluence client
    confluence = Confluence(
        url=os.environ["ATLASSIAN_URL"],
        username=os.environ["ATLASSIAN_USERNAME"],
        password=os.environ["ATLASSIAN_TOKEN"],
        cloud=True,  # Set to False for server installations
    )

    ctx.obj.update(
        {
            "PROGRESS": progress,
            "DRY_RUN": dry_run,
            "ATLASSIAN_URL": os.environ["ATLASSIAN_URL"],
            "ATLASSIAN_USERNAME": os.environ["ATLASSIAN_USERNAME"],
            "ATLASSIAN_TOKEN": os.environ["ATLASSIAN_TOKEN"],
            "CONFLUENCE": confluence,
        }
    )


# Register commands
cli.add_command(add)
cli.add_command(remove)
cli.add_command(replace)
cli.add_command(from_json)
cli.add_command(from_stdin_json)
cli.add_command(get)


def main():
    """Entry point for the ctag CLI tool."""
    cli()


if __name__ == "__main__":
    main()
