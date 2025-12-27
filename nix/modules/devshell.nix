{ inputs, ... }:
{
  perSystem =
    { config
    , self'
    , pkgs
    , lib
    , ...
    }:
    {
      packages.ctag-cli = pkgs.writeShellScriptBin "ctag-cli" ''
        exec cargo run --quiet -- "$@"
      '';

      packages.ctag-warning = pkgs.writeShellScriptBin "ctag" ''
        echo "You are using the 'ctag' binary directly in the Nushell environment."
        echo "To use the structured Nushell integration, run: use nu/ctag.nu"
        echo "Then try your command again."
      '';

      devShells.default = pkgs.mkShell {
        name = "ctag-shell";
        inputsFrom = [
          self'.devShells.rust
          config.pre-commit.devShell # See ./nix/modules/pre-commit.nix
        ];
        packages = with pkgs; [
          just
          nixd # Nix language server
          bacon
          omnix
          asciinema
          vhs
          self'.packages.ctag-cli
          # Alias ctag-cli to ctag for convenience in default shell
          (writeShellScriptBin "ctag" ''
            exec ${self'.packages.ctag-cli}/bin/ctag-cli "$@"
          '')
        ];
      };

      devShells.nu = pkgs.mkShell {
        name = "ctag-nu";
        inputsFrom = [
          # We inherit most inputs but NOT the packages list from default shell directly
          # because we want to exclude the 'ctag' binary alias.
          self'.devShells.rust
          config.pre-commit.devShell
        ];
        packages = with pkgs; [
          just
          self'.packages.ctag-cli
          self'.packages.ctag-warning # Warns user if they run 'ctag' without loading module
          nushell
        ];
        shellHook = ''
                    # Remove any 'ctag' binary from PATH that might have been inherited (e.g. from rust devshell)
                    # This ensures that 'ctag' is not available unless the user loads the nushell module.
                    # We filter out paths ending in 'ctag-<version>/bin' or similar.
                    export PATH=$(echo "$PATH" | tr ':' '\n' | grep -v "ctag-[0-9].*/bin" | tr '\n' ':')

                    # Only start interactive nu session if we are in a terminal and not running a command
                    if [[ $- == *i* ]]; then

                      # Create a custom config that attempts to load the user's config and then our module
                      # We use a unique name to avoid conflicts
                      CONFIG_PATH="$PWD/.ctag_nix_config.nu"

                      # Resolve the default config directory using nushell itself
                      USER_CONFIG_DIR=$(nu -c '$nu.default-config-dir' | tr -d '\n')
                      USER_CONFIG_FILE="$USER_CONFIG_DIR/config.nu"

                      # Check if user config exists inside BASH, so we only generate the source command if valid.
                      # Nushell 'source' is parse-time and fails if file is missing, even inside 'if'.
                      USER_CONFIG_SOURCE_CMD=""
                      if [ -f "$USER_CONFIG_FILE" ]; then
                        USER_CONFIG_SOURCE_CMD="source \"$USER_CONFIG_FILE\""
                      fi

                      # We use a heredoc to create the config file
                      cat <<EOF > "$CONFIG_PATH"
          # Disable banner first
          \$env.config = ( \$env.config? | default {} | upsert show_banner false )

          print "Loading ctag environment..."

          # Load user config if it was found
          $USER_CONFIG_SOURCE_CMD

          # Load the ctag module
          # Crucial: Variable expansion happens here in Bash to put the literal path in the file
          use "$PWD/nu/ctag.nu"

          print "ctag module loaded."
          EOF
                      export NU_CONFIG_FILE="$CONFIG_PATH"
                      echo "Launching Nushell with ctag integration..."
                      # Using --config to be explicit, though env var should work
                      exec nu --config "$CONFIG_PATH"
                    fi
        '';
      };
    };
}
