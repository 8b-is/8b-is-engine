#!/usr/bin/env bash
# auth.sh — the ONE-TIME hand-off: the headless Google auth for the
# Colab CLI. Run it once in a terminal; it prints a URL, you sign in
# with peter.lodri@gmail.com, paste the code back — the refresh token
# caches in ~/.colab-cli-oauth-config.json and every later drive is
# headless.
set -euo pipefail
echo "⟦ colab auth ⟧ sign in with peter.lodri@gmail.com — copy the URL, paste the code"
colab whoami
echo "⟦ the lane is open ⟧ now: ./tools/unsloth/drive.sh"
