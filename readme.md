# ctag

A command line program for synchronizing Confluence pages with a local file system.

## Usage

```sh
ctag --help
```

## Features

- [x] Pull Confluence pages (and their children) to a local filesystem path.

- [x] Push local changes back to Confluence, creating or updating pages as needed.

- [x] Local confluence representation contains version metadata, attachments

- [x] Support for attachments, allowing files to be downloaded and uploaded along with page content.

- [ ] Edit metadata (title, version, etc...) locally and have page metadata updated on the remote upon push.

- [ ] Better support for pushing to spaces with no matching page id (i.e. creating a new page) verses updating a space with an existing matching page id (i.e. updating a page). 

```sh
# if creating a new page it will preserve the version history and owner
ctag push --preserve ... 

# if an existing page is found that is being updated, overwrite the remote metadata in favor of the local metadata. If there's no local metadata, remote metadata is "reset" 
ctag push --overwrite ... 
```

- [ ] Better support for true sync features and edge cases. 

```sh
# Pull only pages that have changed since the last sync
ctag pull --skip-unchanged https://your-instance.atlassian.net/wiki/spaces/SPACE/pages/123456 ./local-pages

# Push only pages that have changed locally
ctag push --skip-unchanged ./local-pages https://your-instance.atlassian.net/wiki/spaces/SPACE/pages/123456

# Specify a custom configuration file
ctag pull --config /path/to/config.json https://your-instance.atlassian.net/wiki/spaces/SPACE/pages/123456 ./local-pages
```

- [ ] Parallelism for larger migrations

- [ ] Better dry run visualization


## Installation from Source

```sh
git clone https://github.com/ocasazza/ctag.git
cd ctag
pip install -e .
```


## Configuration

ctag can be configured using environment variables, a .env file, or a configuration file:

### .env File

Create a `.env` file in your current directory:

```
CONFLUENCE_URL=https://your-instance.atlassian.net
CONFLUENCE_USERNAME=your-email@example.com
ATLASSIAN_TOKEN=your-api-token
```