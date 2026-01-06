{
  description = "Dev env for kindle-dashboard";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ fenix.overlays.default ];
      };
      nativeBuildInputs = [
        (fenix.packages.${system}.stable.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustfmt"
        ])
      ];
      buildInputs = with pkgs; [ rust-analyzer-nightly python3 ];
    in
    {
      # `eachDefaultSystem` transforms the input, our output set
      # now simply has `packages.default` which gets turned into
      # `packages.${system}.default` (for each system)
      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "kindle-weather-dashboard";
        version = "0.1.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };

      devShells.default = pkgs.mkShell {
        inherit buildInputs nativeBuildInputs;
      };
    }
  );
}
