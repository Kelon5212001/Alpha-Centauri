mod ui;
mod map_view;
mod camera;
mod audio;

use bevy::prelude::*;
use smac_core::GameState;

#[derive(Resource)]
pub struct GameStateResource(pub GameState);

#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected_tile: Option<(usize, usize)>,
    pub selected_unit: Option<usize>,
    pub selected_base: Option<usize>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SMAC Rust AI - Bevy Client".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GameStateResource(GameState::new_game(20, 20, 42)))
        .init_resource::<SelectionState>()
        .add_systems(Startup, setup)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(map_view::MapViewPlugin)
        .add_plugins(audio::AudioPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        camera::MainCamera,
    ));
}
