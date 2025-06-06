#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Debug script for CQL queries.
"""

import os
import sys
import json
from dotenv import load_dotenv
from atlassian import Confluence

# Load environment variables
load_dotenv()

# Check for required environment variables
required_vars = ["CONFLUENCE_URL", "CONFLUENCE_USERNAME", "ATLASSIAN_TOKEN"]
missing = [var for var in required_vars if not os.environ.get(var)]
if missing:
    print(f"Missing required environment variables: {', '.join(missing)}")
    sys.exit(1)

# Create Confluence client
confluence = Confluence(
    url=os.environ["CONFLUENCE_URL"],
    username=os.environ["CONFLUENCE_USERNAME"],
    password=os.environ["ATLASSIAN_TOKEN"],
    cloud=True,
)

# CQL query to execute
cql_expression = "space = itkb AND title ~ 'Apple - ' AND title ~ 'Macbook'"
expand = "space,metadata.labels,version,content"

# Execute the query
print(f"Executing CQL query: {cql_expression}")
try:
    results = confluence.cql(cql=cql_expression, limit=1, expand=expand)

    # Print the raw results structure
    print("\nRaw results structure:")
    print(json.dumps(results))

    # Print details about each page
    pages = results.get("results", [])
    print(f"\nFound {len(pages)} pages")

    for i, page in enumerate(pages):
        print(f"\nPage {i+1}:")
        print(f"  Title: {page.get('title', 'Unknown')}")
        print(f"  ID: {page.get('id', 'No ID')}")
        print(f"  Content ID: {page.get('content', {}).get('id', 'No Content ID')}")
        print(f"  Space: {page.get('space', {}).get('key', 'Unknown')}")
        print(f"  Available fields: {', '.join(page.keys())}")

except Exception as e:
    print(f"Error executing CQL query: {str(e)}")
    sys.exit(1)
