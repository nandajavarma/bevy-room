use bevy::prelude::*;

use crate::common::*;
use std::time::Duration;
use rand::prelude::*;
use crate::on_timer;

const CRATE_SCALE: f32 = 4.0;

const CRATE_SPAWN_PERIOD: u64 = 1000;

pub struct CratePlugin;

impl Plugin for CratePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, loot);
        app.add_systems(Update, spawn_crates.run_if(on_timer(Duration::from_millis(CRATE_SPAWN_PERIOD))));
    }
}


fn spawn_crates(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_state: Res<State<AppState>>,
    windows: Query<&Window>,
) {
    if current_state.get().to_owned() != AppState::InGame {
        return;
    }
    let crate_texture = asset_server.load("crate.png");

    let mut rng = rand::rng();

    commands.spawn((
        Sprite {
            image: crate_texture.clone(),
        ..default()
        },
        Collider { collided: false },
        Crate { gold: 100 },
        Transform::from_scale(Vec3::splat(CRATE_SCALE)).with_translation(Vec3::new(rng.random_range(-windows.single().unwrap().width() / 2.0..windows.single().unwrap().width() / 2.0), rng.random_range(-windows.single().unwrap().height() / 2.0..windows.single().unwrap().height() / 2.0), -1.0)),
    ));
}


fn loot(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut crate_query: Query<(&mut Collider, &Crate, Entity)>,
) {
    for (collider, crate_component, entity) in crate_query.iter_mut() {
        if collider.collided {
            game_state.gold += crate_component.gold;
            commands.entity(entity).despawn();

        }
    }
}