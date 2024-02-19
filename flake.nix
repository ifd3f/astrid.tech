{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    seams.url = "github:ifd3f/seams";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, seams, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        site = seams.lib.makeSite {
          inherit pkgs;
          name = "astrid.tech";
          content = ./.;
        };
        withCname = pkgs.runCommand "astrid.tech" { } ''
          mkdir -p $out
          cp -ar ${site}/* ${site}/.* $out/
          echo 'astrid.tech' > $out/CNAME
        '';
      in {
        packages.default = withCname;
        devShells.default = with pkgs;
          mkShell {
            buildInputs =
              [ nodePackages.prettier seams.packages.${system}.seams ];
          };
      });
}
