use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;
use rand::prelude::*;

use crate::common::*;

pub struct BulletPlugin;

const BULLET_SPAWN_PERIOD: u64 = 1000;
const BULLET_SPEED: f32 = 10.0;
const BULLET_SCALE: f32 = 4.0;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_bullets.run_if(on_timer(Duration::from_millis(BULLET_SPAWN_PERIOD))));
        app.add_systems(Update, move_bullets);
        app.add_systems(Update, bullet_damage);
    }
}

#[derive(Component)]
struct Bullet {
    speed: f32,
}

fn spawn_bullets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows:  Query<&mut Window>,
    current_state: Res<State<AppState>>,
) {
    if current_state.get().to_owned() != AppState::InGame {
        return;
    }

    let bullet_texture = asset_server.load("sensei.png");

    let mut rng = rand::rng();
    let window = windows.single().unwrap();

    commands.spawn((
        Sprite {
            image: bullet_texture.clone(),
        ..default()
        },
        Collider { collided: false },
        Bullet {
            speed: BULLET_SPEED,
        },
        Transform::from_scale(Vec3::splat(BULLET_SCALE)).with_translation(Vec3::new(-window.width() / 2.0, rng.random_range(-window.height() / 2.0..window.height() / 2.0), -1.0)),
    ));
}

fn move_bullets(
    mut query: Query<(&mut Bullet, &mut Transform)>,
    windows: Query<&Window>,
) {
    let window = windows.single().unwrap();
    for (bullet, mut transform) in &mut query {
        transform.translation.x += bullet.speed;
        if transform.translation.x > window.width() + 100.0 {
            transform.translation.x = -window.width() - 100.0;
        }
    }
}

fn bullet_damage(
    mut commands: Commands,
    mut bullet_query: Query<(&mut Bullet, &Collider, &Transform, Entity)>,
    mut game_state: ResMut<GameState>,
) {
    for (_bullet, collider, _transform, entity) in bullet_query.iter_mut() {
        if collider.collided {
            if game_state.health > 10 {
                game_state.health -= 10;
            } else {
                game_state.health = 0;
            }
            commands.entity(entity).despawn();
        }
    }
}