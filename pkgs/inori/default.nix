{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "inori";
  version = "0.3.0";
  src = ../..;

  cargoLock.lockFile = ../../Cargo.lock;

  meta = with lib; {
    description = "Client for the Music Player Daemon (MPD)";
    homepage = "https://github.com/eshrh/inori";
    license = licenses.gpl3Only;
    platforms = platforms.unix ++ platforms.windows;
    mainProgram = "inori";
    maintainers = with maintainers; [ stephen-huan ];
  };
}
