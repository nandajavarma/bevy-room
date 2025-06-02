//! Animates a sprite in response to a keyboard event.
//!
//! See `sprite_sheet.rs` for an example where the sprite animation loops indefinitely.

use std::time::Duration;

use bevy::{input::common_conditions::{input_just_pressed}, prelude::*, time::common_conditions::on_timer};
use rand::prelude::*;

const SCALE: f32 = 6.0;
const CRATE_SCALE: f32 = 4.0;
const SPEED: f32 = 10.0;

const CRATE_SPAWN_PERIOD: u64 = 10000;
const BULLET_SPAWN_PERIOD: u64 = 1000;
const BULLET_SPEED: f32 = 10.0;


#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States, Reflect)]
enum AppState {
  #[default]
  InGame,
  Paused,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, execute_animations)
        .add_systems(Update, move_character::<CharacterSprite>)
        .add_systems(Update, collision)
        .add_systems(Update, update_game_text)
        .add_systems(Update, loot)
        .add_systems(Update, spawn_crates.run_if(on_timer(Duration::from_millis(CRATE_SPAWN_PERIOD))))
        .add_systems(Update, spawn_bullets.run_if(on_timer(Duration::from_millis(BULLET_SPAWN_PERIOD))))
        .add_systems(Update, move_bullets)
        .add_systems(Update, bullet_logic)
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
        create_gabe(&mut commands, &asset_server, &mut texture_atlas_layouts);
        for entity in text_game_over.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn move_character<S: Component>(
    mut character: Single<&mut Transform, With<S>>,
    mut movement_state: Single<&mut MovementState, With<S>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut translation = Vec3::new(0.0, 0.0, 0.0);
    if input.pressed(KeyCode::KeyW) {
        translation += Vec3::new(0.0, 1.0, 0.0);
    }
    if input.pressed(KeyCode::KeyS) {
        translation += Vec3::new(0.0, -1.0, 0.0);
    }
    if input.pressed(KeyCode::KeyA) {
        translation += Vec3::new(-1.0, 0.0, 0.0);
    }
    if input.pressed(KeyCode::KeyD) {
        translation += Vec3::new(1.0, 0.0, 0.0);
    }
    
    let is_currently_moving = translation != Vec3::new(0.0, 0.0, 0.0);
    movement_state.is_moving = is_currently_moving;
    character.translation += translation * SPEED;
    if translation.x < 0.0 {
        character.scale.x = -SCALE;
    } else {
        character.scale.x = SCALE;
    }
}

fn collision(
    mut player_query: Query<(&CharacterSprite, &Transform, &Sprite)>,
    mut collider_query: Query<(&Transform, &mut Collider, Entity)>,
) {
    if let Ok((_player, player_transform, _player_sprite)) = player_query.single_mut() {
        let player_size = 100.0;

        for (collider_transform, mut collider, _collider_entity) in collider_query.iter_mut() {
            if (collider_transform.translation - player_transform.translation).length() < player_size  {
                if !collider.collided {
                    collider.collided = true;
                }
            }
        }
    }
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

fn bullet_logic(
    mut commands: Commands,
    mut bullet_query: Query<(&mut Bullet, &Collider, &mut Transform, Entity)>,
    mut game_state: ResMut<GameState>,
) {
    for (bullet, collider, mut transform, entity) in bullet_query.iter_mut() {
        transform.translation.x += bullet.speed;
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

#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
}

impl AnimationConfig {
    fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

// This system loops through all the sprites in the `TextureAtlas`, from  `first_sprite_index` to
// `last_sprite_index` (both defined in `AnimationConfig`).
fn execute_animations(
    time: Res<Time>, 
    mut query: Query<(&mut AnimationConfig, &mut Sprite, &MovementState)>,
) {
    for (mut config, mut sprite, movement_state) in &mut query {
        // Only animate if the character is moving
        if !movement_state.is_moving {
            continue;
        }
        
        // We track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index == config.last_sprite_index {
                    // ...and it IS the last frame, then we move back to the first frame and continue.
                    atlas.index = config.first_sprite_index;
                } else {
                    // ...and it is NOT the last frame, then we move to the next frame...
                    atlas.index += 1;
                }
                // Reset the frame timer to start counting all over again
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
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
    let window = windows.single().unwrap(); // Get window


    commands.spawn((
        Sprite {
            image: bullet_texture.clone(),
        ..default()
        },
        Collider { collided: false },
        Bullet {
            speed: BULLET_SPEED,
        },
        Transform::from_scale(Vec3::splat(CRATE_SCALE)).with_translation(Vec3::new(-window.width() / 2.0, rng.random_range(-window.height() / 2.0..window.height() / 2.0), -1.0)),
    ));
}

fn move_bullets(
    mut query: Query<(&mut Bullet, &mut Transform)>,
) {
    for (bullet, mut transform) in &mut query {
        transform.translation.x += bullet.speed;
    }
}

#[derive(Component)]
struct Bullet {
    speed: f32,
}

#[derive(Component)]
struct CharacterSprite;

#[derive(Component)]
struct Crate {
    gold: u32,
}

#[derive(Component)]
struct Collider {
    collided: bool,
}

#[derive(Resource)]
struct GameState {
    gold: u32,
    health: u32,
    is_game_over: bool,
}

#[derive(Component)]
struct TextStat;

#[derive(Component)]
struct TextGameOver;

// Add this new component to track movement state
#[derive(Component, Default)]
struct MovementState {
    is_moving: bool,
}

fn create_gabe(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load the sprite sheet using the `AssetServer`
    let gabe_texture = asset_server.load("gabe-idle-run.png");

    // The sprite sheet has 7 sprites arranged in a row, and they are all 24px x 24px
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // The first (left-hand) sprite runs at 10 FPS
    let animation_config_1 = AnimationConfig::new(1, 6, 10);

    // Create the first (left-hand) sprite
    commands.spawn((
        Sprite {
            image: gabe_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config_1.first_sprite_index,
            }),
            ..default()
        },
        Transform::from_scale(Vec3::splat(SCALE)).with_translation(Vec3::new(-70.0, 0.0, 0.0)),
        CharacterSprite,
        animation_config_1,
        MovementState::default(),
    ));


}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    commands.insert_resource(GameState { gold: 0, health: 100, is_game_over: false });

    create_gabe(&mut commands, &asset_server, &mut texture_atlas_layouts);

        // Create a minimal UI explaining how to interact with the example
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