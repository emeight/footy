use bevy::prelude::*;

const SPEED: f32 = 200.0;
const FRAMES: usize = 4;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct AnimationTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))    // crisp pixel art
        .add_systems(Startup, setup)
        .add_systems(Update, (movement, animate))
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(assets.load("background.png")));

    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(32), FRAMES as u32, 1, None, None));
    commands.spawn((
        Sprite::from_atlas_image(assets.load("player.png"), TextureAtlas { layout, index: 0 }),
        Transform::from_xyz(0.0, 0.0, 1.0), // z=1 draws above background
        Player,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) { dir.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) { dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyA) { dir.x += 1.0; }
    if keys.pressed(KeyCode::KeyD) { dir.x += 1.0; }

    for mut transform in &mut query {
        transform.translation += (dir.normalize_or_zero() * SPEED * time.delta_secs()).extend(0.0);
    }
}

fn animate(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut timer, mut sprite) in &mut query {
        if timer.0.tick(time.delta()).just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % FRAMES;
            }
        }
    }
}
