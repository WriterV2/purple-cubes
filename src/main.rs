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
#[derive(Component)]
struct Direction(Vec3);

// timer for cube spawns
#[derive(Component)]
struct SpawnTimer {
    timer: Timer,
}

fn main() {
    App::new()
        // window settings
        .insert_resource(WindowDescriptor {
            title: "Purple Cubes".to_string(),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        // spawn camera and add resources
        .add_startup_system(setup)
        // spawn cubes every 2 seconds
        .add_system(spawn_cube)
        // move cube in its direction with its speed in delta seconds
        .add_system(movement)
        .run();
}

// spawn camera and add resources
fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    // timer for cube spawns with 2 seconds interval
    commands.insert_resource(SpawnTimer {
        timer: Timer::new(std::time::Duration::from_secs(2), true),
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
        // set direction of cube randomly
        let direction = match thread_rng().gen_range(0..4) {
            0 => Direction(Vec3::X),
            1 => Direction(Vec3::NEG_X),
            2 => Direction(Vec3::Y),
            3 => Direction(Vec3::NEG_Y),
            _ => unreachable!(),
        };

        // get primary window or panic
        let window = if let Some(win) = windows.get_primary() {
            win
        } else {
            panic!("No primary window")
        };

        // set size of cube as 5% of the longest window side's length
        let size = Vec3::splat(window.width().max(window.height()) * 0.05);

        // spawn entitiy
        commands
            .spawn()
            .insert(Cube)
            .insert(direction)
            .insert(Speed(20.))
            .insert_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                transform: Transform::default()
                    .with_translation(Vec3::ZERO)
                    .with_scale(size),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
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
