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
import json

# Import our modules
from src.tags import TagManager
from src.cql import CQLProcessor, SearchResultItem
from src.interactive import InteractiveHandler
from src.utils import clean_title, sanitize_text
from src.json_processor import JSONCommand, read_commands_from_json, validate_json_file
from src.csv_processor import read_commands_from_csv, validate_csv_file

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

def filter_excluded_pages(pages: List[SearchResultItem], excluded_pages: List[SearchResultItem]) -> List[SearchResultItem]:
    """Filter out pages that are in the excluded_pages list based on page ID.
    
    Args:
        pages: List of pages to filter
        excluded_pages: List of pages to exclude
        
    Returns:
        Filtered list of pages
    """
    excluded_ids = [page.content.id for page in excluded_pages if page.content and page.content.id]
    return [page for page in pages if not (page.content and page.content.id and page.content.id in excluded_ids)]


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
@click.option('--cql-exclude', required=False, help="CQL expression to match pages that should be excluded from this operation")
@click.pass_context
def add(ctx, cql_expression, tags, interactive, abort_key, cql_exclude):
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
    
    # Apply exclusion if specified
    if cql_exclude:
        click.echo(f"Finding pages to exclude: {cql_exclude}")
        excluded_pages = cql_processor.get_all_results(cql_exclude)
        if excluded_pages:
            original_count = len(pages)
            pages = filter_excluded_pages(pages, excluded_pages)
            click.echo(f"Excluded {original_count - len(pages)} pages. {len(pages)} pages remaining.")
    
    if dry_run:
        click.echo("DRY RUN: No changes will be made.")
        for page in pages:
            title = sanitize_text(page.title if page.title else 'Unknown')
            space = page.resultGlobalContainer.title if page.resultGlobalContainer and page.resultGlobalContainer.title else 'Unknown'
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
@click.option('--cql-exclude', required=False, help="CQL expression to match pages that should be excluded from this operation")
@click.pass_context
def remove(ctx, cql_expression, tags, interactive, abort_key, cql_exclude):
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
    
    # Apply exclusion if specified
    if cql_exclude:
        click.echo(f"Finding pages to exclude: {cql_exclude}")
        excluded_pages = cql_processor.get_all_results(cql_exclude)
        if excluded_pages:
            original_count = len(pages)
            pages = filter_excluded_pages(pages, excluded_pages)
            click.echo(f"Excluded {original_count - len(pages)} pages. {len(pages)} pages remaining.")
    
    if dry_run:
        click.echo("DRY RUN: No changes will be made.")
        for page in pages:
            title = sanitize_text(page.title if page.title else 'Unknown')
            space = page.resultGlobalContainer.title if page.resultGlobalContainer and page.resultGlobalContainer.title else 'Unknown'
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
@click.option('--cql-exclude', required=False, help="CQL expression to match pages that should be excluded from this operation")
@click.pass_context
def replace(ctx, cql_expression, tag_pairs, interactive, abort_key, cql_exclude):
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
    
    # Apply exclusion if specified
    if cql_exclude:
        click.echo(f"Finding pages to exclude: {cql_exclude}")
        excluded_pages = cql_processor.get_all_results(cql_exclude)
        if excluded_pages:
            original_count = len(pages)
            pages = filter_excluded_pages(pages, excluded_pages)
            click.echo(f"Excluded {original_count - len(pages)} pages. {len(pages)} pages remaining.")
    
    if dry_run:
        click.echo("DRY RUN: No changes will be made.")
        for page in pages:
            title = sanitize_text(page.title if page.title else 'Unknown')
            space = page.resultGlobalContainer.title if page.resultGlobalContainer and page.resultGlobalContainer.title else 'Unknown'
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


@cli.command()
@click.argument('json_file', type=click.Path(exists=True, file_okay=True, dir_okay=False, readable=True))
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.pass_context
def from_json(ctx, json_file, abort_key):
    """
    Execute commands from a JSON file.
    
    JSON_FILE is the path to a JSON file containing commands to execute.
    
    The JSON file should have the following structure:
    {
        "description": "Optional description of the commands",
        "commands": [
            {
                "action": "add",
                "cql_expression": "space = DOCS",
                "tags": ["tag1", "tag2"],
                "interactive": false,
                "cql_exclude": null
            },
            {
                "action": "replace",
                "cql_expression": "space =
@cli.command()
@click.argument('csv_file', type=click.Path(exists=True, file_okay=True, dir_okay=False, readable=True))
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.pass_context
def from_csv(ctx, csv_file, abort_key):
    """
    Execute commands from a CSV file.
    
    CSV_FILE is the path to a CSV file containing commands to execute.
    
    The CSV file should have the following columns:
    - action: The action to perform (add, remove, or replace)
    - cql_expression: The CQL query to select pages
    - tags: For add/remove actions, a comma-separated list of tags;
            for replace actions, a comma-separated list of old=new pairs
    - interactive: (optional) Whether to confirm each action interactively (true/false)
    - cql_exclude: (optional) CQL expression to match pages that should be excluded
    
    Example:
        ctag from_csv commands.csv
    """
    confluence = ctx.obj['CONFLUENCE']
    dry_run = ctx.obj['DRY_RUN']
    
    # Validate the CSV file
    if not validate_csv_file(csv_file):
        click.echo(f"Error: Invalid CSV file format: {csv_file}")
        return
    
    # Read commands from the CSV file
    try:
        commands = read_commands_from_csv(csv_file)
    except Exception as e:
        click.echo(f"Error reading CSV file: {str(e)}")
        return
    
    if not commands:
        click.echo("No valid commands found in the CSV file.")
        return
    
    click.echo(f"Found {len(commands)} commands in the CSV file.")
    
    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)
    
    # Process each command
    total_results = {
        'total': 0,
        'processed': 0,
        'skipped': 0,
        'success': 0,
        'failed': 0
    }
    
    for i, command in enumerate(commands):
        click.echo(f"\nExecuting command {i+1}/{len(commands)}: {command}")
        
        # Get matching pages
        click.echo(f"Finding pages matching: {command.cql_expression}")
        pages = cql_processor.get_all_results(command.cql_expression)
        
        if not pages:
            click.echo("No pages found matching the CQL expression.")
            continue
        
        click.echo(f"Found {len(pages)} matching pages.")
        
        # Apply exclusion if specified
        if command.cql_exclude:
            click.echo(f"Finding pages to exclude: {command.cql_exclude}")
            excluded_pages = cql_processor.get_all_results(command.cql_exclude)
            if excluded_pages:
                original_count = len(pages)
                pages = filter_excluded_pages(pages, excluded_pages)
                click.echo(f"Excluded {original_count - len(pages)} pages. {len(pages)} pages remaining.")
        
        if dry_run:
            click.echo("DRY RUN: No changes will be made.")
            for page in pages:
                title = sanitize_text(page.title if page.title else 'Unknown')
                space = page.resultGlobalContainer.title if page.resultGlobalContainer and page.resultGlobalContainer.title else 'Unknown'
                
                if command.action == 'add':
                    click.echo(f"Would add tags {command.tags} to '{title}' (Space: {space})")
                elif command.action == 'remove':
                    click.echo(f"Would remove tags {command.tags} from '{title}' (Space: {space})")
                else:  # replace
                    click.echo(f"Would replace tags {list(command.tag_mapping.keys())} with {list(command.tag_mapping.values())} on '{title}' (Space: {space})")
            continue
        
        # Set up interactive handler if needed
        interactive_handler = None
        if command.interactive:
            interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
        
        # Process the pages
        if command.action in ('add', 'remove'):
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tags=command.tags,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        else:  # replace
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tag_mapping=command.tag_mapping,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        
        # Update total results
        for key in total_results:
            total_results[key] += results[key]
        
        # Display results for this command
        click.echo(f"\nResults for command {i+1}:")
        click.echo(f"  Total pages: {results['total']}")
        click.echo(f"  Processed: {results['processed']}")
        click.echo(f"  Skipped: {results['skipped']}")
        click.echo(f"  Successful: {results['success']}")
        click.echo(f"  Failed: {results['failed']}")
        
        # Check if aborted
        if results.get('aborted', False):
            click.echo("\nAborted by user. Stopping execution.")
            break
    
    # Display overall results
    click.echo(f"\nOverall Results:")
    click.echo(f"  Total pages: {total_results['total']}")
    click.echo(f"  Processed: {total_results['processed']}")
    click.echo(f"  Skipped: {total_results['skipped']}")
    click.echo(f"  Successful: {total_results['success']}")
    click.echo(f"  Failed: {total_results['failed']}")
@cli.command()
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.pass_context
def from_stdin_json(ctx, abort_key):
    """
    Execute commands from JSON data provided via stdin.
    
    This command reads JSON data from stdin and executes the commands.
    The JSON data should have the same structure as the JSON file used with the from_json command.
    
    Example:
        cat examples/commands.json | ctag from_stdin_json
        echo '{"commands":[{"action":"add","cql_expression":"space = DOCS","tags":["tag1"]}]}' | ctag from_stdin_json
    """
    from src.stdin_processor import read_json_from_stdin, is_stdin_available
    
    if not is_stdin_available():
        click.echo("Error: No data provided via stdin. Use a pipe or redirect to provide JSON data.")
        return
    
    confluence = ctx.obj['CONFLUENCE']
    dry_run = ctx.obj['DRY_RUN']
    
    # Read commands from stdin
    try:
        commands = read_json_from_stdin()
    except Exception as e:
        click.echo(f"Error reading JSON from stdin: {str(e)}")
        return
    
    if not commands:
        click.echo("No valid commands found in the JSON data.")
        return
    
    click.echo(f"Found {len(commands)} commands in the JSON data.")
    
    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)
    
    # Process each command
    total_results = {
        'total': 0,
        'processed': 0,
        'skipped': 0,
        'success': 0,
        'failed': 0
    }
    
    for i, command in enumerate(commands):
        click.echo(f"\nExecuting command {i+1}/{len(commands)}: {command}")
        
        # Get matching pages
        click.echo(f"Finding pages matching: {command.cql_expression}")
        pages = cql_processor.get_all_results(command.cql_expression)
        
        if not pages:
            click.echo("No pages found matching the CQL expression.")
            continue
        
        click.echo(f"Found {len(pages)} matching pages.")
        
        # Apply exclusion if specified
        if command.cql_exclude:
            click.echo(f"Finding pages to exclude: {command.cql_exclude}")
            excluded_pages = cql_processor.get_all_results(command.cql_exclude)
            if excluded_pages:
                original_count = len(pages)
                pages = filter_excluded_pages(pages, excluded_pages)
                click.echo(f"Excluded {original_count - len(pages)} pages. {len(pages)} pages remaining.")
        
        if dry_run:
            click.echo("DRY RUN: No changes will be made.")
            for page in pages:
                title = sanitize_text(page.title if page.title else 'Unknown')
                space = page.resultGlobalContainer.title if page.resultGlobalContainer and page.resultGlobalContainer.title else 'Unknown'
                
                if command.action == 'add':
                    click.echo(f"Would add tags {command.tags} to '{title}' (Space: {space})")
                elif command.action == 'remove':
                    click.echo(f"Would remove tags {command.tags} from '{title}' (Space: {space})")
                else:  # replace
                    click.echo(f"Would replace tags {list(command.tag_mapping.keys())} with {list(command.tag_mapping.values())} on '{title}' (Space: {space})")
            continue
        
        # Set up interactive handler if needed
        interactive_handler = None
        if command.interactive:
            interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
        
        # Process the pages
        if command.action in ('add', 'remove'):
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tags=command.tags,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        else:  # replace
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tag_mapping=command.tag_mapping,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        
        # Update total results
        for key in total_results:
            total_results[key] += results[key]
        
        # Display results for this command
        click.echo(f"\nResults for command {i+1}:")
        click.echo(f"  Total pages: {results['total']}")
        click.echo(f"  Processed: {results['processed']}")
        click.echo(f"  Skipped: {results['skipped']}")
        click.echo(f"  Successful: {results['success']}")
        click.echo(f"  Failed: {results['failed']}")
        
        # Check if aborted
        if results.get('aborted', False):
            click.echo("\nAborted by user. Stopping execution.")
            break
    
    # Display overall results
    click.echo(f"\nOverall Results:")
    click.echo(f"  Total pages: {total_results['total']}")
    click.echo(f"  Processed: {total_results['processed']}")
    click.echo(f"  Skipped: {total_results['skipped']}")
    click.echo(f"  Successful: {total_results['success']}")
    click.echo(f"  Failed: {total_results['failed']}")


@cli.command()
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.pass_context
def from_stdin_csv(ctx, abort_key):
    """
    Execute commands from CSV data provided via stdin.
    
    This command reads CSV data from stdin and executes the commands.
    The CSV data should have the same structure as the CSV file used with the from_csv command.
    
    Example:
        cat examples/commands.csv | ctag from_stdin_csv
        echo 'action,cql_expression,tags\nadd,space = DOCS,tag1,tag2' | ctag from_stdin_csv
    """
    from src.stdin_processor import read_csv_from_stdin, is_stdin_available
    
    if not is_stdin_available():
        click.echo("Error: No data provided via stdin. Use a pipe or redirect to provide CSV data.")
        return
    
    confluence = ctx.obj['CONFLUENCE']
    dry_run = ctx.obj['DRY_RUN']
    
    # Read commands from stdin
    try:
        commands = read_csv_from_stdin()
    except Exception as e:
        click.echo(f"Error reading CSV from stdin: {str(e)}")
        return
    
    if not commands:
        click.echo("No valid commands found in the CSV data.")
        return
    
    click.echo(f"Found {len(commands)} commands in the CSV data.")
    
    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)
    
    # Process each command
    total_results = {
        'total': 0,
        'processed': 0,
        'skipped': 0,
        'success': 0,
        'failed': 0
    }
    
    for i, command in enumerate(commands):
        click.echo(f"\nExecuting command {i+1}/{len(commands)}: {command}")
        
        # Get matching pages
        click.echo(f"Finding pages matching: {command.cql_expression}")
        pages = cql_processor.get_all_results(command.cql_expression)
        
        if not pages:
            click.echo("No pages found matching the CQL expression.")
            continue
        
        click.echo(f"Found {len(pages)} matching pages.")
        
        # Apply exclusion if specified
        if command.cql_exclude:
            click.echo(f"Finding pages to exclude: {command.cql_exclude}")
            excluded_pages = cql_processor.get_all_results(command.cql_exclude)
            if excluded_pages:
                original_count = len(pages)
                pages = filter_excluded_pages(pages, excluded_pages)
                click.echo(f"Excluded {original_count - len(pages)} pages. {len(pages)} pages remaining.")
        
        if dry_run:
            click.echo("DRY RUN: No changes will be made.")
            for page in pages:
                title = sanitize_text(page.title if page.title else 'Unknown')
                space = page.resultGlobalContainer.title if page.resultGlobalContainer and page.resultGlobalContainer.title else 'Unknown'
                
                if command.action == 'add':
                    click.echo(f"Would add tags {command.tags} to '{title}' (Space: {space})")
                elif command.action == 'remove':
                    click.echo(f"Would remove tags {command.tags} from '{title}' (Space: {space})")
                else:  # replace
                    click.echo(f"Would replace tags {list(command.tag_mapping.keys())} with {list(command.tag_mapping.values())} on '{title}' (Space: {space})")
            continue
        
        # Set up interactive handler if needed
        interactive_handler = None
        if command.interactive:
            interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
        
        # Process the pages
        if command.action in ('add', 'remove'):
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tags=command.tags,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        else:  # replace
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tag_mapping=command.tag_mapping,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        
        # Update total results
        for key in total_results:
            total_results[key] += results[key]
        
        # Display results for this command
        click.echo(f"\nResults for command {i+1}:")
        click.echo(f"  Total pages: {results['total']}")
        click.echo(f"  Processed: {results['processed']}")
        click.echo(f"  Skipped: {results['skipped']}")
        click.echo(f"  Successful: {results['success']}")
        click.echo(f"  Failed: {results['failed']}")
        
        # Check if aborted
        if results.get('aborted', False):
            click.echo("\nAborted by user. Stopping execution.")
            break
    
    # Display overall results
    click.echo(f"\nOverall Results:")
    click.echo(f"  Total pages: {total_results['total']}")
    click.echo(f"  Processed: {total_results['processed']}")
    click.echo(f"  Skipped: {total_results['skipped']}")
    click.echo(f"  Successful: {total_results['success']}")
    click.echo(f"  Failed: {total_results['failed']}")


@cli.command()
@click.argument('json_data')
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.pass_context
def from_json_string(ctx, json_data, abort_key):
    """
    Execute commands from a JSON string provided as a command line argument.
    
    JSON_DATA is a JSON string containing commands to execute.
    The JSON string should have the same structure as the JSON file used with the from_json command.
    
    Example:
        ctag from_json_string '{"commands":[{"action":"add","cql_expression":"space = DOCS","tags":["tag1"]}]}'
    """
    confluence = ctx.obj['CONFLUENCE']
    dry_run = ctx.obj['DRY_RUN']
    
    # Parse and validate the JSON data
    try:
        data = json.loads(json_data)
        
        # Validate and parse using Pydantic model
        commands_file = CommandsFileModel.model_validate(data)
        
        # Process commands
        commands = []
        for i, cmd_model in enumerate(commands_file.commands):
            try:
                # Create command object from the Pydantic model
                command = JSONCommand(cmd_model)
                commands.append(command)
                
            except Exception as e:
                logger.error(f"Error processing command {i+1}: {str(e)}")
                continue
    
    except json.JSONDecodeError as e:
        click.echo(f"Error parsing JSON string: {str(e)}")
        return
    except Exception as e:
        click.echo(f"Error processing JSON string: {str(e)}")
        return
    
    if not commands:
        click.echo("No valid commands found in the JSON string.")
        return
    
    click.echo(f"Found {len(commands)} commands in the JSON string.")
    
    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)
    
    # Process each command
    total_results = {
        'total': 0,
        'processed': 0,
        'skipped': 0,
        'success': 0,
        'failed': 0
    }
    
    for i, command in enumerate(commands):
        click.echo(f"\nExecuting command {i+1}/{len(commands)}: {command}")
        
        # Get matching pages
        click.echo(f"Finding pages matching: {command.cql_expression}")
        pages = cql_processor.get_all_results(command.cql_expression)
        
        if not pages:
            click.echo("No pages found matching the CQL expression.")
            continue
        
        click.echo(f"Found {len(pages)} matching pages.")
        
        # Apply exclusion if specified
        if command.cql_exclude:
            click.echo(f"Finding pages to exclude: {command.cql_exclude}")
            excluded_pages = cql_processor.get_all_results(command.cql_exclude)
            if excluded_pages:
                original_count = len(pages)
                pages = filter_excluded_pages(pages, excluded_pages)
                click.echo(f"Excluded {original_count - len(pages)} pages. {len(pages)} pages remaining.")
        
        if dry_run:
            click.echo("DRY RUN: No changes will be made.")
            for page in pages:
                title = sanitize_text(page.title if page.title else 'Unknown')
                space = page.resultGlobalContainer.title if page.resultGlobalContainer and page.resultGlobalContainer.title else 'Unknown'
                
                if command.action == 'add':
                    click.echo(f"Would add tags {command.tags} to '{title}' (Space: {space})")
                elif command.action == 'remove':
                    click.echo(f"Would remove tags {command.tags} from '{title}' (Space: {space})")
                else:  # replace
                    click.echo(f"Would replace tags {list(command.tag_mapping.keys())} with {list(command.tag_mapping.values())} on '{title}' (Space: {space})")
            continue
        
        # Set up interactive handler if needed
        interactive_handler = None
        if command.interactive:
            interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
        
        # Process the pages
        if command.action in ('add', 'remove'):
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tags=command.tags,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        else:  # replace
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tag_mapping=command.tag_mapping,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        
        # Update total results
        for key in total_results:
            total_results[key] += results[key]
        
        # Display results for this command
        click.echo(f"\nResults for command {i+1}:")
        click.echo(f"  Total pages: {results['total']}")
        click.echo(f"  Processed: {results['processed']}")
        click.echo(f"  Skipped: {results['skipped']}")
        click.echo(f"  Successful: {results['success']}")
        click.echo(f"  Failed: {results['failed']}")
        
        # Check if aborted
        if results.get('aborted', False):
            click.echo("\nAborted by user. Stopping execution.")
            break
    
    # Display overall results
    click.echo(f"\nOverall Results:")
    click.echo(f"  Total pages: {total_results['total']}")
    click.echo(f"  Processed: {total_results['processed']}")
    click.echo(f"  Skipped: {total_results['skipped']}")
    click.echo(f"  Successful: {total_results['success']}")
    click.echo(f"  Failed: {total_results['failed']}")


@cli.command()
@click.argument('csv_data')
@click.option('--abort-key', default='q', help="Key to abort all operations in interactive mode")
@click.pass_context
def from_csv_string(ctx, csv_data, abort_key):
    """
    Execute commands from a CSV string provided as a command line argument.
    
    CSV_DATA is a CSV string containing commands to execute.
    The CSV string should have the same structure as the CSV file used with the from_csv command.
    
    Example:
        ctag from_csv_string 'action,cql_expression,tags
add,space = DOCS,tag1,tag2'
    """
    import io
    
    confluence = ctx.obj['CONFLUENCE']
    dry_run = ctx.obj['DRY_RUN']
    
    # Parse and validate the CSV data
    try:
        # Create a file-like object from the CSV string
        csv_file = io.StringIO(csv_data)
        
        # Use the CSV reader
        reader = csv.DictReader(csv_file)
        
        if not reader.fieldnames:
            raise ValueError("CSV data has no header row")
        
        required_fields = ['action', 'cql_expression', 'tags']
        for field in required_fields:
            if field not in reader.fieldnames:
                raise ValueError(f"CSV data is missing required column: {field}")
        
        commands = []
        for i, row in enumerate(reader):
            try:
                # Process the row
                action = row['action'].strip().lower()
                cql_expression = row['cql_expression'].strip()
                tags_str = row['tags'].strip()
                interactive = row.get('interactive', '').strip().lower() in ('true', 'yes', '1')
                cql_exclude = row.get('cql_exclude', '').strip() or None
                
                # Parse tags based on action
                if action in ('add', 'remove'):
                    # For add/remove, tags is a comma-separated list
                    tags = [tag.strip() for tag in tags_str.split(',') if tag.strip()]
                else:  # replace
                    # For replace, tags is a comma-separated list of old=new pairs
                    tag_pairs = [pair.strip() for pair in tags_str.split(',') if pair.strip()]
                    tags = {}
                    for pair in tag_pairs:
                        if '=' in pair:
                            old, new = pair.split('=', 1)
                            tags[old.strip()] = new.strip()
                        else:
                            logger.warning(f"Ignoring invalid tag pair '{pair}' in row {i+1}")
                
                # Create a CommandModel instance
                cmd_data = {
                    "action": action,
                    "cql_expression": cql_expression,
                    "tags": tags,
                    "interactive": interactive,
                    "cql_exclude": cql_exclude
                }
                
                # Validate using Pydantic model
                cmd_model = CommandModel.model_validate(cmd_data)
                
                # Create command object
                command = JSONCommand(cmd_model)
                commands.append(command)
                
            except Exception as e:
                logger.error(f"Error processing row {i+1}: {str(e)}")
                continue
    
    except Exception as e:
        click.echo(f"Error processing CSV string: {str(e)}")
        return
    
    if not commands:
        click.echo("No valid commands found in the CSV string.")
        return
    
    click.echo(f"Found {len(commands)} commands in the CSV string.")
    
    # Initialize our processors
    cql_processor = CQLProcessor(confluence)
    tag_manager = TagManager(confluence)
    
    # Process each command
    total_results = {
        'total': 0,
        'processed': 0,
        'skipped': 0,
        'success': 0,
        'failed': 0
    }
    
    for i, command in enumerate(commands):
        click.echo(f"\nExecuting command {i+1}/{len(commands)}: {command}")
        
        # Get matching pages
        click.echo(f"Finding pages matching: {command.cql_expression}")
        pages = cql_processor.get_all_results(command.cql_expression)
        
        if not pages:
            click.echo("No pages found matching the CQL expression.")
            continue
        
        click.echo(f"Found {len(pages)} matching pages.")
        
        # Apply exclusion if specified
        if command.cql_exclude:
            click.echo(f"Finding pages to exclude: {command.cql_exclude}")
            excluded_pages = cql_processor.get_all_results(command.cql_exclude)
            if excluded_pages:
                original_count = len(pages)
                pages = filter_excluded_pages(pages, excluded_pages)
                click.echo(f"Excluded {original_count - len(pages)} pages. {len(pages)} pages remaining.")
        
        if dry_run:
            click.echo("DRY RUN: No changes will be made.")
            for page in pages:
                title = sanitize_text(page.title if page.title else 'Unknown')
                space = page.resultGlobalContainer.title if page.resultGlobalContainer and page.resultGlobalContainer.title else 'Unknown'
                
                if command.action == 'add':
                    click.echo(f"Would add tags {command.tags} to '{title}' (Space: {space})")
                elif command.action == 'remove':
                    click.echo(f"Would remove tags {command.tags} from '{title}' (Space: {space})")
                else:  # replace
                    click.echo(f"Would replace tags {list(command.tag_mapping.keys())} with {list(command.tag_mapping.values())} on '{title}' (Space: {space})")
            continue
        
        # Set up interactive handler if needed
        interactive_handler = None
        if command.interactive:
            interactive_handler = InteractiveHandler(default_response=True, abort_value=abort_key)
        
        # Process the pages
        if command.action in ('add', 'remove'):
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tags=command.tags,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        else:  # replace
            results = tag_manager.process_pages(
                pages=pages,
                action=command.action,
                tag_mapping=command.tag_mapping,
                interactive=command.interactive,
                interactive_handler=interactive_handler
            )
        
        # Update total results
        for key in total_results:
            total_results[key] += results[key]
        
        # Display results for this command
        click.echo(f"\nResults for command {i+1}:")
        click.echo(f"  Total pages: {results['total']}")
        click.echo(f"  Processed: {results['processed']}")
        click.echo(f"  Skipped: {results['skipped']}")
        click.echo(f"  Successful: {results['success']}")
        click.echo(f"  Failed: {results['failed']}")
        
        # Check if aborted
        if results.get('aborted', False):
            click.echo("\nAborted by user. Stopping execution.")
            break
    
    # Display overall results
    click.echo(f"\nOverall Results:")
    click.echo(f"  Total pages: {total_results['total']}")
    click.echo(f"  Processed: {total_results['processed']}")
    click.echo(f"  Skipped: {total_results['skipped']}")
    click.echo(f"  Successful: {total_results['success']}")
    click.echo(f"  Failed: {total_results['failed']}")
