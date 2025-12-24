# Development

## Development Environment

To enter the development environment, run:

```bash
direnv allow .
```

This requires `direnv` to be installed as well as nix (with flakes) to be installed and enabled.

The rust environment can be interacted with using cargo, but it is recommended to use nix environment commands.

### Environment variables

Create a `.env` file in the project root with your Confluence credentials:

```env
ATLASSIAN_URL=https://your-domain.atlassian.net
ATLASSIAN_USERNAME=your-email@example.com
ATLASSIAN_TOKEN=your-api-token
```

To generate an API token, visit: https://id.atlassian.com/manage-profile/security/api-tokens

## Docs

Documentation is auto-generated from the source code. To view it:

```bash
cargo doc --open
```

The demo gif can be generated using `vhs`:

```bash
nix develop --accept-flake-config --command vhs demo.tape
```

## CI/CD

CI/CD is handled by Omnix and GitHub Actions. The workflow is defined in `.github/workflows/ci.yml` as well as the `omnix.yaml` file.

## Testing

### End-to-End Tests

The end-to-end tests are located in the `e2e` directory. They use the `assert_cmd` crate to run the `ctag` binary and verify its behavior.

To run the end-to-end tests, use the following command:

```bash
cargo test --test e2e_basic --test e2e_bulk --test e2e_advanced -- --ignored
```

## Unit Tests

Unit tests are located with their respective modules in the `src` directory.

To run the unit tests, use the following command:

```bash
cargo test
```
