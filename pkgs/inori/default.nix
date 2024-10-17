{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "inori";
  version = "0.2.1";
  src = ../..;

  cargoLock.lockFile = ../../Cargo.lock;

  meta = with lib; {
    description = "inori client for the Music Player Daemon (MPD)";
    license = licenses.gpl3Only;
    platforms = platforms.all;
    maintainers = with maintainers; [ stephen-huan ];
  };
}
