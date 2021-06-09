use std::f32::consts::PI;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::CursorMoved;
use bevy::window::WindowResized;
use rand::Rng;

// https://youtu.be/qsYE1wMEMPA
// https://bevy-cheatbook.github.io/cookbook/clear-color.html
// ! Each call to angle_between should make sure the vector's length isn't zero

const WIDTH: usize = 10;
const HEIGHT: usize = 10;
const CELL_SIZE: f32 = 50.0;

// TODO Maybe separate into VelocityGrid and DensityGrid
struct Grid(Vec<Vec<Cell>>);
#[derive(Debug)]
struct DensitySquare;
#[derive(Debug)]
struct VelocityArrow;
struct Position {
    x: usize,
    y: usize,
}

#[derive(Debug)]
struct Cell {
    velocity: Vec2,
    density: f32,
}

fn create_grid() -> Grid {
    let mut rng = rand::thread_rng();
    let mut grid = Vec::with_capacity(HEIGHT);

    for _ in 0..HEIGHT {
        let mut row = Vec::with_capacity(WIDTH);
        for _ in 0..WIDTH {
            let vel_len = rng.gen_range(1.0..10.0);
            let vel_ang = rng.gen_range(0.0..2.0 * PI);
            let vel_x = vel_len * vel_ang.cos();
            let vel_y = vel_len * vel_ang.sin();
            let velocity = Vec2::new(vel_x, vel_y);

            let density = rng.gen_range(0.0..1.0);

            row.push(Cell { velocity, density })
        }
        grid.push(row);
    }

    Grid(grid)
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    // Camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Grid
    let grid = create_grid();
    commands.spawn().insert(grid);

    let mut rng = rand::thread_rng();

    let half_cell = CELL_SIZE / 2.0;
    let half_x = WIDTH as f32 * half_cell - half_cell;
    let half_y = HEIGHT as f32 * half_cell - half_cell;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let v = rng.gen_range(0.0..=1.0);
            let h = rng.gen_range(0.0..=180.0);
            let cell_material = materials.add(Color::rgb(v, v, v).into());
            let arrow_material = materials.add(Color::hsl(h, 1.0, 0.5).into());

            let transform_x = x as f32 * CELL_SIZE - half_x;
            let transform_y = y as f32 * CELL_SIZE - half_y;

            commands
                .spawn_bundle(SpriteBundle {
                    material: cell_material,
                    transform: Transform::from_xyz(transform_x, transform_y, 0.0),
                    sprite: Sprite::new(Vec2::new(CELL_SIZE, CELL_SIZE)),
                    ..Default::default()
                })
                .insert(DensitySquare)
                .insert(Position { x, y });

            commands
                .spawn_bundle(SpriteBundle {
                    material: arrow_material,
                    transform: Transform::from_xyz(transform_x, transform_y, 0.0),
                    sprite: Sprite::new(Vec2::new(CELL_SIZE / 2.0, 3.0)),
                    ..Default::default()
                })
                .insert(VelocityArrow)
                .insert(Position { x, y });
        }
    }
}

fn window_startup_system(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    let width = WIDTH as f32 * CELL_SIZE;
    let height = HEIGHT as f32 * CELL_SIZE;
    window.set_resolution(width, height);
    window.set_title("Fluid Simulation".to_string());
}

fn diffusion_system(time: Res<Time>, mut qg: Query<&mut Grid>) {
    if let Ok(mut grid) = qg.single_mut() {
        let dt = time.delta_seconds();
        for row in &mut grid.0 {
            for cell in row {
                cell.density = (cell.density + dt) % 1.0;

                // let len = cell.velocity.length();
                // let angle = cell.velocity.angle_between(Vec2::X);
                // let angle_2 = angle + PI / 180.0 * dt;
                // println!("{} {}", angle, angle_2);
                // let angle = angle_2;
                // cell.velocity = Vec2::new(len * angle.cos(), len * angle.sin());
                cell.velocity.y = (cell.velocity.y + 0.01) % 1.0;
            }
        }
    }
}

/// Display the grid density values as squares
fn density_square_system(
    qg: Query<&Grid>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&DensitySquare, &Position, &mut Handle<ColorMaterial>)>,
) {
    if let Ok(grid) = qg.single() {
        for (_density_square, position, color) in query.iter_mut() {
            let color_mat = materials.get_mut(&*color).unwrap();
            let Position { x, y } = position;
            let v = grid.0[*y][*x].density;
            color_mat.color = Color::rgb(v, v, v);
        }
    }
}

//// Display the velocity of each cell as colored arrows
fn velocity_arrow_direction_system(
    qg: Query<&Grid>,
    mut query: Query<(&VelocityArrow, &Position, &mut Transform)>,
) {
    if let Ok(grid) = qg.single() {
        for (_velocity_arrow, position, mut transform) in query.iter_mut() {
            // println!("{:?} {:?}", _velocity_arrow, transform);
            let rotation = &mut transform.rotation;

            let Position { x, y } = position;
            let vel: Vec2 = grid.0[*y][*x].velocity;
            let angle = vel.angle_between(Vec2::X);
            *rotation = Quat::from_rotation_z(angle);
            // println!("{:?} {:?}", vel, rotation);
        }
    }
}

fn velocity_arrow_color_system(
    qg: Query<&Grid>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&VelocityArrow, &Position, &mut Handle<ColorMaterial>)>,
) {
    if let Ok(grid) = qg.single() {
        for (_velocity_arrow, position, color) in query.iter_mut() {
            let color_mat = materials.get_mut(&*color).unwrap();
            let Position { x, y } = position;
            let len = grid.0[*y][*x].velocity.length();
            // Hue goes from 180 to 9
            let hue = 90.0 / len.clamp(0.5, 10.0);
            color_mat.color = Color::hsl(hue, 1.0, 0.5);
        }
    }
}

/// https://github.com/bevyengine/bevy/blob/main/crates/bevy_window/src/event.rs
///
/// This system prints out all mouse events as they come in
fn print_mouse_events_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut window_resized_events: EventReader<WindowResized>,
) {
    // TODO Apply an external force into the simulation grid
    // for event in mouse_motion_events.iter() {
    //     info!("{:?}", event.delta);
    // }
    // for event in cursor_moved_events.iter() {
    //     info!("{:?}", event);
    // }
    for event in window_resized_events.iter() {
        info!("{:?}", event);
    }
}

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_system(window_startup_system.system())
        .add_system(diffusion_system.system())
        // .add_system(density_square_system.system())
        .add_system(velocity_arrow_direction_system.system())
        .add_system(velocity_arrow_color_system.system())
        .add_system(print_mouse_events_system.system())
        .run();
}

// mod breakout;
// fn main() {
//     breakout::main();
// }
