#!/usr/bin/env bash
set -euo pipefail

# GPT/Codex project backup script.
# Usage:
#   bash scripts/gpt-git-backup.sh "checkpoint: describe what changed"
#
# Optional:
#   SKIP_CHECKS=1 bash scripts/gpt-git-backup.sh "dirty checkpoint: describe why"

MESSAGE="${1:-checkpoint: GPT project backup}"
ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "$ROOT_DIR"

BRANCH="$(git branch --show-current)"
if [[ -z "$BRANCH" ]]; then
  echo "ERROR: Could not determine current branch."
  exit 1
fi

if ! git remote get-url origin >/dev/null 2>&1; then
  echo "ERROR: No git remote named 'origin' found."
  echo "Run: git remote -v"
  exit 1
fi

SAFE_BRANCH="$(echo "$BRANCH" | tr '/: ' '---')"
STAMP="$(date +'%Y%m%d-%H%M%S')"
TAG="backup-${SAFE_BRANCH}-${STAMP}"

echo "== GPT Git Backup =="
echo "Repo:   $ROOT_DIR"
echo "Branch: $BRANCH"
echo "Tag:    $TAG"
echo "Msg:    $MESSAGE"
echo

echo "== Current status =="
git status --short
echo

if [[ "${SKIP_CHECKS:-0}" != "1" ]]; then
  echo "== Running content validation =="
  cargo run -p smac_core --bin validate_content --quiet

  echo
  echo "== Running workspace tests =="
  cargo test --workspace --quiet
else
  echo "WARNING: SKIP_CHECKS=1 set. Skipping validation and tests."
fi

echo
echo "== Staging changes =="
git add -A

if git diff --cached --quiet; then
  echo "No staged changes to commit."
else
  echo "== Committing =="
  git commit -m "$MESSAGE"
fi

echo
echo "== Creating backup tag =="
git tag -a "$TAG" -m "$MESSAGE"

echo
echo "== Pushing branch and tag =="
git push -u origin "$BRANCH"
git push origin "$TAG"

echo
echo "Backup complete."
git log --oneline -1
echo "Created tag: $TAG"
