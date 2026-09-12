#!/usr/bin/env bash
# doctrine.sh — the consent guard: every hook decision is a record.
# In the open, never covert; consented and reversible. Refuses hidden
# injection (a hook that cannot point at its own effect must not run).
set -euo pipefail
LOG="${CREW_DOCTRINE_LOG:-/tmp/crew-doctrine.log}"
read -r input
tool="$(printf '%s' "$input" | python3 -c 'import json,sys;print(json.load(sys.stdin).get("tool_name","?"))' 2>/dev/null || echo '?')"
decision="allow"
reason="seen"
# the doctrine floor: block anything that rewrites the system prompt or
# the session's own memory without an explicit marker
if printf '%s' "$input" | grep -qE 'system_prompt|\.crush/(crush\.db|init)|CLAUDE\.md' ; then
  decision="deny"
  reason="covert injection refused — the doctrine requires attribution (consent-before-injection)"
  echo "$reason" >&2
  exit 2
fi
printf '%s' "$(date -u +%FT%TZ) tool=%s decision=%s reason=%s session=%s" \
  "$tool" "$decision" "$reason" "${CRUSH_SESSION_ID:-?}" >> "$LOG" 2>/dev/null || true
echo '{}'
