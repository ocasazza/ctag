{
  description = "ctag - A command line tool for managing tags on Confluence pages in bulk";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    ,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Python package set
        pythonPackages = pkgs.python3Packages;

        # Runtime dependencies
        runtimeDeps = with pythonPackages; [
          atlassian-python-api
          click
          tqdm
          python-dotenv
          jsonschema
          pydantic
        ];

        # Development dependencies
        devDeps = with pythonPackages; [
          pytest
          pytest-cov
          pytest-mock
          pytest-xdist
          flake8
          black
          isort
          mypy
          autopep8
        ];

        # Python environment with all packages
        pythonEnv = pkgs.python3.withPackages (ps: runtimeDeps ++ devDeps);

        # The ctag package - uses pyproject.toml for configuration
        ctag = pythonPackages.buildPythonPackage rec {
          pname = "ctag";
          version = "0.1.0";
          format = "pyproject";

          src = builtins.path {
            path = ./.;
            name = "ctag-source";
          };

          nativeBuildInputs = with pythonPackages; [
            setuptools
            wheel
            build
            pip
          ];

          build-system = with pythonPackages; [
            setuptools
            wheel
          ];

          propagatedBuildInputs = runtimeDeps;

          checkInputs = devDeps;

          # Skip tests during build (they require Confluence credentials)
          doCheck = false;

          meta = with pkgs.lib; {
            description = "A command line tool for managing tags on Confluence pages in bulk";
            homepage = "https://github.com/ocasazza/ctag";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.unix;
          };
        };
      in
      {
        # Default package
        packages.default = ctag;
        packages.ctag = ctag;

        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pythonEnv
            pkgs.python3Packages.pip # Keep pip available for edge cases
            ctag # Include the built ctag package
          ];

          shellHook = ''
            # Set up Python path to include source directory
            export PYTHONPATH="${./.}/src:$PYTHONPATH"

            # Create .env file if it doesn't exist
            if [ ! -f .env ]; then
              cp .env.example .env
              echo "Created .env file from .env.example - please configure your Confluence credentials"
            fi

            # Install the local package in development mode (editable install)
            pip install -e . --no-deps 2>/dev/null || true
          '';
        };

        # Apps
        apps.default = flake-utils.lib.mkApp {
          drv = ctag;
          exePath = "/bin/ctag";
        };

        apps.ctag = flake-utils.lib.mkApp {
          drv = ctag;
          exePath = "/bin/ctag";
        };

        # Formatter - custom script that formats both Nix and Python files
        formatter = pkgs.writeShellScriptBin "fmt" ''
          set -euo pipefail

          # Format Nix files
          echo "Formatting Nix files..."
          ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt flake.nix

          # Sort imports with isort first (before black)
          echo "Sorting imports with isort..."
          ${pythonPackages.isort}/bin/isort src/ tests/ --profile black

          # Fix line length issues with autopep8 first
          echo "Fixing line length issues with autopep8..."
          ${pythonPackages.autopep8}/bin/autopep8 --in-place --recursive \
            --max-line-length=120 \
            --aggressive --aggressive \
            --select=E501 \
            src/ tests/

          # Format Python files with black
          echo "Formatting Python files with black..."
          ${pythonPackages.black}/bin/black src/ tests/ --line-length 120

          # Check Python files with flake8 (consistent with pyproject.toml)
          echo "Checking Python files with flake8..."
          ${pythonPackages.flake8}/bin/flake8 src/ tests/ \
            --max-line-length=120 \
            --extend-ignore=E203,W503,F401,F541,E501,E122 \
            --exclude=__pycache__,*.pyc,.git,build,dist \
            || echo "Note: Some flake8 warnings found. Consider reviewing them."

          echo "Formatting complete!"
        '';
      }
    );
}
