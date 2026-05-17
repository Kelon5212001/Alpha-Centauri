#[cfg(feature = "audio")]
mod audio;
mod camera;
mod map_view;
mod ui;

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
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
    .add_plugins(map_view::MapViewPlugin);

    #[cfg(feature = "audio")]
    app.add_plugins(audio::AudioPlugin);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), camera::MainCamera));
}
