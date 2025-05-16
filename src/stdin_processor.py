#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Stdin processor module for reading tag commands from stdin.

This module provides functionality for reading tag commands from stdin
as part of the ctag CLI tool, supporting both JSON and CSV formats.
"""

import sys
import json
import csv
import io
import logging
from typing import List, Dict, Optional, Any, Union

from src.json_processor import JSONCommand, CommandModel, CommandsFileModel
from src.csv_processor import read_commands_from_csv

logger = logging.getLogger(__name__)

def read_json_from_stdin() -> List[JSONCommand]:
    """Read JSON commands from stdin.
    
    Returns:
        List of JSONCommand objects
    """
    try:
        # Read from stdin
        data = json.load(sys.stdin)
        
        # Validate and parse using Pydantic model (handle both v1 and v2)
        if hasattr(CommandsFileModel, 'model_validate'):
            commands_file = CommandsFileModel.model_validate(data)
        elif hasattr(CommandsFileModel, 'parse_obj'):
            commands_file = CommandsFileModel.parse_obj(data)
        else:
            commands_file = CommandsFileModel(**data)
        
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
        
        return commands
    
    except json.JSONDecodeError as e:
        logger.error(f"Error parsing JSON from stdin: {str(e)}")
        raise
    except Exception as e:
        logger.error(f"Error reading JSON from stdin: {str(e)}")
        raise

def read_csv_from_stdin() -> List[JSONCommand]:
    """Read CSV commands from stdin.
    
    Returns:
        List of JSONCommand objects
    """
    try:
        # Read from stdin and create a file-like object
        stdin_data = sys.stdin.read()
        csv_file = io.StringIO(stdin_data)
        
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
                
                # Validate using Pydantic model (handle both v1 and v2)
                if hasattr(CommandModel, 'model_validate'):
                    cmd_model = CommandModel.model_validate(cmd_data)
                elif hasattr(CommandModel, 'parse_obj'):
                    cmd_model = CommandModel.parse_obj(cmd_data)
                else:
                    cmd_model = CommandModel(**cmd_data)
                
                # Create command object
                command = JSONCommand(cmd_model)
                commands.append(command)
                
            except Exception as e:
                logger.error(f"Error processing row {i+1}: {str(e)}")
                continue
        
        return commands
    
    except Exception as e:
        logger.error(f"Error reading CSV from stdin: {str(e)}")
        raise

def is_stdin_available() -> bool:
    """Check if stdin has data available.
    
    Returns:
        True if stdin has data, False otherwise
    """
    import os
    import stat
    
    mode = os.fstat(0).st_mode
    return stat.S_ISFIFO(mode) or stat.S_ISREG(mode)
