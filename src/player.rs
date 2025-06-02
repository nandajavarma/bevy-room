use bevy::prelude::*;

use crate::common::*;

const SCALE: f32 = 6.0;
const SPEED: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, execute_animations);
        app.add_systems(Update, move_character::<CharacterSprite>);
        app.add_systems(Update, collision);
    }
}

fn move_character<S: Component>(
    mut character: Single<&mut Transform, With<S>>,
    mut movement_state: Single<&mut MovementState, With<S>>,
    windows: Query<&Window>,
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
    let window = windows.single().unwrap();
    let margin = 20.0;
    character.translation.x = character.translation.x.clamp(-window.width() / 2.0 + margin, window.width() / 2.0 - margin);
    character.translation.y = character.translation.y.clamp(-window.height() / 2.0 + margin, window.height() / 2.0 - margin);
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

impl PlayerPlugin {
    pub fn create_player(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) {
        let texture = asset_server.load("gabe-idle-run.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        let animation_config = AnimationConfig::new(0, 6, 12);

        commands.spawn((
            Sprite::from_atlas_image(texture, TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_config.first_sprite_index,
            }),
            Transform::from_scale(Vec3::splat(SCALE)),
            animation_config,
            CharacterSprite,
            MovementState { is_moving: false },
        ));
    }
}