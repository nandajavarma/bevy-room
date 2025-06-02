//! Animates a sprite in response to a keyboard event.
//!
//! See `sprite_sheet.rs` for an example where the sprite animation loops indefinitely.


use bevy::{input::common_conditions::{input_just_pressed}, prelude::*, time::common_conditions::on_timer};

mod common;
use common::*;

mod bullet;
use bullet::BulletPlugin;

mod player;
use player::PlayerPlugin;

mod crates;
use crates::CratePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .init_state::<AppState>()
        .init_resource::<GameState>()
        .add_plugins(BulletPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(CratePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_game_text)
        .add_systems(Update, game_over)
        .add_systems(Update, restart_game.run_if(input_just_pressed(KeyCode::Space)))
        .run();
}

fn restart_game(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    text_game_over: Query<Entity, With<TextGameOver>>,
) {
    if game_state.is_game_over {
        next_state.set(AppState::InGame);
        game_state.is_game_over = false;
        game_state.health = 100;
        game_state.gold = 0;
        PlayerPlugin::create_player(&mut commands, &asset_server, &mut texture_atlas_layouts);
        for entity in text_game_over.iter() {
            commands.entity(entity).despawn();
        }
    }
}


fn game_over(
    mut game_state: ResMut<GameState>,
    entities: Query<(&Transform, &Sprite, Entity)>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    windows: Query<&Window>,
) {
    if game_state.health <= 0 {
        game_state.is_game_over = true;
        // despawn all the entities when game over
        for (_transform, _sprite, entity) in entities.iter() {
            commands.entity(entity).despawn();
        }
        commands.spawn((
            Text::new("Game Over\nPress Space to restart"),
            TextGameOver,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(windows.single().unwrap().height() / 2.0),
                left: Val::Px(windows.single().unwrap().width() / 2.0),
                ..default()
            },
        ));


        next_state.set(AppState::Paused);
    }
}

fn update_game_text(
    game_state: Res<GameState>,
    mut text: Single<&mut Text, With<TextStat>>,
) {
    text.0 = format!("Gold: {}\nHealth: {}", game_state.gold, game_state.health);
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);
    PlayerPlugin::create_player(&mut commands, &asset_server, &mut texture_atlas_layouts);

    commands.spawn((
        Text::new("Gold: 0\nHealth: 100"),
        TextStat,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}