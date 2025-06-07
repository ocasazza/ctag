#!/usr/bin/env python3
# -*- coding: utf-8 -*-

from setuptools import setup, find_packages

with open("readme.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

with open("requirements.txt", "r", encoding="utf-8") as fh:
    requirements = fh.read().splitlines()

setup(
    name="ctag",
    version="0.1.0",
    author="Olive Casazza",
    author_email="olive.casazza@schrodinger.com",
    description="A command line tool for managing tags on Confluence pages in bulk",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/ocasazza/ctag",
    packages=find_packages(),
    include_package_data=True,
    package_data={
        "src.models": ["*.json"],
    },
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Topic :: Utilities",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    python_requires=">=3.6",
    install_requires=requirements,
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "pytest-cov>=4.0.0",
            "flake8>=6.0.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "ctag=src.main:main",
        ],
    },
)
