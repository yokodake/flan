{
  description = "flan, ";

  inputs.mozilla = { url = "github:mozilla/nixpkgs-mozilla"; flake = false; };
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs =
    { self
    , nixpkgs
    , mozilla
    , flake-utils
    , ...
    } :
    let

      rustOverlay = final: prev:
        let rustChannel = prev.rustChannelOf {
              channel = "nightly";
              sha256 = "sha256-N72kh3PZQe8K+NYywqO882ih0diEbO6DawWjS6yWShk=";
            };
        in
        { inherit rustChannel;
          rustc = rustChannel.rust;
          cargo = rustChannel.rust;
        };
    in flake-utils.lib.eachDefaultSystem
        (system:
          let pkgs = import nixpkgs {
            inherit system;
            overlays = [
              (import "${mozilla}/rust-overlay.nix")
              rustOverlay
            ];
          };
          in
          {
            devShell = pkgs.mkShell {
              name = "flan";
              buildInputs = with pkgs; [
                rustChannel.rust
              ];
            };
          }
        );
}
