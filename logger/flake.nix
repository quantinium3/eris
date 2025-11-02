{
  description = "Eris - key logger";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@ { self
    , nixpkgs
    , systems
    , ...
    }:
    let
      inherit (nixpkgs) lib;
      eachSystem = lib.genAttrs (import systems);
      pkgsFor = eachSystem
        (system: import nixpkgs {
          localSystem.system = system;
        });
    in
    {
      packages = lib.mapAttrs
        (system: pkgs:
          let
            fs = lib.fileset;
            src = fs.difference (fs.gitTracked ./.) (fs.unions [
              ./flake.lock
              (fs.fileFilter (file: file.hasExt ".nix") ./.)
            ]);

          in
          {
            default = self.packages.${system}.eris-logger;
            eris-logger = pkgs.rustPlatform.buildRustPackage {
              name = "eris-logger";
              src = fs.toSource {
                root = ./.;
                fileset = src;
              };

              cargoLock = {
                lockFile = ./Cargo.lock;
                allowBuiltinFetchGit = true;
              };

              buildType = "release";
              doCheck = false;
              strictDeps = true;
            };
          })
        pkgsFor;
    };
}
