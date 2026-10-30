use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

const WALK_SPEED: f32 = 40.0;
const RUN_SPEED: f32 = 80.0;
const TILE: f32 = 16.0;

// player_grid.png: 12x3 grid of 16x16 cells. Side/3-4 frames face right; flipped for left.
// Each frame is (index, mirror). Mirror swaps the stepping leg on front/back views.
type Frame = (usize, bool);

// Turnaround order: front, 3/4 front, side, 3/4 back, back
static NEUTRAL: [Frame; 5] = [(0, false), (1, false), (2, false), (3, false), (4, false)];
static STANCE: [Frame; 5] = [(12, false), (13, false), (14, false), (15, false), (16, false)];

// Strides return to the standing frame between steps
static WALK_DOWN: [Frame; 4] = [(5, false), (0, false), (5, true), (0, false)];
static WALK_UP: [Frame; 4] = [(6, false), (4, false), (6, true), (4, false)];
static WALK_SIDE: [Frame; 4] = [(7, false), (2, false), (8, false), (2, false)];
static RUN_DOWN: [Frame; 6] = [(17, false), (18, false), (12, false), (17, true), (18, true), (12, false)];
static RUN_UP: [Frame; 6] = [(19, false), (20, false), (16, false), (19, true), (20, true), (16, false)];
static RUN_SIDE: [Frame; 8] = [(24, false), (14, false), (25, false), (14, false), (26, false), (14, false), (27, false), (14, false)];
static RUN_DOWN_DIAG: [Frame; 8] = [(28, false), (13, false), (29, false), (13, false), (30, false), (13, false), (31, false), (13, false)];
static RUN_UP_DIAG: [Frame; 8] = [(32, false), (15, false), (33, false), (15, false), (34, false), (15, false), (35, false), (15, false)];

#[derive(Component)]
struct Player {
    facing: IVec2,
    windup: Timer, // time spent in stance before running
}

#[derive(Component)]
struct Animation {
    frames: &'static [Frame],
    current: usize,
    flip: bool, // facing left
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin { meta_check: AssetMetaCheck::Never, ..default() }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (movement, animate).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((Camera2d, Transform::from_scale(Vec3::splat(0.25)))); // 4x zoom

    let tiles = assets.load("tiles.png");
    let tile_layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(16), 2, 1, None, None));
    let (cols, rows) = (40, 24);
    for y in 0..rows {
        for x in 0..cols {
            commands.spawn((
                Sprite::from_atlas_image(tiles.clone(), TextureAtlas { layout: tile_layout.clone(), index: ((x / 4) % 2) as usize }),
                Transform::from_xyz((x - cols / 2) as f32 * TILE, (y - rows / 2) as f32 * TILE, 0.0),
            ));
        }
    }

    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(16), 12, 3, None, None));
    commands.spawn((
        Sprite::from_atlas_image(assets.load("player_grid.png"), TextureAtlas { layout, index: 0 }),
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player { facing: IVec2::NEG_Y, windup: Timer::from_seconds(0.15, TimerMode::Once) },
        Animation { frames: &NEUTRAL[0..1], current: 0, flip: false },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

// Index into the turnaround frames for a facing direction
fn turn(facing: IVec2) -> usize {
    match (facing.x != 0, facing.y) {
        (false, -1) => 0,
        (true, -1) => 1,
        (true, 0) => 2,
        (true, _) => 3,
        _ => 4,
    }
}

fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Transform, &mut Animation)>,
) {
    let mut dir = IVec2::ZERO;
    if keys.pressed(KeyCode::KeyW) { dir.y += 1; }
    if keys.pressed(KeyCode::KeyS) { dir.y -= 1; }
    if keys.pressed(KeyCode::KeyA) { dir.x -= 1; }
    if keys.pressed(KeyCode::KeyD) { dir.x += 1; }
    let sprinting = dir != IVec2::ZERO && keys.pressed(KeyCode::ShiftLeft);

    for (mut player, mut transform, mut anim) in &mut query {
        if dir != IVec2::ZERO { player.facing = dir; }
        if sprinting { player.windup.tick(time.delta()); } else { player.windup.reset(); }
        let running = player.windup.is_finished();

        let speed = if !sprinting { WALK_SPEED } else if running { RUN_SPEED } else { 0.0 };
        transform.translation += (dir.as_vec2().normalize_or_zero() * speed * time.delta_secs()).extend(0.0);

        let t = turn(player.facing);
        let frames: &'static [Frame] = if dir == IVec2::ZERO {
            &NEUTRAL[t..=t]
        } else if !sprinting {
            if dir.x != 0 { &WALK_SIDE } else if dir.y > 0 { &WALK_UP } else { &WALK_DOWN }
        } else if !running {
            &STANCE[t..=t]
        } else {
            match (dir.x != 0, dir.y) {
                (true, 0) => &RUN_SIDE,
                (true, -1) => &RUN_DOWN_DIAG,
                (true, _) => &RUN_UP_DIAG,
                (false, 1) => &RUN_UP,
                _ => &RUN_DOWN,
            }
        };

        anim.flip = player.facing.x < 0;
        if anim.frames != frames {
            anim.frames = frames;
            anim.current = 0;
        }
    }
}

fn animate(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Animation, &mut Sprite)>) {
    for (mut timer, mut anim, mut sprite) in &mut query {
        if timer.0.tick(time.delta()).just_finished() {
            anim.current = (anim.current + 1) % anim.frames.len();
        }
        let (index, mirror) = anim.frames[anim.current];
        sprite.flip_x = anim.flip != mirror;
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = index;
        }
    }
}
