# Atlassian API Python Boilerplate

Some boilerplate code for making requests to Atlassian products via python. 

For examples' sake, this code assumes you're building a command line tool called `atool`

## Usage

```sh
atool --help
```

## Installation from Source

```sh
git clone https://github.com/ocasazza/atool.git
cd atool
pip install -e .
```

## Usage Configuration

atool can be configured using environment variables, a .env file, or a configuration file:

### .env File

Create a `.env` file in your current directory:

```
CONFLUENCE_URL=https://your-instance.atlassia n.net
CONFLUENCE_USERNAME=your-email@example.com
ATLASSIAN_TOKEN=your-api-token
```

To generate Atlassian tokens, see Atlassian's documentation for managing API keys: [Manage Api Tokens For Your Atlassian Account](https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/)
