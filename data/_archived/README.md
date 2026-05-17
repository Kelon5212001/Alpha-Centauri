# Archived Data Files

These files were moved out of the active `data/` root on 2026-05-15 during `Operation Clean Slate`.

Reason:

- they are not part of the current `smac_core` runtime loading path
- they were creating ambiguity about which JSON files are actually authoritative

Archived files:

- `buildings.json`: empty placeholder file from an older content layout
- `societies.json`: empty placeholder file from an older content layout
- `unit_runtime.json`: older unit runtime stats table superseded by `units.json` plus `smac_core` runtime mappings

If one of these files needs to become active again, it should be reintroduced with:

1. an explicit caller in the active runtime/tooling path
2. validation coverage
3. top-level documentation updates
