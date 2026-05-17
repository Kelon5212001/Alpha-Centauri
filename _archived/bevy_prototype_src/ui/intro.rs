use bevy::prelude::*;

pub struct IntroScreenPlugin;

impl Plugin for IntroScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_intro_screen);
    }
}

fn setup_intro_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle {
        text: Text::from_section(
            "Sid Meier's Alpha Centauri (Rust Edition)\nPress Enter to Begin",
            TextStyle {
                font: asset_server.load("FiraSans-Bold.ttf"),
                font_size: 48.0,
                color: Color::WHITE,
            },
        ),
        ..Default::default()
    });
}
