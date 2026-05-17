# GUI Refinement & AI Efficiency Benefits

## Overview
This update focuses on professional GUI/UX refinements to improve the "flow" of gameplay and introduces the first positive mechanical benefit for the **AI Dependence** index.

## Key UX Refinements

### 1. Context-Aware Tab Switching
- Interacting with the map (clicking a unit or tile) now automatically switches the sidebar to the **Selection** tab.
- This ensures that relevant data is always visible exactly when the player needs it, eliminating manual tab switching after map interactions.

### 2. Enhanced Strategic Map Visuals
- Replaced standard alphanumeric symbols with high-quality strategic glyphs to improve map readability:
    - **Units**: Used distinct shapes like `○` (Colony), `▲` (Infantry), `⚒` (Engineer), `■` (Garrison), and `⚡` (Shock).
    - **Bases**: Differentiated between player bases (`⛫`) and foreign bases (`⌬`).
    - **Aliens**: Added unique biological markers (`🪱`, `🦑`).
- These glyphs make the strategic situation (frontlines, colonies, hazards) clear at a glance, even when zoomed out.

## AI Dependence: Efficiency Bonuses
The "Synthetic Governance" path now offers tangible rewards to balance its high-risk crisis triggers:
- **Trigger**: AI Dependence > 20.
- **Benefit**: Provides a scaling **Efficiency Bonus** to all base yields (Nutrients, Minerals, Energy).
- **Scale**: Up to **+25% global yield bonus** at 100 AI Dependence.
- **Trade-off**: Players must decide whether to leverage this massive economic boost at the cost of increasing the risk of "Governance Override" and other high-dependence crises.

## Changes

### `smac_gui`
- **`main.rs`**: Updated `handle_tile_click` to set `active_tab = SidebarTab::Selection`.

### `smac_core`
- **`presentation.rs`**: Updated `unit_map_symbol` and `base_map_symbol` with new strategic glyphs.
- **`game_state.rs`**: Updated `effective_base_yields` to calculate and apply the scaling AI Dependence bonus.

## Verification
- **UX Flow**: Verified that clicking a base or unit instantly focuses the Selection tab.
- **Visuals**: Confirmed that map glyphs are distinct and readable.
- **Mechanics**: All yield calculation tests remain stable; the bonus scales correctly as intended.
