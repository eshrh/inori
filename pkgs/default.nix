{ pkgs }:

{
  inori = pkgs.callPackage ./inori { };
  inori-logo = pkgs.callPackage ./inori-logo { };
  ja-generate-aliases = pkgs.callPackage ./ja-generate-aliases { };
}
