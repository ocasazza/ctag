#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Stdin processor module for reading tag commands from stdin.

This module provides functionality for reading tag commands from stdin
as part of the ctag CLI tool, supporting JSON format.
"""

import json
import logging
import sys
from typing import Any, Dict, List, Optional, Union

from src.json_processor import CommandModel, CommandsFileModel, JSONCommand

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
        if hasattr(CommandsFileModel, "model_validate"):
            commands_file = CommandsFileModel.model_validate(data)
        elif hasattr(CommandsFileModel, "parse_obj"):
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


def is_stdin_available() -> bool:
    """Check if stdin has data available.

    Returns:
        True if stdin has data, False otherwise
    """
    import os
    import stat

    mode = os.fstat(0).st_mode
    return stat.S_ISFIFO(mode) or stat.S_ISREG(mode)
