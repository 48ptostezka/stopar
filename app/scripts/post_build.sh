#!/bin/sh
# Trunk post_build hook, run after every build in the folder about to become dist/.
#
# 1. Copies the game's pictures (games/<game>/images/) into images/.
# 2. Writes precache-manifest.js: the list of every file the service worker
#    (public/sw.js) keeps for offline play, and a version derived from their
#    contents. Any change to any file gives a new version, so phones replace
#    their cached copy on the next visit.
set -eu

game="${STOPA_GAME:-galaxy}"
pictures="$TRUNK_SOURCE_DIR/../games/$game/images"

cd "$TRUNK_STAGING_DIR"

rm -rf images
if [ -d "$pictures" ]; then
  cp -R "$pictures" images
fi

files=$(find . -type f ! -name precache-manifest.js ! -name sw.js | sed 's|^\./||' | LC_ALL=C sort)
version=$(for f in $files; do cat "$f"; done | cksum | cut -d ' ' -f 1)

{
  printf '// Written by scripts/post_build.sh during the build. Do not edit.\n'
  printf 'self.PRECACHE_VERSION = "%s";\n' "$version"
  printf 'self.PRECACHE_URLS = [\n  "./",\n'
  for f in $files; do printf '  "%s",\n' "$f"; done
  printf '];\n'
} > precache-manifest.js
