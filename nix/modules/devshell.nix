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
          nushell
          (writeShellScriptBin "ctag" ''
            exec cargo run --quiet -- "$@"
          '')
        ];
      };

      devShells.nu = pkgs.mkShell {
        name = "ctag-nu";
        inputsFrom = [ self'.devShells.default ];
        shellHook = ''
          echo "🚀 Welcome to the ctag Nushell environment!"
          echo "💡 Tip: Run 'use nu/ctag.nu' to load the integration."
          exec nu
        '';
      };
    };
}
