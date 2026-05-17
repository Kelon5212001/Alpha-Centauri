# Visual Transition Plan

## Current State
The project currently uses `smac_gui` powered by `eframe/egui`. This provides a highly functional, immediate-mode GUI for interacting with the deterministic core (`smac_core`). The map is rendered as a grid of colored `egui::Button`s with text overlays, and the interface relies heavily on side panels and standard immediate-mode widgets.

While effective for the "Playability Proving Phase" (which successfully stabilized the 100-turn simulation), this UI paradigm is not suitable for a polished, commercial-quality strategy game presentation.

## Goal
Transition the visual presentation from the immediate-mode reference UI to a rich, asset-driven presentation layer, while maintaining the strict decoupling between the presentation layer and `smac_core`.

## Phase 1: Engine Evaluation
Before integrating sprites, the rendering backend must be evaluated:
1.  **Extend `egui`**: Utilize `egui_extras::image` or `egui::TextureHandle` to replace colored buttons with PNG/WebP sprites. This is the fastest path but may struggle with high-resolution zooming, panning, and particle effects.
2.  **Migrate to `Macroquad` / `ggez`**: Lightweight 2D frameworks that offer better sprite batching and shader support while retaining Rust's simplicity.
3.  **Resurrect `Bevy`**: The archived `bevy_prototype_src` could be revived. Bevy offers a robust ECS, excellent rendering performance, and a clear path to 3D if desired. Given that `smac_core` is now a stable, headless state machine, it can easily be slotted into Bevy as a Resource or decoupled thread.

*Recommendation*: Resurrect the Bevy prototype as `smac_bevy`. The headless stability of `smac_core` means Bevy only needs to act as a pure view/controller layer, polling `smac_core` for state and rendering the appropriate Sprites and UI components.

## Phase 2: Asset Pipeline
1.  **Tilesets**: Create or source a seamless hex or isometric tileset for terrain (Flat, Rolling, Rocky) and moisture levels.
2.  **Unit Sprites**: Design distinct sprites or 3D models for the primary unit chassis (Infantry, Speeder, Hovertank, Sea, Air).
3.  **Facility & Improvement Indicators**: Create visual overlays or distinct models for Farms, Mines, Solar Collectors, Forests, and Thermal Boreholes.
4.  **UI Chrome**: Transition the functional `egui` panels into styled UI layouts with thematic borders, sci-fi fonts, and immersive sound effects.

## Phase 3: Implementation Strategy
1.  Create a new workspace member (e.g., `smac_client` or `smac_bevy`).
2.  Import `smac_core` as a dependency.
3.  Implement a basic game loop that calls `state.apply_action()` based on player input.
4.  Implement a rendering loop that reads `state.tiles` and `state.units` to draw the map.
5.  Gradually port the functional panels (Base Management, Workshop, Diplomacy) from `smac_gui` into the new engine's UI system.
6.  Once feature parity is reached, archive `smac_gui` and make the new client the primary entry point.
