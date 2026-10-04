#!/usr/bin/env bash
# Block until every GitHub Actions run for HEAD finishes; exit status is 0 only
# if all of them succeeded. Polls the GitHub API every 5 seconds (outbound HTTPS
# only). Requires an authenticated `gh`.
set -euo pipefail

sha="$(git rev-parse HEAD)"
ids=""
for _ in $(seq 1 75); do
  ids="$(gh run list --commit "$sha" --json databaseId --jq '.[].databaseId')"
  [ -n "$ids" ] && break
  sleep 2
done
[ -n "$ids" ] || { echo "no workflow run found for $sha" >&2; exit 1; }

# Let sibling workflows (e.g. a tag-triggered release run) register.
sleep 5
ids="$(gh run list --commit "$sha" --json databaseId --jq '.[].databaseId')"

status=0
for id in $ids; do
  gh run watch "$id" --exit-status --interval 5 >/dev/null || status=1
  gh run view "$id" --json workflowName,url,jobs \
    --jq '"\(.workflowName): \(.url)", (.jobs[] | "  \(.name): \(.conclusion)")'
done
exit "$status"
