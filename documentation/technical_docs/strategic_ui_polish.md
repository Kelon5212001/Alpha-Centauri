# Strategic UI Polish: Header & Map Badges

## Overview
This update introduces significant UI "polish" that enhances real-time feedback and reduces the need for constant tab-switching. The interface now surfaces critical strategic information directly in the main view.

## 1. Global Strategic Header
- **Function**: The Top Bar now displays real-time faction statistics for the player.
- **Surface Data**:
    - **Energy**: Current reserves.
    - **Research**: Progress of the current project (e.g., `12/40`).
    - **Strategic Indices**: Live values for **Food Security**, **Planet Toxicity**, and **AI Dependence**.
- **Impact**: Players can now monitor their global economy and environmental risks without leaving the map or switching to the Factions tab.

## 2. Map Status Badges
- **Function**: The tactical map now displays small status markers (badges) on tiles to signal immediate issues.
- **Badges**:
    - `!` (Red): **Unrest**. Indicates a base is experiencing social instability.
    - `?` (Yellow): **Queue Alert**. Indicates a base is idle or has an empty production queue despite having governor recommendations.
    - `*` (Orange): **Damage Alert**. Indicates a unit has lost HP and may need recovery.
- **Impact**: Players can identify troubled bases and damaged units at a glance across the entire map, facilitating faster crisis management.

## 3. UI Flow Refinement
- **Context-Aware Selection**: Clicking any tile or unit on the map now **automatically switches** the sidebar to the **Selection** tab.
- **Impact**: This significantly speeds up the gameplay loop of "Click Map -> Issue Command -> Click Next", as the relevant action panel is always visible upon interaction.

## Technical Changes

### `smac_core`
- **`MapTileDisplayState`**: Added `status_glyph` field to carry tactical markers.
- **`GameState::map_tile_display_state`**: Implemented logic to detect unrest, empty queues, and unit damage to populate status glyphs.

### `smac_gui`
- **`draw_top_bar`**: Updated to render faction statistics from the player owner.
- **`draw_map`**: Implemented badge rendering using the `painter().text()` API, positioned at the top-right of each tile button.
- **`handle_tile_click`**: Added automatic tab focusing.

## Verification
- **Header**: Confirmed stats update correctly after ending turns.
- **Badges**: Verified that adding a unit to a queue or healing a unit correctly removes its badge.
- **Selection**: Confirmed clicking map jumps to Selection tab.
- **Stability**: All 130+ tests passed.
