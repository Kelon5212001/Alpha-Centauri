use bevy::prelude::*;

pub struct FactionSelectPlugin;

impl Plugin for FactionSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_faction_select_screen);
    }
}

fn setup_faction_select_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    // DO NOT spawn a camera here!
    // commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle {
        text: Text::from_section(
            "Select Your Faction (Demo)",
            TextStyle {
                font: asset_server.load("FiraSans-Bold.ttf"),
                font_size: 36.0,
                color: Color::YELLOW,
            },
        ),
        ..Default::default()
    });
    // TODO: Add faction selection logic
}
