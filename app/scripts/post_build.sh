#!/bin/sh
# Trunk post_build hook, run after every build in the folder about to become dist/.
#
# 1. Copies the game's pictures (games/<game>/images/) into images/.
# 2. Writes precache-manifest.js: the list of every file the service worker
#    (public/sw.js) keeps for offline play, and a version derived from their
#    names and contents. Any change to any file gives a new version, so
#    phones replace their cached copy on the next visit.
#
# File names may contain anything (spaces, accents, #, brackets): the list
# is read line by line, never split on spaces, and every name is written as
# a URL with unsafe characters percent-encoded. Hidden files such as
# .DS_Store are left out.
set -eu

game="${STOPA_GAME:-galaxy}"
pictures="$TRUNK_SOURCE_DIR/../games/$game/images"

cd "$TRUNK_STAGING_DIR"

rm -rf images
if [ -d "$pictures" ]; then
  cp -R "$pictures" images
fi

# One file per line, sorted the same way on every computer.
list=$(find . -type f ! -name precache-manifest.js ! -name sw.js ! -name '.*' \
  | sed 's|^\./||' | LC_ALL=C sort)

version=$(printf '%s\n' "$list" | while IFS= read -r f; do
  printf '%s\n' "$f"
  cat "$f"
done | cksum | cut -d ' ' -f 1)

# Percent-encodes every byte outside A-Z a-z 0-9 . _ ~ / - (so "IMG 1.png"
# becomes "IMG%201.png" and "jeskyně.png" becomes "jeskyn%C4%9B.png").
urls=$(printf '%s\n' "$list" | LC_ALL=C awk '
  BEGIN { for (i = 1; i < 256; i++) code[sprintf("%c", i)] = i }
  {
    url = ""
    for (i = 1; i <= length($0); i++) {
      c = substr($0, i, 1)
      if (c ~ /[A-Za-z0-9._~\/-]/) url = url c
      else url = url sprintf("%%%02X", code[c])
    }
    printf "  \"%s\",\n", url
  }')

{
  printf '// Written by scripts/post_build.sh during the build. Do not edit.\n'
  printf 'self.PRECACHE_VERSION = "%s";\n' "$version"
  printf 'self.PRECACHE_URLS = [\n  "./",\n'
  printf '%s\n' "$urls"
  printf '];\n'
} > precache-manifest.js
