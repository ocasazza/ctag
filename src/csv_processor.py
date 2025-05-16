#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
CSV processor module for reading tag commands from CSV files.

This module provides functionality for reading tag commands from CSV files
as part of the ctag CLI tool.
"""

import csv
import logging
from typing import List, Dict, Optional, Any, Union
import os
import json
from pydantic import BaseModel, create_model_from_schema

from src.json_processor import JSONCommand, CommandModel

logger = logging.getLogger(__name__)

def read_commands_from_csv(csv_file_path: str) -> List[JSONCommand]:
    """Read commands from a CSV file.
    
    The CSV file should have the following columns:
    - action: The action to perform (add, remove, or replace)
    - cql_expression: The CQL query to select pages
    - tags: For add/remove actions, a comma-separated list of tags;
            for replace actions, a comma-separated list of old=new pairs
    - interactive: (optional) Whether to confirm each action interactively (true/false)
    - cql_exclude: (optional) CQL expression to match pages that should be excluded
    
    Args:
        csv_file_path: Path to the CSV file
        
    Returns:
        List of JSONCommand objects
    """
    commands = []
    
    try:
        with open(csv_file_path, 'r', newline='') as csvfile:
            reader = csv.DictReader(csvfile)
            
            if not reader.fieldnames:
                raise ValueError("CSV file has no header row")
            
            required_fields = ['action', 'cql_expression', 'tags']
            for field in required_fields:
                if field not in reader.fieldnames:
                    raise ValueError(f"CSV file is missing required column: {field}")
            
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
        logger.error(f"Error reading CSV file '{csv_file_path}': {str(e)}")
        raise
    
    return commands


def validate_csv_file(csv_file_path: str) -> bool:
    """Validate a CSV file.
    
    Args:
        csv_file_path: Path to the CSV file
        
    Returns:
        True if valid, False otherwise
    """
    try:
        with open(csv_file_path, 'r', newline='') as csvfile:
            reader = csv.DictReader(csvfile)
            
            if not reader.fieldnames:
                logger.error(f"CSV file '{csv_file_path}' has no header row")
                return False
            
            required_fields = ['action', 'cql_expression', 'tags']
            for field in required_fields:
                if field not in reader.fieldnames:
                    logger.error(f"CSV file '{csv_file_path}' is missing required column: {field}")
                    return False
            
            # Check first row to validate structure
            first_row = next(reader, None)
            if first_row:
                action = first_row['action'].strip().lower()
                if action not in ('add', 'remove', 'replace'):
                    logger.error(f"Invalid action '{action}' in CSV file '{csv_file_path}'")
                    return False
        
        return True
        
    except Exception as e:
        logger.error(f"Error validating CSV file '{csv_file_path}': {str(e)}")
        return False


def create_example_csv(output_path: str) -> bool:
    """Create an example CSV file.
    
    Args:
        output_path: Path to write the example CSV file to
        
    Returns:
        True if successful, False otherwise
    """
    try:
        with open(output_path, 'w', newline='') as csvfile:
            writer = csv.writer(csvfile)
            
            # Write header
            writer.writerow(['action', 'cql_expression', 'tags', 'interactive', 'cql_exclude'])
            
            # Write example rows
            writer.writerow(['add', 'space = DOCS AND title ~ "Project"', 'documentation,project', 'false', ''])
            writer.writerow(['remove', 'space = ARCHIVE', 'outdated,deprecated', 'true', 'label = "keep"'])
            writer.writerow(['replace', 'space = DOCS AND label = "old-tag"', 'old-tag=new-tag,typo=correct', 'false', 'label = "do-not-modify"'])
        
        logger.info(f"Created example CSV file at {output_path}")
        return True
        
    except Exception as e:
        logger.error(f"Error creating example CSV file: {str(e)}")
        return False
