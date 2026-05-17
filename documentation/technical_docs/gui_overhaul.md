# GUI Overhaul: Modular Sidebar & Streamlined Controls

## Overview
The `smac_gui` has undergone a major visual and functional overhaul to improve usability and "visual polish". The interface now utilizes a tabbed sidebar to manage the increasing complexity of game information, and the top bar has been decluttered to focus on core gameplay actions.

## Key Changes

### 1. Tabbed Sidebar
The sidebar is now split into five logical tabs, accessible via buttons or keyboard shortcuts (`1-5`):
- **Selection (`1`)**: Shows detailed information about the currently selected unit or tile, and includes the Minimap for navigation.
- **Factions (`2`)**: Contains the global Research panel, Faction Overviews, and the newly added Strategic Indices (Food Security, AI Dependence, etc.).
- **Logistics (`3`)**: A dedicated view for the Command Console and high-level logistics/convoy state.
- **Saves (`4`)**: Centralizes all save/load functionality, including slot selection, file renaming, and the Save Browser.
- **Logs (`5`)**: Displays the full event history in a large scrollable area.

### 2. Streamlined Top Bar
- Removed all save/load inputs (file ID, display name, category) from the top bar.
- Removed the "Save Manual/Autosave" and "Load" buttons from the top bar.
- These controls are now located exclusively in the **Saves** tab of the sidebar.
- The top bar is now focused on: **Mission Year**, **Next Unit**, **End Turn**, **New Game**, **Map Overlay**, and **Zoom**.

### 3. Expert UX: Keyboard Shortcuts
To improve the flow of turn-based gameplay, several essential shortcuts have been implemented:
- **`Space`**: Select next unit.
- **`Enter`**: End turn (applies automations and wraps up the current year).
- **`1`, `2`, `3`, `4`, `5`**: Rapidly switch between sidebar tabs.

## Technical Details
- Added `SidebarTab` enum and `active_tab` state to `SmacApp`.
- Refactored `draw_side_panel` into a modular `match` statement.
- Implemented `handle_input` for global key events.
- Cleaned up redundant UI rendering logic in the main loop.

## Verification
- `cargo check -p smac_gui` passed.
- Tab switching and control movement verified as functional.
- Keyboard shortcuts implemented and tested.
