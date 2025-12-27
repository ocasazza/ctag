# ctag - Confluence Tag Manager

A command-line tool for managing tags on Confluence pages in bulk, written in Rust with Nix for environment management.

![ctag demo](demo.gif)

## Usage

See [./docs/usage.md ](./docs/usage.md) for usage instructions.

## Development

### Devshells

This project uses Nix for reproducible development environments.

*   **Default (Nushell)**: `nix develop`
    Pre-configured environment that automatically loads the `ctag` Nushell module. (see `docs/nu.md`).
*   **Bash**: `nix develop .#bash`
    Standard Rust environment with `cargo`, `rustc`, and build dependencies.

### Run tests

```bash
cargo test
```

### Format code

```bash
cargo fmt
```

### Lint code

```bash
cargo clippy
```

### Using just

The project includes a `justfile` for common tasks:

```bash
just build    # Build the project
just run      # Run the project
just test     # Run tests
just fmt      # Format code
just check    # Run clippy
```

## Building with Nix

To build the project using Nix:

```bash
nix build
```

The binary will be available in `./result/bin/ctag`.

## Documentation

Documentation is auto-generated from the source code. To view it:

```bash
cargo doc --open
```

## License

See [LICENSE](LICENSE) file for details.

## Migration from Python

This project was migrated from Python to Rust. The CLI interface remains exactly the same, so all existing scripts and workflows should continue to work without modification.
