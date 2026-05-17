# AGENTS.md

## Project

This is the SMAC Rust AI project.

Active workspace:
- smac_core: deterministic game rules, AI, content, save/load, presentation state
- smac_gui: eframe/egui front end
- data: JSON game content
- documentation: project notes and technical docs

## Working rules

- Use terminal-safe commands.
- Prefer `cat > file <<'EOF'` for full file replacement.
- Always show the working directory before major file edits.
- Do not remove roles, docs, archives, or historical context unless explicitly instructed.
- Do not force push.
- Do not rewrite Git history unless explicitly instructed.
- Keep smac_core as the gameplay authority.
- Keep smac_gui as the view layer.

## Validation commands

Before a clean backup, run:

cargo run -p smac_core --bin validate_content --quiet
cargo test --workspace --quiet

## Git backup policy

At the end of each meaningful implementation block, sprint, graphics asset import, balance pass, or before a risky refactor, run:

bash scripts/gpt-git-backup.sh "checkpoint: short description of completed work"

If the user explicitly wants a backup even though tests are failing, run:

SKIP_CHECKS=1 bash scripts/gpt-git-backup.sh "dirty checkpoint: short description and known issue"

Expected behavior:
- stage all tracked/untracked non-ignored repo changes
- commit with the provided message
- create an annotated backup tag
- push the current branch
- push the backup tag

Never use git push --force for project backups.
