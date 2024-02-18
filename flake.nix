{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    seams.url = "github:ifd3f/seams";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, seams, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };
      in {
        packages.default = seams.lib.makeSite {
          inherit pkgs;
          name = "astrid.tech";
          content = ./.;
        };
        devShells.default = with pkgs;
          mkShell {
            buildInputs =
              [ nodePackages.prettier seams.packages.${system}.seams ];
          };
      });
}
