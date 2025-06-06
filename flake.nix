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

          # Ensure JSON schema files are included in the package
          postInstall = ''
            # Copy JSON schema files to the installed package
            cp -r $src/src/models/*.json $out/lib/python*/site-packages/src/models/
          '';

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
          buildInputs = with pkgs; [
            python3
            python3Packages.pip
            python3Packages.setuptools
            python3Packages.wheel
            python3Packages.virtualenv
          ];

          shellHook = ''
            # Create virtual environment if it doesn't exist
            if [ ! -d .venv ]; then
              python -m venv .venv
            fi

            # Activate virtual environment
            source .venv/bin/activate

            # Install dependencies if not already installed
            pip install -e .
            pip install -r requirements-dev.txt

            # Create .env file if it doesn't exist
            if [ ! -f .env ]; then
              cp .env.example .env
            fi
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
