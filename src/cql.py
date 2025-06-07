#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
CQL (Confluence Query Language) processor module.

This module provides functionality for executing CQL queries and retrieving
matching Confluence pages as part of the ctag CLI tool.
"""

import json
import logging
from typing import Any, Dict, List, Optional, Union

from src.models.search_results import SearchResultItem

logger = logging.getLogger(__name__)


class CQLProcessor:
    """Processes CQL queries and retrieves matching Confluence pages."""

    def __init__(self, confluence):
        """Initialize the CQLProcessor with a Confluence client.

        Args:
            confluence: An authenticated Confluence client instance
        """
        self.confluence = confluence

    def execute_query(
        self,
        cql_expression: str,
        start: int = 0,
        limit: Optional[int] = None,
        expand: Optional[str] = None,
    ) -> List[SearchResultItem]:
        """Execute a CQL query and return matching pages.

        Args:
            cql_expression: The CQL query expression
            start: Starting index for pagination
            limit: Maximum number of results to return
            expand: Comma-separated list of properties to expand

        Returns:
            List of page dictionaries matching the query
        """
        if not expand:
            # Default expansions to get useful page information
            expand = "space,metadata.labels,version"

        try:
            logger.info(f"Executing CQL query: {cql_expression}")
            results = self.confluence.cql(cql=cql_expression, start=start, limit=limit, expand=expand)

            # Add debug logging
            logger.info(f"Results type: {type(results)}")

            # Handle different response formats
            if isinstance(results, dict):
                # Standard response format
                results_list = results.get("results", [])
                logger.info(f"Results keys: {results.keys()}")
                logger.info(f"Results_list type: {type(results_list)}")
            elif isinstance(results, list):
                # Some API versions might return a list directly
                results_list = results
                logger.info("Results is already a list")
            elif isinstance(results, str):
                # Handle string response (possibly JSON)
                try:
                    parsed_results = json.loads(results)
                    if isinstance(parsed_results, dict):
                        results_list = parsed_results.get("results", [])
                    else:
                        results_list = parsed_results if isinstance(parsed_results, list) else []
                    logger.info(
                        f"Parsed results from string, got {
        len(results_list)} items"
                    )
                except json.JSONDecodeError:
                    logger.error("Failed to parse results string as JSON")
                    results_list = []
            else:
                # Fallback for unexpected types
                logger.error(f"Unexpected results type: {type(results)}")
                results_list = []

            # Convert each result item to a SearchResultItem object
            pages: List[SearchResultItem] = []
            for item in results_list:
                try:
                    # Try to use model_validate (Pydantic v2) or parse_obj
                    # (Pydantic v1)
                    if hasattr(SearchResultItem, "model_validate"):
                        pages.append(SearchResultItem.model_validate(item))
                    elif hasattr(SearchResultItem, "parse_obj"):
                        pages.append(SearchResultItem.parse_obj(item))
                    else:
                        # Fallback to direct instantiation
                        pages.append(SearchResultItem(**item))
                except Exception as e:
                    logger.error(f"Error creating SearchResultItem: {str(e)}")
                    # Try with a more lenient approach - create a dict with only the
                    # fields we need
                    try:
                        # Extract just the essential fields we need
                        minimal_item = {
                            "content": {"id": item.get("content", {}).get("id")},
                            "title": item.get("title"),
                        }
                        pages.append(SearchResultItem(**minimal_item))
                        logger.info(
                            f"Created SearchResultItem with minimal data for {
        minimal_item['title']}"
                        )
                    except Exception as e2:
                        logger.error(
                            f"Failed to create even minimal SearchResultItem: {
        str(e2)}"
                        )
                        logger.error(f"Item: {item}")

            logger.info(f"CQL query returned {len(pages)} results")

            return pages

        except Exception as e:
            logger.error(
                f"Error executing CQL query '{cql_expression}': {
        str(e)}"
            )
            return []

    def get_all_results(
        self,
        cql_expression: str,
        expand: Optional[str] = None,
        batch_size: int = 100,
    ) -> List[SearchResultItem]:
        """Get all results for a CQL query, handling pagination.

        Args:
            cql_expression: The CQL query expression
            expand: Comma-separated list of properties to expand
            batch_size: Number of results to fetch per request

        Returns:
            List of all page dictionaries matching the query
        """
        all_pages = []
        start = 0

        while True:
            batch = self.execute_query(
                cql_expression=cql_expression,
                start=start,
                limit=batch_size,
                expand=expand,
            )

            if not batch:
                break

            all_pages.extend(batch)

            if len(batch) < batch_size:
                break

            start += batch_size

        return all_pages
