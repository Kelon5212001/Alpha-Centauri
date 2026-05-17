mod factions;
mod tech;
mod worldgen;
mod ui;

use bevy::prelude::*;
use ui::intro::IntroScreenPlugin;
use ui::faction_select::FactionSelectPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((IntroScreenPlugin, FactionSelectPlugin))
        .run();
}
