{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:ipetkov/crane";
    };
  };


  outputs = { self, nixpkgs, crane }:
  let supportedSystems = [ "aarch64-linux" "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });
  in {
    packages = forAllSystems (system: let pkgs = nixpkgsFor.${system}; in {
      default = crane.lib.${system}.buildPackage {
        src = ./.;
        
        NIRI_CONFIG = pkgs.stdenvNoCC.mkDerivation {
          name = "niri_config";
          src = ./.;
  
          buildPhase = "cp -r $src/niri_config.kdl $out";
        } + "niri_config.kdl";
        STARDUST_RES_PREFIXES = pkgs.stdenvNoCC.mkDerivation {
          name = "resources";
          src = ./.;
  
          buildPhase = "cp -r $src/res $out";
        };
      };
    });

    devShells = forAllSystems (system: {
      default = crane.lib.${system}.devShell {
      };
    });
  };
}
