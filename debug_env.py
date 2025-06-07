#!/usr/bin/env python3
"""
Debug script to test environment variable loading.
"""
import os
from dotenv import load_dotenv

print("=== Environment Debug ===")
print(f"Current working directory: {os.getcwd()}")
print(f".env file exists: {os.path.exists('.env')}")

# Test the exact same code as in main.py
print("\n=== Loading .env file ===")
load_dotenv(dotenv_path=".env")

# Check if variables are loaded
print("\n=== Environment Variables ===")
print(f"ATLASSIAN_URL: {os.environ.get('ATLASSIAN_URL', 'NOT FOUND')}")
print(f"ATLASSIAN_USERNAME: {os.environ.get('ATLASSIAN_USERNAME', 'NOT FOUND')}")
print(f"ATLASSIAN_TOKEN: {os.environ.get('ATLASSIAN_TOKEN', 'NOT FOUND')}")

# Test the check_environment function logic
REQUIRED_ENV_VARS = {
    "ATLASSIAN_URL": "The base URL of your Confluence instance",
    "ATLASSIAN_USERNAME": "Your Confluence username",
    "ATLASSIAN_TOKEN": "Your Atlassian API token",
}

print("\n=== Environment Check ===")
missing = []
for var, desc in REQUIRED_ENV_VARS.items():
    value = os.environ.get(var)
    if not value:
        missing.append(f"{var} - {desc}")
        print(f"❌ {var}: MISSING")
    else:
        print(f"✅ {var}: {value[:20]}..." if len(value) > 20 else f"✅ {var}: {value}")

if missing:
    print(f"\n❌ Missing variables: {len(missing)}")
    for var in missing:
        print(f"  {var}")
else:
    print("\n✅ All environment variables found!")
