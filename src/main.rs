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

// 30 seconds timer for round
#[derive(Resource)]
struct RoundTimer {
    timer: Timer,
}

// player's score
#[derive(Resource)]
struct Score(i16);

// scoreboard label
#[derive(Component)]
struct Scoreboard;

// event for color handle ID of despawned cube
struct DespawnedCubeColorHandleID(bevy::asset::HandleId);

// app states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppState {
    Menu,
    DuringRound,
    AfterRound,
}

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
        // state for active round
        .add_state(AppState::DuringRound)
        // event for color handle id of despawned cube (to check for missing input)
        .add_event::<DespawnedCubeColorHandleID>()
        // spawn camera and add resources
        .add_startup_system(setup)
        // (re)set score to 0 and (re)start round timer
        .add_system_set(SystemSet::on_enter(AppState::DuringRound).with_system(round_setup))
        .add_system_set(
            SystemSet::on_update(AppState::DuringRound)
                // spawn cubes every 2 seconds
                .with_system(spawn_cube)
                // move cube in its direction with its speed in delta seconds
                .with_system(movement)
                // validate key input and add/deduct points
                .with_system(handle_key_input)
                // update scoreboard to current score
                .with_system(update_scoreboard)
                // despawn cube after despawn timer finished
                .with_system(despawn_cube)
                // reduce score for missing input
                .with_system(handle_missing_input)
                // end round if time is up
                .with_system(check_round_timer),
        )
        // clean up cubes after round ended
        .add_system_set(SystemSet::on_exit(AppState::DuringRound).with_system(cleanup::<Cube>))
        // set up state after round
        .add_system_set(SystemSet::on_enter(AppState::AfterRound).with_system(after_round_setup))
        // restart with left arrow or go to menu with right arrow
        .add_system_set(
            SystemSet::on_update(AppState::AfterRound).with_system(handle_after_round_key_input),
        )
        // clean up text UI after leaving after-round
        .add_system_set(SystemSet::on_exit(AppState::AfterRound).with_system(cleanup::<Text>))
        .run();
}

// spawn camera and add resources
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // timer for cube spawns with 2 seconds interval
    commands.insert_resource(SpawnTimer {
        timer: Timer::new(std::time::Duration::from_secs(2), TimerMode::Repeating),
    });
    // 30 seconds timer for round
    commands.insert_resource(RoundTimer {
        timer: Timer::new(std::time::Duration::from_secs(30), TimerMode::Once),
    });
    // player's score starting with 0
    commands.insert_resource(Score(0));
}

// reset score and timer at the beginning of round
fn round_setup(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut round_timer: ResMut<RoundTimer>,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
) {
    // get primary window or panic
    let Some(window) = windows.get_primary() else {
        panic!("No primary window");
    };

    // (Re)set score to 0
    score.0 = 0;

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

    // Restart round timer if finished
    if round_timer.timer.finished() {
        round_timer.timer.reset();
    }
}

// display message and instruction after round
fn after_round_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
    score: Res<Score>,
) {
    // get primary window or panic
    let Some(window) = windows.get_primary() else {
        panic!("No primary window");
    };

    // display instruction
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "Left arrow - Restart | Right arrow - Menu",
            TextStyle {
                font: asset_server.load("fonts/5by7.ttf"),
                font_size: window.width().max(window.height()) * 0.02,
                color: Color::WHITE,
            },
        )
        .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(0., window.height() * -0.3, 0.2),
        ..default()
    });

    // messages based on score
    const MESSAGES: [[&str; 3]; 3] = [
        // < 5
        [
            "Forgot to read the rules? :D",
            "Not really trying, are you? -_-",
            "Congrats for saving your energy!",
        ],
        // < 20
        [
            "Great! Let's try for greater!",
            "It's easy! But there's always more to achieve, right? :)",
            "You're good with these purple cubes, aren't you?",
        ],
        // > 20
        [
            "Wow! That's amazing!",
            "You're aiming for the sky!",
            "Stunning! Don't forget to take a break!",
        ],
    ];
    // chose random message based on score
    let msg = if score.0 < 5 {
        String::from(MESSAGES[0][thread_rng().gen_range(0..2)])
    } else if score.0 < 20 {
        String::from(MESSAGES[1][thread_rng().gen_range(0..2)])
    } else {
        String::from(MESSAGES[2][thread_rng().gen_range(0..2)])
    };

    // display message about score
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            msg,
            TextStyle {
                font: asset_server.load("fonts/5by7.ttf"),
                font_size: window.width().max(window.height()) * 0.05,
                color: Color::PURPLE,
            },
        )
        .with_alignment(TextAlignment::CENTER),
        text_2d_bounds: bevy::text::Text2dBounds {
            size: Vec2::new(window.width() * 0.9, window.height() / 2.),
        },
        transform: Transform::from_xyz(0., 0., 0.2),
        ..default()
    });
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
            .insert(Speed(thread_rng().gen_range(size.x * 2.0..size.x * 3.0)))
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

        // set spawn timer duration randomly between 1.0 and 1.5 seconds
        spawn_timer
            .timer
            .set_duration(std::time::Duration::from_secs_f32(
                thread_rng().gen_range(1.0..1.5),
            ));
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
    mut query: Query<(Entity, &mut DespawnTimer, &Handle<ColorMaterial>)>,
    mut event: EventWriter<DespawnedCubeColorHandleID>,
    time: Res<Time>,
) {
    for (entity, mut despawn_timer, color_handle) in query.iter_mut() {
        despawn_timer.timer.tick(time.delta());
        if despawn_timer.timer.finished() {
            // send color cube's color handle id to check for missing input
            event.send(DespawnedCubeColorHandleID(color_handle.id()));
            commands.entity(entity).despawn();
        }
    }
}

// deduct 5 points for missing input for purple cube
fn handle_missing_input(
    mut event: EventReader<DespawnedCubeColorHandleID>,
    mut score: ResMut<Score>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    for handle in event.iter() {
        if materials
            .get(&materials.get_handle(handle.0))
            .unwrap()
            .color
            == Color::PURPLE
        {
            score.0 -= 5;
        }
    }
}

// handle keys in after-round state: left for restart, right for menu
fn handle_after_round_key_input(keys: Res<Input<KeyCode>>, mut app_state: ResMut<State<AppState>>) {
    if keys.just_pressed(KeyCode::Left) {
        app_state.set(AppState::DuringRound).unwrap();
    } else if keys.just_pressed(KeyCode::Right) {
        app_state.set(AppState::Menu).unwrap();
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
    if score.is_changed() {
        for mut text in &mut query {
            text.sections[0].value = format!("Score: {}", score.0);
        }
    }
}

// update round timer and check if rounded ended
fn check_round_timer(
    mut timer: ResMut<RoundTimer>,
    time: Res<Time>,
    mut app_state: ResMut<State<AppState>>,
) {
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        // transition to after-round state
        app_state.set(AppState::AfterRound).unwrap();
    }
}

// cleanup all entities with component T
fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
