{ lib, stdenvNoCC, fetchFromGitHub, tectonic, poppler_utils }:

stdenvNoCC.mkDerivation rec {
  name = "inori-logo";
  src = ./src;

  strictDeps = true;
  nativeBuildInputs = [ tectonic poppler_utils ];
  preferLocalBuild = true;

  cache = fetchFromGitHub {
    owner = "stephen-huan";
    repo = "tectonic-cache";
    rev = "acfff2b6f6fcfeaae2098b1069230c57cd56c92a";
    hash = "sha256-99Q2l48G0R8gUaE31BgZqeAKiBzOVfzh/PIAauZT5mo=";
  };

  buildPhase = ''
    runHook preBuild

    mkdir -p .cache
    export XDG_CACHE_HOME=$(realpath .cache)
    export TECTONIC_CACHE_DIR=${cache}
    export SOURCE_DATE_EPOCH=0
    substitute ${name}.tex ${name}-black.tex --subst-var-by fg 000000
    tectonic --only-cached -Z deterministic-mode ${name}-black.tex
    pdftocairo -svg ${name}-black.pdf
    substitute ${name}.tex ${name}-white.tex --subst-var-by fg ffffff
    tectonic --only-cached -Z deterministic-mode ${name}-white.tex
    pdftocairo -svg ${name}-white.pdf

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm644 ${name}-black.svg -T $out/${name}.svg
    install -Dm644 ${name}-white.svg -T $out/${name}-white.svg

    runHook postInstall
  '';

  meta = with lib; {
    description = "Logo for inori";
    license = licenses.unlicense;
    platforms = platforms.all;
    maintainers = with maintainers; [ stephen-huan ];
  };
}
