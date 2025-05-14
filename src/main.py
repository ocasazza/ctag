#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
ctag - A command line tool for managing tags on Confluence pages in bulk.

This module serves as the entry point for the application.
"""

import sys
import os
import logging
import click
from dotenv import load_dotenv
from atlassian import Confluence
from typing import List, Dict, Optional

# Import our modules
from src.tags import TagManager
from src.cql import CQLProcessor
from src.interactive import InteractiveHandler
from src.utils import clean_title, sanitize_text

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

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
    ctag - Manage Confluence page tags in bulk.

    This tool allows you to:
    - Add, remove, or replace tags on Confluence pages in bulk
    - Use CQL queries to select pages based on various criteria
    - Interactively confirm each action before execution

    Configuration:
    Create a .env file with:
    - CONFLUENCE_URL: Your Confluence instance URL
    - CONFLUENCE_USERNAME: Your username
    - ATLASSIAN_TOKEN: Your API token

    Example Usage:
    $ ctag add "space = DOCS" tag1 tag2 tag3
    $ ctag remove "title ~ 'Project*'" tag1 tag2 --interactive
    $ ctag replace "lastmodified > -7d" old1=new1 old2=new2
    """
    # Load environment variables from .env file
    load_dotenv()
    
    # Check environment variables
    check_environment()

    # Initialize the context object with our options
    ctx.ensure_object(dict)
    
    # Create Confluence client
    confluence = Confluence(
        url=os.environ["CONFLUENCE_URL"],
        username=os.environ["CONFLUENCE_USERNAME"],
        password=os.environ["ATLASSIAN_TOKEN"],
        cloud=True  # Set to False for server installations
    )
    
    ctx.obj.update({
        "PROGRESS": progress,
        "RECURSE": recurse,
        "DRY_RUN": dry_run,
        "CONFLUENCE_URL": os.environ["CONFLUENCE_URL"],
        "CONFLUENCE_USERNAME": os.environ["CONFLUENCE_USERNAME"],
        "ATLASSIAN_TOKEN": os.environ["ATLASSIAN_TOKEN"],
        "CONFLUENCE": confluence
    })

@cli.command()
@click.argument('cql_expression')
@click.argument('tags', nargs=-1, required=True)
@click.option('--interactive', is_flag=True, help="Confirm each action interactively")
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.pass_context
def add(ctx, cql_expression, tags, interactive, abort_key):
    """
    Add tags to pages matching CQL expression.
    
    CQL_EXPRESSION is a Confluence Query Language expression.
    TAGS are one or more tags to add to matching pages.
    
    Example:
        ctag add "space = DOCS" tag1 tag2 tag3
    """
    confluence = ctx.obj['CONFLUENCE']
    dry_run = ctx.obj['DRY_RUN']
    
    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)
    
    # Get matching pages
    click.echo(f"Finding pages matching: {cql_expression}")
    pages = cql_processor.get_all_results(cql_expression)
    
    if not pages:
        click.echo("No pages found matching the CQL expression.")
        return
    
    click.echo(f"Found {len(pages)} matching pages.")
    
    if dry_run:
        click.echo("DRY RUN: No changes will be made.")
        for page in pages:
            title = sanitize_text(page.get('title', 'Unknown'))
            space = page.get('space', {}).get('key', 'Unknown')
            click.echo(f"Would add tags {list(tags)} to '{title}' (Space: {space})")
        return
    
    # Set up interactive handler if needed
    interactive_handler = None
    if interactive:
        interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
    
    # Process the pages
    results = tag_manager.process_pages(
        pages=pages,
        action='add',
        tags=list(tags),
        interactive=interactive,
        interactive_handler=interactive_handler
    )
    
    # Display results
    click.echo(f"\nResults:")
    click.echo(f"  Total pages: {results['total']}")
    click.echo(f"  Processed: {results['processed']}")
    click.echo(f"  Skipped: {results['skipped']}")
    click.echo(f"  Successful: {results['success']}")
    click.echo(f"  Failed: {results['failed']}")


@cli.command()
@click.argument('cql_expression')
@click.argument('tags', nargs=-1, required=True)
@click.option('--interactive', is_flag=True, help="Confirm each action interactively")
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.pass_context
def remove(ctx, cql_expression, tags, interactive, abort_key):
    """
    Remove tags from pages matching CQL expression.
    
    CQL_EXPRESSION is a Confluence Query Language expression.
    TAGS are one or more tags to remove from matching pages.
    
    Example:
        ctag remove "space = DOCS" tag1 tag2
    """
    confluence = ctx.obj['CONFLUENCE']
    dry_run = ctx.obj['DRY_RUN']
    
    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)
    
    # Get matching pages
    click.echo(f"Finding pages matching: {cql_expression}")
    pages = cql_processor.get_all_results(cql_expression)
    
    if not pages:
        click.echo("No pages found matching the CQL expression.")
        return
    
    click.echo(f"Found {len(pages)} matching pages.")
    
    if dry_run:
        click.echo("DRY RUN: No changes will be made.")
        for page in pages:
            title = sanitize_text(page.get('title', 'Unknown'))
            space = page.get('space', {}).get('key', 'Unknown')
            click.echo(f"Would remove tags {list(tags)} from '{title}' (Space: {space})")
        return
    
    # Set up interactive handler if needed
    interactive_handler = None
    if interactive:
        interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
    
    # Process the pages
    results = tag_manager.process_pages(
        pages=pages,
        action='remove',
        tags=list(tags),
        interactive=interactive,
        interactive_handler=interactive_handler
    )
    
    # Display results
    click.echo(f"\nResults:")
    click.echo(f"  Total pages: {results['total']}")
    click.echo(f"  Processed: {results['processed']}")
    click.echo(f"  Skipped: {results['skipped']}")
    click.echo(f"  Successful: {results['success']}")
    click.echo(f"  Failed: {results['failed']}")


@cli.command()
@click.argument('cql_expression')
@click.argument('tag_pairs', nargs=-1, required=True)
@click.option('--interactive', is_flag=True, help="Confirm each action interactively")
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.pass_context
def replace(ctx, cql_expression, tag_pairs, interactive, abort_key):
    """
    Replace tags on pages matching CQL expression.
    
    CQL_EXPRESSION is a Confluence Query Language expression.
    TAG_PAIRS are one or more old=new tag pairs.
    
    Example:
        ctag replace "space = DOCS" old1=new1 old2=new2
    """
    confluence = ctx.obj['CONFLUENCE']
    dry_run = ctx.obj['DRY_RUN']
    
    # Parse tag pairs
    tag_mapping = {}
    for pair in tag_pairs:
        try:
            old_tag, new_tag = pair.split('=', 1)  # Split on first equals sign
            tag_mapping[old_tag.strip()] = new_tag.strip()
        except ValueError:
            raise click.BadParameter(f"Invalid tag pair format: {pair}. Use format 'oldtag=newtag'")
    
    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)
    
    # Get matching pages
    click.echo(f"Finding pages matching: {cql_expression}")
    pages = cql_processor.get_all_results(cql_expression)
    
    if not pages:
        click.echo("No pages found matching the CQL expression.")
        return
    
    click.echo(f"Found {len(pages)} matching pages.")
    
    if dry_run:
        click.echo("DRY RUN: No changes will be made.")
        for page in pages:
            title = sanitize_text(page.get('title', 'Unknown'))
            space = page.get('space', {}).get('key', 'Unknown')
            click.echo(f"Would replace tags {list(tag_mapping.keys())} with {list(tag_mapping.values())} on '{title}' (Space: {space})")
        return
    
    # Set up interactive handler if needed
    interactive_handler = None
    if interactive:
        interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
    
    # Process the pages
    results = tag_manager.process_pages(
        pages=pages,
        action='replace',
        tag_mapping=tag_mapping,
        interactive=interactive,
        interactive_handler=interactive_handler
    )
    
    # Display results
    click.echo(f"\nResults:")
    click.echo(f"  Total pages: {results['total']}")
    click.echo(f"  Processed: {results['processed']}")
    click.echo(f"  Skipped: {results['skipped']}")
    click.echo(f"  Successful: {results['success']}")
    click.echo(f"  Failed: {results['failed']}")


def main():
    """Main entry point for the application."""
    try:
        cli()
    except Exception as e:
        click.echo(f"Error: {str(e)}", err=True)
        sys.exit(1)


if __name__ == "__main__":
    sys.exit(main())
