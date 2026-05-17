use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_controls);
    }
}

#[derive(Component)]
pub struct MainCamera;

fn camera_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut projection)) = query.get_single_mut() else {
        return;
    };

    let mut pan = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        pan.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        pan.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        pan.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        pan.x += 1.0;
    }

    if pan.length_squared() > 0.0 {
        pan = pan.normalize();
    }

    // Adjust pan speed based on current zoom level
    let pan_speed = 500.0 * projection.scale;
    transform.translation += pan.extend(0.0) * pan_speed * time.delta_seconds();

    let mut zoom = 0.0;
    for event in mouse_wheel_events.read() {
        match event.unit {
            MouseScrollUnit::Line => {
                zoom -= event.y;
            }
            MouseScrollUnit::Pixel => {
                zoom -= event.y * 0.01;
            }
        }
    }

    if zoom != 0.0 {
        let zoom_speed = 0.1;
        projection.scale *= 1.0 + (zoom * zoom_speed);
        // Clamp scale to prevent extreme zooming
        projection.scale = projection.scale.clamp(0.1, 10.0);
    }
}
