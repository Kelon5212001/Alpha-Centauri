use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use smac_core::{Improvement, Terrain};

pub struct MapViewPlugin;

impl Plugin for MapViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapAssets>()
            .add_systems(Startup, load_assets)
            .add_systems(
                Update,
                (
                    sync_map_terrain,
                    sync_map_units,
                    sync_map_bases,
                    handle_map_interaction,
                ),
            );
    }
}

#[derive(Resource, Default)]
struct MapAssets {
    ocean: Handle<Image>,
    flat: Handle<Image>,
    rolling: Handle<Image>,
    rocky: Handle<Image>,
    fungus: Handle<Image>,
    crater: Handle<Image>,
    forest: Handle<Image>,
    thermal_borehole: Handle<Image>,
    solar_collector: Handle<Image>,
    condenser: Handle<Image>,
    road: Handle<Image>,
    mine: Handle<Image>,
    farm: Handle<Image>,
    base_player: Handle<Image>,
    base_ai: Handle<Image>,
    unit_player: Handle<Image>,
    unit_ai: Handle<Image>,
}

fn load_assets(mut assets: ResMut<MapAssets>, asset_server: Res<AssetServer>) {
    assets.ocean = asset_server.load("textures/terrain/ocean.png");
    assets.flat = asset_server.load("textures/terrain/flat.png");
    assets.rolling = asset_server.load("textures/terrain/rolling.png");
    assets.rocky = asset_server.load("textures/terrain/rocky.png");
    assets.fungus = asset_server.load("textures/terrain/fungus.png");
    assets.crater = asset_server.load("textures/terrain/crater.png");
    assets.forest = asset_server.load("textures/terrain/forest.png");
    assets.thermal_borehole = asset_server.load("textures/terrain/thermal_borehole.png");
    assets.solar_collector = asset_server.load("textures/terrain/solar_collector.png");
    assets.condenser = asset_server.load("textures/terrain/condenser.png");
    assets.road = asset_server.load("textures/terrain/road.png");
    assets.mine = asset_server.load("textures/terrain/mine.png");
    assets.farm = asset_server.load("textures/terrain/farm.png");
    assets.base_player = asset_server.load("textures/markers/base_player.png");
    assets.base_ai = asset_server.load("textures/markers/base_ai.png");
    assets.unit_player = asset_server.load("textures/markers/unit_player.png");
    assets.unit_ai = asset_server.load("textures/markers/unit_ai.png");
}

#[derive(Component)]
struct TileEntity {
    x: usize,
    y: usize,
}

#[derive(Component)]
struct ImprovementEntity;

#[derive(Component)]
struct UnitId(usize);

#[derive(Component)]
struct BaseId(usize);

#[derive(Component)]
struct PrevPos {
    x: usize,
    y: usize,
}

const HEX_RADIUS: f32 = 32.0;

fn sync_map_terrain(
    mut commands: Commands,
    game_state: Res<crate::GameStateResource>,
    assets: Res<MapAssets>,
    tile_query: Query<(Entity, &TileEntity)>,
    imp_query: Query<Entity, With<ImprovementEntity>>,
) {
    let state = &game_state.0;

    if tile_query.is_empty() {
        for y in 0..state.height {
            for x in 0..state.width {
                let cx = x as f32 * HEX_RADIUS * 1.5;
                let cy = y as f32 * HEX_RADIUS * 1.5;

                let texture = match state.tile(x, y).map(|t| t.terrain) {
                    Some(Terrain::Ocean) => assets.ocean.clone(),
                    Some(Terrain::Flat) => assets.flat.clone(),
                    Some(Terrain::Rolling) => assets.rolling.clone(),
                    Some(Terrain::Rocky) => assets.rocky.clone(),
                    Some(Terrain::Fungus) => assets.fungus.clone(),
                    Some(Terrain::Crater) => assets.crater.clone(),
                    _ => assets.ocean.clone(),
                };

                commands.spawn((
                    SpriteBundle {
                        texture,
                        transform: Transform::from_xyz(cx, cy, 0.0),
                        ..default()
                    },
                    TileEntity { x, y },
                ));
            }
        }
    }

    for entity in imp_query.iter() {
        commands.entity(entity).despawn();
    }
    for y in 0..state.height {
        for x in 0..state.width {
            let cx = x as f32 * HEX_RADIUS * 1.5;
            let cy = y as f32 * HEX_RADIUS * 1.5;
            if let Some(imp) = state.tile(x, y).and_then(|t| t.improvement) {
                let texture = match imp {
                    Improvement::Forest => assets.forest.clone(),
                    Improvement::ThermalBorehole => assets.thermal_borehole.clone(),
                    Improvement::Solar => assets.solar_collector.clone(),
                    Improvement::Condenser => assets.condenser.clone(),
                    Improvement::Road => assets.road.clone(),
                    Improvement::Mine => assets.mine.clone(),
                    Improvement::Farm => assets.farm.clone(),
                    _ => continue,
                };
                commands.spawn((
                    SpriteBundle {
                        texture,
                        transform: Transform::from_xyz(cx, cy, 0.1),
                        ..default()
                    },
                    ImprovementEntity,
                ));
            }
        }
    }
}

fn sync_map_units(
    mut commands: Commands,
    game_state: Res<crate::GameStateResource>,
    assets: Res<MapAssets>,
    mut unit_query: Query<(Entity, &UnitId, &mut Transform, &mut PrevPos)>,
) {
    let state = &game_state.0;
    let mut processed_ids = std::collections::HashSet::new();

    for (entity, unit_id, mut transform, mut prev_pos) in unit_query.iter_mut() {
        if let Some(unit) = state.units.iter().find(|u| u.id == unit_id.0 && u.alive) {
            let cx = unit.x as f32 * HEX_RADIUS * 1.5;
            let cy = unit.y as f32 * HEX_RADIUS * 1.5;

            if unit.x != prev_pos.x || unit.y != prev_pos.y {
                let dx = unit.x as f32 - prev_pos.x as f32;
                let dy = unit.y as f32 - prev_pos.y as f32;
                let angle = dy.atan2(dx);
                transform.rotation = Quat::from_rotation_z(angle);

                prev_pos.x = unit.x;
                prev_pos.y = unit.y;
            }

            transform.translation.x = cx;
            transform.translation.y = cy;
            processed_ids.insert(unit.id);
        } else {
            commands.entity(entity).despawn();
        }
    }

    for unit in &state.units {
        if !unit.alive || processed_ids.contains(&unit.id) {
            continue;
        }
        let cx = unit.x as f32 * HEX_RADIUS * 1.5;
        let cy = unit.y as f32 * HEX_RADIUS * 1.5;
        let texture = if unit.owner == state.player_owner() {
            assets.unit_player.clone()
        } else {
            assets.unit_ai.clone()
        };
        commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(cx, cy, 2.0),
                ..default()
            },
            UnitId(unit.id),
            PrevPos {
                x: unit.x,
                y: unit.y,
            },
        ));
    }
}

fn sync_map_bases(
    mut commands: Commands,
    game_state: Res<crate::GameStateResource>,
    assets: Res<MapAssets>,
    mut base_query: Query<(Entity, &BaseId, &mut Transform)>,
) {
    let state = &game_state.0;
    let mut processed_ids = std::collections::HashSet::new();

    for (entity, base_id, mut transform) in base_query.iter_mut() {
        if let Some(base) = state.bases.iter().find(|b| b.id == base_id.0) {
            let cx = base.x as f32 * HEX_RADIUS * 1.5;
            let cy = base.y as f32 * HEX_RADIUS * 1.5;
            transform.translation.x = cx;
            transform.translation.y = cy;
            processed_ids.insert(base.id);
        } else {
            commands.entity(entity).despawn();
        }
    }

    for base in &state.bases {
        if processed_ids.contains(&base.id) {
            continue;
        }
        let cx = base.x as f32 * HEX_RADIUS * 1.5;
        let cy = base.y as f32 * HEX_RADIUS * 1.5;
        let texture = if base.owner == state.player_owner() {
            assets.base_player.clone()
        } else {
            assets.base_ai.clone()
        };
        commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(cx, cy, 1.0),
                ..default()
            },
            BaseId(base.id),
        ));
    }
}

fn handle_map_interaction(
    game_state: Res<crate::GameStateResource>,
    mut selection: ResMut<crate::SelectionState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<crate::camera::MainCamera>>,
    q_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut contexts: EguiContexts,
) {
    let Ok((camera, camera_transform)) = q_camera.get_single() else {
        return;
    };
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() || ctx.is_using_pointer() {
        return;
    }

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let x = (world_pos.x / (HEX_RADIUS * 1.5)).round() as isize;
            let y = (world_pos.y / (HEX_RADIUS * 1.5)).round() as isize;

            let state = &game_state.0;
            if x >= 0 && x < state.width as isize && y >= 0 && y < state.height as isize {
                let tx = x as usize;
                let ty = y as usize;

                if mouse_input.just_pressed(MouseButton::Left) {
                    selection.selected_tile = Some((tx, ty));
                    let tile = state.tile(tx, ty);
                    selection.selected_base = tile.and_then(|t| t.base);
                    selection.selected_unit = tile.and_then(|t| t.unit);
                }

                if let Some(tile) = state.tile(tx, ty) {
                    egui::show_tooltip_at_pointer(ctx, egui::Id::new("map_tooltip"), |ui| {
                        ui.heading(format!("Tile: ({}, {})", tx, ty));
                        ui.label(format!("Terrain: {:?}", tile.terrain));
                        if let Some(imp) = tile.improvement {
                            ui.label(format!("Improvement: {:?}", imp));
                        }

                        let yields = state.tile_total_yields(tx, ty);
                        ui.label(format!(
                            "Yields: N:{} M:{} E:{}",
                            yields.nutrients, yields.minerals, yields.energy
                        ));

                        if let Some(base_id) = tile.base {
                            if let Some(base) = state.base(base_id) {
                                ui.separator();
                                ui.label(format!("Base: {}", base.name));
                                ui.label(format!("Population: {}", base.population));
                            }
                        }

                        if let Some(unit_id) = tile.unit {
                            if let Some(unit) = state.unit(unit_id) {
                                ui.separator();
                                ui.label(format!("Unit ID: {}", unit.id));
                                ui.label(format!("HP: {}", unit.hp));
                            }
                        }
                    });
                }
            }
        }
    }
}
