{ lib, rustPlatform, qwerty_movement_keys ? false }:

rustPlatform.buildRustPackage {
  pname = "inori";
  version = "0.1.0";
  src = ../..;

  cargoLock.lockFile = ../../Cargo.lock;
  buildNoDefaultFeatures = qwerty_movement_keys;
  buildFeatures = lib.optional qwerty_movement_keys "qwerty_movement_keys";

  meta = with lib; {
    description = "inori client for the Music Player Daemon (MPD)";
    license = licenses.gpl3Only;
    platforms = platforms.all;
    maintainers = with maintainers; [ stephen-huan ];
  };
}
