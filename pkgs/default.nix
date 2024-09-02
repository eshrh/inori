{ pkgs }:

{
  inori = pkgs.callPackage ./inori { };
  inori-logo = pkgs.callPackage ./inori-logo { };
}
