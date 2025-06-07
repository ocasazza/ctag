#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
JSON processor module for reading tag commands from JSON files.

This module provides functionality for reading tag commands from JSON files
as part of the ctag CLI tool, with schema validation to enforce types.
"""

import json
import logging
from typing import Any, Dict, List, Optional, Union

from src.models import CommandModel, CommandsFileModel

logger = logging.getLogger(__name__)


class JSONCommand:
    """Represents a command read from a JSON file."""

    def __init__(self, command_model: CommandModel):
        """Initialize a JSONCommand from a CommandModel.

        Args:
            command_model: The CommandModel instance
        """
        self.action = command_model.action.lower().strip()
        self.cql_expression = command_model.cql_expression.strip()
        self.interactive = (
            command_model.interactive
            if hasattr(command_model, "interactive") and command_model.interactive is not None
            else False
        )
        self.cql_exclude = (
            command_model.cql_exclude.strip()
            if hasattr(command_model, "cql_exclude") and command_model.cql_exclude
            else None
        )

        # Set tags or tag_mapping based on action
        if self.action in ("add", "remove"):
            if isinstance(command_model.tags, list):
                self.tags = command_model.tags
                self.tag_mapping = None
            else:
                raise ValueError(
                    f"For '{
        self.action}' action, 'tags' must be a list of strings"
                )
        else:  # replace
            if isinstance(command_model.tags, dict):
                self.tags = None
                self.tag_mapping = command_model.tags
            else:
                raise ValueError(
                    f"For '{
        self.action}' action, 'tags' must be a dictionary mapping old tags to new tags"
                )

    def __str__(self) -> str:
        """Return a string representation of the command."""
        if self.action in ("add", "remove"):
            tags_info = f"tags={self.tags}"
        else:
            tags_info = f"tag_mapping={self.tag_mapping}"

        exclude_info = (
            f", exclude={
    self.cql_exclude}"
            if self.cql_exclude
            else ""
        )
        interactive_info = ", interactive=True" if self.interactive else ""

        return f"JSONCommand(action={
    self.action}, cql={
        self.cql_expression}, {tags_info}{exclude_info}{interactive_info})"


def read_commands_from_json(json_file_path: str) -> List[JSONCommand]:
    """Read commands from a JSON file.

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
                "cql_expression": "space = DOCS",
                "tags": {
                    "old1": "new1",
                    "old2": "new2"
                },
                "interactive": true,
                "cql_exclude": "space = ARCHIVE"
            }
        ]
    }

    Args:
        json_file_path: Path to the JSON file

    Returns:
        List of JSONCommand objects
    """
    commands = []

    try:
        with open(json_file_path, "r") as jsonfile:
            data = json.load(jsonfile)

        # Validate and parse using Pydantic model (handle both v1 and v2)
        if hasattr(CommandsFileModel, "model_validate"):
            commands_file = CommandsFileModel.model_validate(data)
        elif hasattr(CommandsFileModel, "parse_obj"):
            commands_file = CommandsFileModel.parse_obj(data)
        else:
            commands_file = CommandsFileModel(**data)

        # Process commands
        for i, cmd_model in enumerate(commands_file.commands):
            try:
                # Create command object from the Pydantic model
                command = JSONCommand(cmd_model)
                commands.append(command)

            except Exception as e:
                logger.error(f"Error processing command {i+1}: {str(e)}")
                continue

    except json.JSONDecodeError as e:
        logger.error(f"Error parsing JSON file '{json_file_path}': {str(e)}")
        raise
    except Exception as e:
        logger.error(f"Error reading JSON file '{json_file_path}': {str(e)}")
        raise

    return commands


def validate_json_file(json_file_path: str) -> bool:
    """Validate a JSON file against the schema.

    Args:
        json_file_path: Path to the JSON file

    Returns:
        True if valid, False otherwise
    """
    try:
        with open(json_file_path, "r") as jsonfile:
            data = json.load(jsonfile)

        # Validate using Pydantic model (handle both v1 and v2)
        if hasattr(CommandsFileModel, "model_validate"):
            CommandsFileModel.model_validate(data)
        elif hasattr(CommandsFileModel, "parse_obj"):
            CommandsFileModel.parse_obj(data)
        else:
            CommandsFileModel(**data)
        return True

    except Exception as e:
        logger.error(f"Validation error for '{json_file_path}': {str(e)}")
        return False
