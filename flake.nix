{
  description = "ctag - A command line tool for managing tags on Confluence pages in bulk";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
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
          flake8
          black
          isort
          mypy
        ];

        # Python environment with all packages
        pythonEnv = pkgs.python3.withPackages (ps: runtimeDeps ++ devDeps);

        # The ctag package
        ctag = pythonPackages.buildPythonPackage rec {
          pname = "ctag";
          version = "0.1.0";
          format = "setuptools";

          src = builtins.path {
            path = ./.;
            name = "ctag-source";
          };

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
            pkgs.python3Packages.pip  # Keep pip available for edge cases
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
            pip install -e .
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

        # Formatter
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
