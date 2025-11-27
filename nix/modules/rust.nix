{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem =
    { config
    , self'
    , pkgs
    , lib
    , ...
    }:
    {
      rust-project.crates."ctag".crane.args = {
        # No special buildInputs needed for this project
      };
      packages.default = self'.packages.ctag;
    };
}
