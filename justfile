default:
    @just --list

# Run pre-commit hooks on all files, including autoformatting
pre-commit-all:
    pre-commit run --all-files

# Run 'cargo run' on the project
run *ARGS:
    cargo run {{ARGS}}

# Run 'bacon' to run the project (auto-recompiles)
watch *ARGS:
	bacon --job run -- -- {{ ARGS }}

unit-test:
	nix develop --accept-flake-config --command "cargo test"

e2e-test:
	nix develop --accept-flake-config --command "cargo test --test e2e_basic --test e2e_bulk --test e2e_advanced --test e2e_regex -- --ignored"
