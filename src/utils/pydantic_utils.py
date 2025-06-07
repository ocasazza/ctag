#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Pydantic utilities module.

This module provides utility functions for working with Pydantic models,
including creating models from JSON schemas.
"""

import json
from typing import Any, Dict, Optional, Type, Union

from pydantic import BaseModel, create_model


def create_model_from_schema(schema: Dict[str, Any], model_name: str) -> Type[BaseModel]:
    """Create a Pydantic model from a JSON schema.

    Args:
        schema: The JSON schema to create the model from
        model_name: The name of the model to create

    Returns:
        A Pydantic model class
    """
    # Extract properties from the schema
    properties = schema.get("properties", {})
    required = schema.get("required", [])

    # Create field definitions for the model
    fields = {}
    for field_name, field_schema in properties.items():
        field_type = _get_field_type(field_schema)
        default = ... if field_name in required else None
        fields[field_name] = (field_type, default)

    # Create the model
    model = create_model(model_name, **fields)

    return model


def _get_field_type(field_schema: Dict[str, Any]) -> Any:
    """Get the Python type for a JSON schema field.

    Args:
        field_schema: The JSON schema for the field

    Returns:
        The Python type for the field
    """
    schema_type = field_schema.get("type")

    if schema_type == "string":
        return str
    elif schema_type == "integer":
        return int
    elif schema_type == "number":
        return float
    elif schema_type == "boolean":
        return bool
    elif schema_type == "array":
        items = field_schema.get("items", {})
        if items.get("type") == "object" and "properties" in items:
            # For arrays of objects, create a nested model
            item_model = create_model_from_schema(items, f"{field_schema.get('title', 'Item')}Model")
            return list[item_model]
        else:
            item_type = _get_field_type(items)
            return list[item_type]
    elif schema_type == "object":
        return Dict[str, Any]
    elif schema_type is None and "anyOf" in field_schema:
        # Handle anyOf by using the first type
        return _get_field_type(field_schema["anyOf"][0])
    elif schema_type is None and "oneOf" in field_schema:
        # Handle oneOf by using Any (since it can be multiple types)
        return Any
    elif isinstance(schema_type, list):
        # Handle type arrays like ["string", "null"]
        non_null_types = [t for t in schema_type if t != "null"]
        if non_null_types:
            # For nullable types, use Optional
            base_type = _get_field_type({"type": non_null_types[0]})
            return Optional[base_type]
        return Any
    else:
        # Default to Any for unknown types
        return Any
