#!/usr/bin/env python3
"""Generate inori aliases.json entries from an MPD library.

Requires:
  - python-mpd2
  - fugashi
  - unidic-lite (or unidic)
  - pykakasi
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import unicodedata
from collections.abc import Mapping
from pathlib import Path
from typing import Any, Literal, TypeGuard

from fugashi import Tagger
from mpd import MPDClient
from pykakasi import kakasi

JP_RE = re.compile(r"[\u3040-\u30ff\u3400-\u4dbf\u4e00-\u9fff]")
SPACE_RE = re.compile(r"\s+")
NON_ASCII_RE = re.compile(r"[^a-z0-9 ]+")

PathAliases = dict[str, str]
AlbumAliases = dict[str, str]
RomanizationStyle = Literal["hepburn", "kunrei", "passport"]


def contains_japanese(text: str) -> bool:
    return JP_RE.search(text) is not None


def normalize_ascii(text: str) -> str:
    text = unicodedata.normalize("NFKC", text).lower()
    text = NON_ASCII_RE.sub(" ", text)
    return SPACE_RE.sub(" ", text).strip()


def token_reading(token: Any) -> str:
    feat = getattr(token, "feature", None)
    for attr in ("kana", "pron", "pronBase", "lemma"):
        val = getattr(feat, attr, None) if feat is not None else None
        if isinstance(val, str) and val != "*":
            return val
    return token.surface


def romanize_japanese(
    text: str, tagger: Tagger, kks: Any, style: RomanizationStyle
) -> str:
    readings = [token_reading(tok) for tok in tagger(text)]
    src = " ".join(readings) if readings else text
    converted = " ".join(item[style] for item in kks.convert(src))
    return normalize_ascii(converted)


def is_obj_mapping(value: object) -> TypeGuard[Mapping[str, object]]:
    return isinstance(value, Mapping) and all(
        isinstance(k, str) for k in value.keys()
    )


def first_str(value: object) -> str | None:
    if isinstance(value, str):
        return value
    if isinstance(value, list) and value and isinstance(value[0], str):
        return value[0]
    return None


def parse_existing_entries(path: Path) -> tuple[PathAliases, AlbumAliases]:
    if not path.exists():
        return {}, {}
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, list):
        raise ValueError(f"{path} must be a JSON array")
    out_path: PathAliases = {}
    out_album: AlbumAliases = {}
    for i, entry in enumerate(raw):
        if not is_obj_mapping(entry):
            raise ValueError(f"{path} entry {i} must be an object")
        alias = first_str(entry.get("alias"))
        if not isinstance(alias, str):
            raise ValueError(f"{path} entry {i} must include string key 'alias'")
        p = first_str(entry.get("path"))
        a = first_str(entry.get("album"))
        if p is not None and a is None:
            out_path[p] = alias
        elif a is not None and p is None:
            out_album[a] = alias
        else:
            raise ValueError(
                f"{path} entry {i} must include exactly one of 'path' or 'album'"
            )
    return out_path, out_album


def get_tag(song: Mapping[str, object], key: str) -> str | None:
    candidates = (key, key.lower(), key.upper(), key.title())
    for cand in candidates:
        val = first_str(song.get(cand))
        if val is not None:
            return val
    return None


def merge_aliases(
    old_path: PathAliases,
    old_album: AlbumAliases,
    new_path: PathAliases,
    new_album: AlbumAliases,
    overwrite: bool,
) -> tuple[PathAliases, AlbumAliases]:
    if overwrite:
        merged_path = dict(old_path)
        merged_album = dict(old_album)
        merged_path.update(new_path)
        merged_album.update(new_album)
    else:
        merged_path = dict(new_path)
        merged_album = dict(new_album)
        merged_path.update(old_path)
        merged_album.update(old_album)
    return merged_path, merged_album


def serialize_entries(
    path_aliases: PathAliases, album_aliases: AlbumAliases
) -> list[dict[str, str]]:
    entries: list[dict[str, str]] = []
    for p in sorted(path_aliases):
        entries.append({"path": p, "alias": path_aliases[p]})
    for a in sorted(album_aliases):
        entries.append({"album": a, "alias": album_aliases[a]})
    return entries


def default_alias_file() -> Path:
    xdg = os.environ.get("XDG_CONFIG_HOME")
    if xdg:
        return Path(xdg) / "inori" / "aliases.json"
    return Path.home() / ".config" / "inori" / "aliases.json"


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description="Generate inori aliases.json from MPD metadata."
    )
    p.add_argument("--host", default=os.environ.get("MPD_HOST", "localhost"))
    p.add_argument("--port", type=int, default=int(os.environ.get("MPD_PORT", "6600")))
    p.add_argument("--password", default=os.environ.get("MPD_PASSWORD"))
    p.add_argument("--output", type=Path, default=default_alias_file())
    p.add_argument(
        "--style",
        choices=("hepburn", "kunrei", "passport"),
        default="kunrei",
        help="Romanization style (default: kunrei).",
    )
    p.add_argument(
        "--overwrite",
        action="store_true",
        help="Prefer newly generated aliases over existing aliases on conflicts.",
    )
    return p


def main() -> int:
    args = build_parser().parse_args()
    style: RomanizationStyle = args.style

    tagger = Tagger()
    kks = kakasi()

    client = MPDClient()
    client.timeout = 20
    client.idletimeout = None
    client.connect(args.host, args.port)
    if args.password:
        client.password(args.password)
    songs = client.listallinfo()
    client.close()
    client.disconnect()

    new_path: PathAliases = {}
    new_album: AlbumAliases = {}
    for song in songs:
        if not is_obj_mapping(song):
            continue
        path = get_tag(song, "file")
        if not path:
            continue
        title = get_tag(song, "Title")
        album = get_tag(song, "Album")

        if title and contains_japanese(title):
            alias = romanize_japanese(title, tagger, kks, style)
            if alias:
                new_path[path] = alias

        if album and contains_japanese(album):
            alias = romanize_japanese(album, tagger, kks, style)
            if alias and album not in new_album:
                new_album[album] = alias

    old_path, old_album = parse_existing_entries(args.output)
    merged_path, merged_album = merge_aliases(
        old_path, old_album, new_path, new_album, args.overwrite
    )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    entries = serialize_entries(merged_path, merged_album)
    args.output.write_text(
        json.dumps(entries, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )

    print(
        "generated:"
        f" titles={len(new_path)} albums={len(new_album)} |"
        f" merged totals: paths={len(merged_path)} albums={len(merged_album)}"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        raise SystemExit(1)
