#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Command models module.

This module provides Pydantic models for command validation and type checking,
generated from the commands JSON schema.
"""

import json
import os
from typing import Any, Dict

from src.utils.pydantic_utils import create_model_from_schema

# Load the commands schema directly
schema_path = os.path.join(os.path.dirname(__file__), "./commands.json")
with open(schema_path, "r") as f:
    commands_schema = json.load(f)

command_schema = commands_schema["properties"]["commands"]["items"]

# Create Pydantic models from JSON schema
# This automatically generates models with all the fields defined in the schema
CommandModel = create_model_from_schema(
    schema=command_schema, model_name="CommandModel"
)

CommandsFileModel = create_model_from_schema(
    schema=commands_schema, model_name="CommandsFileModel"
)

# Export the models for use in other modules
__all__ = ["CommandModel", "CommandsFileModel"]
