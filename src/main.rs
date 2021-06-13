use std::f32::consts::PI;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::pipeline::PipelineDescriptor;
use bevy::render::pipeline::RenderPipeline;
use bevy::render::shader::ShaderStage;
use bevy::render::shader::ShaderStages;
use bevy::window::CursorMoved;
use bevy::window::WindowResized;

// https://youtu.be/qsYE1wMEMPA
// https://bevy-cheatbook.github.io/cookbook/clear-color.html
// ! Each call to angle_between should make sure the vector's length isn't zero

const WIDTH: usize = 20;
const HEIGHT: usize = 20;
const CELL_SIZE: f32 = 30.0;
// const WIDTH: usize = 50;
// const HEIGHT: usize = 50;
// const CELL_SIZE: f32 = 10.0;

const VERTEX_SHADER: &str = r"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Color;
layout(location = 1) out vec3 v_Color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    v_Color = Vertex_Color;
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
";

const FRAGMENT_SHADER: &str = r"
#version 450
layout(location = 1) in vec3 v_Color;
layout(location = 0) out vec4 o_Target;
void main() {
    o_Target = vec4(v_Color, 1.0);
}
";

// TODO Maybe separate into VelocityGrid and DensityGrid
// TODO make a double buffer
#[derive(Clone)]
struct Grid(Vec<Vec<Cell>>);
struct DensitySquare;
struct VelocityArrow;
#[derive(Debug)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Clone, Debug)]
struct Cell {
    velocity: Vec2,
    density: f32,
}

impl Grid {
    pub fn new() -> Self {
        let mut grid = Vec::with_capacity(HEIGHT);

        for _ in 0..HEIGHT {
            let mut row = Vec::with_capacity(WIDTH);
            for _ in 0..WIDTH {
                let velocity = Vec2::ZERO;
                let density = 0.0;

                row.push(Cell { velocity, density })
            }
            grid.push(row);
        }

        Self(grid)
    }

    pub fn get_average<F: Fn(&Cell) -> f32>(&self, x: usize, y: usize, attr: F) -> f32 {
        let x_plus = (x + 1) % WIDTH;
        let x_minus = (x + WIDTH - 1) % WIDTH;
        let y_plus = (y + 1) % HEIGHT;
        let y_minus = (y + HEIGHT - 1) % HEIGHT;

        let n1 = attr(&self.0[y][x_minus]);
        let n2 = attr(&self.0[y][x_plus]);
        let n3 = attr(&self.0[y_minus][x]);
        let n4 = attr(&self.0[y_plus][x]);
        let avg = (n1 + n2 + n3 + n4) / 4.0;
        avg
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    // Camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Grid
    let mut grid = Grid::new();
    grid.0[4][4].density = 20.0;
    grid.0[4][4].velocity.x = 20.0;
    grid.0[4][4].velocity.y = -20.0;
    commands.spawn().insert(grid);

    let half_cell = CELL_SIZE / 2.0;
    let half_x = WIDTH as f32 * half_cell - half_cell;
    let half_y = HEIGHT as f32 * half_cell - half_cell;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let v = 0.0;
            let cell_material = materials.add(Color::rgb(v, v, v).into());

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
        }
    }
}

pub fn arrows_setup(
    mut commands: Commands,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    // Arrow
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    let mut arrow = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);

    // Vertices of the arrow
    //    0
    //
    // 1 3 4 2
    //
    //
    //   5 6
    let v_pos = vec![
        [0.0, 16.0, 0.0],
        [-3.0, 10.0, 0.0],
        [3.0, 10.0, 0.0],
        [-1.0, 10.0, 0.0],
        [1.0, 10.0, 0.0],
        [-1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
    ];
    let v_color = vec![[1.0, 1.0, 0.0]; v_pos.len()];
    arrow.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    arrow.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    let indices = vec![0, 1, 2, 3, 5, 4, 4, 5, 6];
    arrow.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    // let mesh_handle = meshes.add(arrow);
    let render_pipelines =
        RenderPipelines::from_pipelines(vec![RenderPipeline::new(pipeline_handle)]);

    let half_cell = CELL_SIZE / 2.0;
    let half_x = WIDTH as f32 * half_cell - half_cell;
    let half_y = HEIGHT as f32 * half_cell - half_cell;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // let arrow_material = materials.add(Color::hsl(0.0, 1.0, 0.5).into());

            let transform_x = x as f32 * CELL_SIZE - half_x;
            let transform_y = y as f32 * CELL_SIZE - half_y;
            let translation = Vec3::new(transform_x, transform_y, 1.0);

            commands
                .spawn_bundle(MeshBundle {
                    mesh: meshes.add(arrow.clone()),
                    render_pipelines: render_pipelines.clone(),
                    transform: Transform {
                        translation,
                        scale: Vec3::ONE * CELL_SIZE / 15.0,
                        ..Default::default()
                    },
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

fn testing_system(time: Res<Time>, mut qg: Query<&mut Grid>) {
    if let Ok(mut grid) = qg.single_mut() {
        // let dt = time.delta_seconds();
        let s = time.seconds_since_startup() as f32;
        for row in &mut grid.0 {
            for cell in row {
                // cell.density = (cell.density + dt) % 1.0;
                cell.velocity = Vec2::new(s.cos(), s.sin()) * s;
            }
        }
    }
}

fn diffusion_system(time: Res<Time>, mut qg: Query<&mut Grid>) {
    if let Ok(mut grid) = qg.single_mut() {
        let mut new_grid = grid.clone();
        let k = 15.0 * time.delta_seconds();
        for _ in 0..5 {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    // d_n = (d_c + k*s_n) / (1 + k)
                    let avg = new_grid.get_average(x, y, |cell| cell.density);
                    new_grid.0[y][x].density = (grid.0[y][x].density + k * avg) / (1.0 + k);

                    let avg = new_grid.get_average(x, y, |cell| cell.velocity.x);
                    new_grid.0[y][x].velocity.x = (grid.0[y][x].velocity.y + k * avg) / (1.0 + k);

                    let avg = new_grid.get_average(x, y, |cell| cell.velocity.y);
                    new_grid.0[y][x].velocity.y = (grid.0[y][x].velocity.y + k * avg) / (1.0 + k);
                }
            }
        }
        *grid = new_grid;
    }
}

fn advection_system(time: Res<Time>, mut qg: Query<&mut Grid>) {
    if let Ok(mut grid) = qg.single_mut() {
        let mut new_grid = grid.clone();
        let dt = time.delta_seconds();
        for _ in 0..5 {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let pos = Vec2::new(x as f32, y as f32);
                    let f = pos - new_grid.0[y][x].velocity * dt;
                    let ix = f.x as usize;
                    let iy = f.y as usize;
                    let jx = f.x - ix as f32;
                    let jy = f.y - iy as f32;

                    let lerp = |a, b, k| a + k * (b - a);
                    let z1 = lerp(
                        new_grid.0[iy][ix].density,
                        new_grid.0[iy][(ix + 1) % WIDTH].density,
                        jx,
                    );
                    let z2 = lerp(
                        new_grid.0[(iy + 1) % HEIGHT][ix].density,
                        new_grid.0[iy][ix].density,
                        jx,
                    );

                    new_grid.0[y][x].density = lerp(z1, z2, jy);
                }
            }
        }
        *grid = new_grid;
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
            let rotation = &mut transform.rotation;

            let Position { x, y } = position;
            let vel: Vec2 = grid.0[*y][*x].velocity;

            let angle = vel.angle_between(Vec2::X);
            *rotation = Quat::from_rotation_z(angle + PI);
            // println!("{:?} {:?}", vel, rotation);
        }
        // println!("{:?}", grid.0[0][0].velocity);
    }
}

fn velocity_arrow_color_system(
    qg: Query<&Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&VelocityArrow, &Position, &mut Handle<Mesh>)>,
) {
    if let Ok(grid) = qg.single() {
        for (_velocity_arrow, position, mesh_handle) in query.iter_mut() {
            // println!("{:?} {:?}", position, mesh_handle);
            let Position { x, y } = position;
            let len = grid.0[*y][*x].velocity.length();
            // Hue goes from 180 to 9
            let len_max_value = 0.1;
            let hue = 180.0 - len.min(len_max_value) * 180.0 / len_max_value;

            let [r, g, b, _] = Color::hsl(hue, 1.0, 0.5).as_rgba_f32();
            let mesh = meshes.get_mut(&*mesh_handle).unwrap();
            mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, vec![[r, g, b]; 7]);
        }
    }
}

/// https://github.com/bevyengine/bevy/blob/main/crates/bevy_window/src/event.rs
///
/// This system prints out all mouse events as they come in
fn print_mouse_events_system(
    mut qg: Query<&mut Grid>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    // mut window_resized_events: EventReader<WindowResized>,
) {
    if let Ok(mut grid) = qg.single_mut() {
        for (mouse_event, cursor_event) in
            mouse_motion_events.iter().zip(cursor_moved_events.iter())
        {
            // info!("{:?} {:?}", mouse_event.delta, cursor_event.position);

            let x = (cursor_event.position.x / CELL_SIZE) as usize;
            let y = (cursor_event.position.y / CELL_SIZE) as usize;
            if x < WIDTH && y < HEIGHT {
                grid.0[y][x].velocity = 5.0 * mouse_event.delta;
            }
        }
    }
    // for event in cursor_moved_events.iter() {
    //     info!("{:?}", event);
    // }
    // for event in window_resized_events.iter() {
    //     info!("{:?}", event);
    // }
}

fn print_char_event_system(
    mut qg: Query<&mut Grid>,
    mut char_input_events: EventReader<ReceivedCharacter>,
) {
    for event in char_input_events.iter() {
        if event.char == 'r' {
            if let Ok(mut grid) = qg.single_mut() {
                *grid = Grid::new();
            }
        }
    }
}

fn main() {
    App::build()
        // .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_startup_system(window_startup_system.system())
        .add_startup_system(arrows_setup.system())
        // .add_system(testing_system.system())
        .add_system(diffusion_system.system())
        // .add_system(advection_system.system())
        .add_system(velocity_arrow_direction_system.system())
        .add_system(velocity_arrow_color_system.system())
        .add_system(density_square_system.system())
        .add_system(print_mouse_events_system.system())
        .add_system(print_char_event_system.system())
        .run();
}
