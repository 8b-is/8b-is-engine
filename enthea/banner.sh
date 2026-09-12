#!/bin/sh
# banner.sh — pure-ASCII, no-color, wavelike constellation banner.
# A sine ripple walks the family name down 4 rows, frame by frame, in plain
# POSIX sh. No ANSI, no colors, no deps. Drop it anywhere; pipe to less/tail.
set -eu

TEXT="${ENTHEA_BANNER_TEXT:-8b-is   alex,chris,nate,family and p}"
FRAMES="${ENTHEA_BANNER_FRAMES:-48}"
DELAY="${ENTHEA_BANNER_DELAY:-0.05}"
ROWS=4

# Sine table (32 entries, amplitude 0..3): the wave shape, hardcoded so pure
# sh needs no awk/bc/dc.
SINE="0 1 1 2 2 2 1 1 0 0 0 0 0 1 1 2 2 2 2 1 1 0 0 0 0 0 1 1 2 2 2 1"
set -- $SINE

len=${#TEXT}
frame=0
while [ "$frame" -lt "$FRAMES" ]; do
  row=0
  while [ "$row" -lt "$ROWS" ]; do
    line=""
    i=0
    while [ "$i" -lt "$len" ]; do
      idx=$(((frame + i) % 32))
      # shellcheck disable=SC2004
      off=$((idx + 1))
      eval "off=\${$off}"
      if [ "$off" -eq "$row" ]; then
        line="${line}$(printf '%s' "$TEXT" | cut -c$((i + 1)))"
      else
        line="${line} "
      fi
      i=$((i + 1))
    done
    printf '%s\n' "$line"
    row=$((row + 1))
  done
  printf '%s\n' ""   # blank separator between frames -> the wave scrolls
  frame=$((frame + 1))
  sleep "$DELAY"
done

# the constellation, pure ascii — a whisper, not a shout
printf '%s\n' "   vaked.dev"

# the whale of the sky and the dolphins of the ocean
printf '%s\n' \
  "      .-~~~-.                 ,-\"-,." \
  "  .- ~ ~-(       )_ _        /_/| .\\\\" \
  " /                     ~ -. /~ /| \\\\\\\\" \
  "|                          \\' | / | ~/" \
  " \\                         .'  \`'  Y'" \
  "   ~- ._ ,. ,.,.,., .__..-~"

printf '%s\n' "DON'T PANIC — and know where your towel is."
