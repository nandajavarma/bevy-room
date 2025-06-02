use bevy::prelude::*;

/// Game states for controlling the overall application flow
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States, Reflect)]
pub enum AppState {
    #[default]
    InGame,
    Paused,
}

/// Global game state resource containing player stats and game status
#[derive(Resource)]
pub struct GameState {
    pub gold: u32,
    pub health: u32,
    pub is_game_over: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            gold: 0,
            health: 100,
            is_game_over: false,
        }
    }
}

/// Component for entities that can collide with other entities
#[derive(Component)]
pub struct Collider {
    pub collided: bool,
}

/// Marker component for the main character sprite
#[derive(Component)]
pub struct CharacterSprite;

/// Component for crate entities that provide loot when collected
#[derive(Component)]
pub struct Crate {
    pub gold: u32,
}

/// Component to track the movement state of entities
#[derive(Component)]
pub struct MovementState {
    pub is_moving: bool,
}

/// Marker component for game statistics text
#[derive(Component)]
pub struct TextStat;

/// Marker component for game over text
#[derive(Component)]
pub struct TextGameOver;

/// Configuration for sprite animations
#[derive(Component)]
pub struct AnimationConfig {
    pub first_sprite_index: usize,
    pub last_sprite_index: usize,
    pub fps: u8,
    pub frame_timer: Timer,
}

impl AnimationConfig {
    pub fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    pub fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(std::time::Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
} 