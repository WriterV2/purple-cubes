use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use rand::prelude::*;

// cube label
#[derive(Component)]
struct Cube;

// speed of cube
#[derive(Component)]
struct Speed(f32);

// direction of cube
#[derive(Component, PartialEq)]
struct Direction(Vec3);

// timer for cube spawns
#[derive(Resource)]
struct SpawnTimer {
    timer: Timer,
}

// timer to despawn cube
#[derive(Component)]
struct DespawnTimer {
    timer: Timer,
}

// player's score
#[derive(Resource)]
struct Score(i16);

// scoreboard label
#[derive(Component)]
struct Scoreboard;

fn main() {
    App::new()
        // default plugins
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                })
                // window settings
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Purple Cubes".to_string(),
                        ..default()
                    },
                    ..default()
                }),
        )
        // spawn camera and add resources
        .add_startup_system(setup)
        // spawn cubes every 2 seconds
        .add_system(spawn_cube)
        // move cube in its direction with its speed in delta seconds
        .add_system(movement)
        // validate key input
        .add_system(handle_key_input)
        // update scoreboard to current score
        .add_system(update_scoreboard)
        // despawn cube after despawn timer finished
        .add_system(despawn_cube)
        .run();
}

// spawn camera and add resources
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, windows: Res<Windows>) {
    // get primary window or panic
    let Some(window) = windows.get_primary() else {
        panic!("No primary window");
    };

    commands.spawn(Camera2dBundle::default());
    // timer for cube spawns with 2 seconds interval
    commands.insert_resource(SpawnTimer {
        timer: Timer::new(std::time::Duration::from_secs(2), TimerMode::Repeating),
    });
    // player's score starting with 0
    commands.insert_resource(Score(0));

    // scoreboard
    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "Score:",
                TextStyle {
                    font: asset_server.load("fonts/5by7.ttf"),
                    font_size: window.width().max(window.height()) * 0.03,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            transform: Transform::from_xyz(0., window.height() * 0.3, 0.2),
            ..default()
        })
        .insert(Scoreboard);
}

// spawn cubes every 2 seconds
fn spawn_cube(
    mut commands: Commands,
    windows: Res<Windows>,
    mut spawn_timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // update the timer
    spawn_timer.timer.tick(time.delta());

    // spawn cube if timer finished
    if spawn_timer.timer.finished() {
        // set color of cube randomly - 80% purple
        let color = if thread_rng().gen_bool(0.8) {
            Color::PURPLE
        } else {
            Color::MIDNIGHT_BLUE
        };

        // set direction of cube randomly
        let direction = match thread_rng().gen_range(0..4) {
            0 => Direction(Vec3::X),
            1 => Direction(Vec3::NEG_X),
            2 => Direction(Vec3::Y),
            3 => Direction(Vec3::NEG_Y),
            _ => unreachable!(),
        };

        // get primary window or panic
        let Some(window) = windows.get_primary() else {
            panic!("No primary window");
        };

        // set size of cube as 5% of the longest window side's length
        let size = Vec3::splat(window.width().max(window.height()) * 0.05);

        // spawn entitiy
        commands
            .spawn(Cube)
            .insert(direction)
            .insert(Speed(150.))
            .insert(DespawnTimer {
                timer: Timer::new(std::time::Duration::from_secs(1), TimerMode::Once),
            })
            .insert(MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                transform: Transform::default()
                    .with_translation(Vec3::ZERO)
                    .with_scale(size),
                material: materials.add(ColorMaterial::from(color)),
                ..default()
            });
    }
}

// move cube in its direction with its speed in delta seconds
fn movement(mut query: Query<(&mut Transform, &Speed, &Direction)>, time: Res<Time>) {
    for (mut transform, speed, direction) in query.iter_mut() {
        transform.translation += direction.0 * speed.0 * time.delta_seconds();
    }
}

// despawn cube after despawn timer finished
fn despawn_cube(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DespawnTimer, &Handle<ColorMaterial>)>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut score: ResMut<Score>,
) {
    for (entity, mut despawn_timer, color_handle) in query.iter_mut() {
        despawn_timer.timer.tick(time.delta());
        if despawn_timer.timer.finished() {
            // TODO: Refactor this to separate system
            // -5 score for missing input
            if materials.get(color_handle).unwrap().color == Color::PURPLE {
                score.0 -= 5;
            }
            commands.entity(entity).despawn();
        }
    }
}

// handle keys: validate input
fn handle_key_input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    query: Query<(Entity, &Direction, &Handle<ColorMaterial>)>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut score: ResMut<Score>,
) {
    // check for every active cube
    for (entity, direction, color_handle) in query.iter() {
        let mut validate_input = |input, dir| {
            if keys.just_pressed(input) {
                // valid if:
                // input matches direction of cube
                // color of cube is purple
                if direction.0 == dir && materials.get(color_handle).unwrap().color == Color::PURPLE
                {
                    // +1 score for correct input
                    score.0 += 1;
                } else {
                    // -5 score for incorrect input
                    score.0 -= 5;
                }
                // despawn cube after key input
                commands.entity(entity).despawn();
            }
        };

        validate_input(KeyCode::Up, Vec3::Y);
        validate_input(KeyCode::Down, Vec3::NEG_Y);
        validate_input(KeyCode::Right, Vec3::X);
        validate_input(KeyCode::Left, Vec3::NEG_X);
    }
}

// update scoreboard to current score
fn update_scoreboard(mut query: Query<&mut Text, With<Scoreboard>>, score: Res<Score>) {
    for mut text in &mut query {
        text.sections[0].value = format!("Score: {}", score.0);
    }
}
