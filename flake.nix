{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

      in {
        devShells = rec {
          default = run;

          run = pkgs.mkShell {
            buildInputs = with pkgs; [ 
              cargo
              clippy
              darwin.apple_sdk.frameworks.Security
              libiconv
              rust-analyzer
              rustc
              rustfmt 
              vips
            ];

            shellHook = ''
              just
            '';
          };
        };
      });
}
