#!/bin/bash
for f in "$1"/*.svg; do
  inkscape "$f" --export-plain-svg --export-filename="$f" 2>/dev/null
done