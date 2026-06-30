{ python3, writeShellApplication }:

let
  pythonEnv = python3.withPackages (ps: [
    ps.fugashi
    ps.unidic-lite
    ps.pykakasi
    ps.mpd2
  ]);
in
writeShellApplication {
  name = "ja-generate-aliases";
  runtimeInputs = [ pythonEnv ];
  text = ''
    exec python3 ${../../scripts/ja/generate.py} "$@"
  '';
}
