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

const WIDTH: usize = 10;
const HEIGHT: usize = 10;
const CELL_SIZE: f32 = 50.0;

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
                let velocity = Vec2::X;
                let density = 0.0;

                row.push(Cell { velocity, density })
            }
            grid.push(row);
        }

        Self(grid)
    }

    pub fn get_average_density(&self, x: usize, y: usize) -> f32 {
        let x_plus = (x + 1) % WIDTH;
        let x_minus = (x + WIDTH - 1) % WIDTH;
        let y_plus = (y + 1) % HEIGHT;
        let y_minus = (y + HEIGHT - 1) % HEIGHT;

        let n1 = self.0[y][x_minus].density;
        let n2 = self.0[y][x_plus].density;
        let n3 = self.0[y_minus][x].density;
        let n4 = self.0[y_plus][x].density;
        let avg = (n1 + n2 + n3 + n4) / 4.0;
        avg
    }

    pub fn get_average_velocity_x(&self, x: usize, y: usize) -> f32 {
        let x_plus = (x + 1) % WIDTH;
        let x_minus = (x + WIDTH - 1) % WIDTH;
        let y_plus = (y + 1) % HEIGHT;
        let y_minus = (y + HEIGHT - 1) % HEIGHT;

        let n1 = self.0[y][x_minus].velocity.x;
        let n2 = self.0[y][x_plus].velocity.x;
        let n3 = self.0[y_minus][x].velocity.x;
        let n4 = self.0[y_plus][x].velocity.x;
        let avg = (n1 + n2 + n3 + n4) / 4.0;
        avg
    }

    pub fn get_average_velocity_y(&self, x: usize, y: usize) -> f32 {
        let x_plus = (x + 1) % WIDTH;
        let x_minus = (x + WIDTH - 1) % WIDTH;
        let y_plus = (y + 1) % HEIGHT;
        let y_minus = (y + HEIGHT - 1) % HEIGHT;

        let n1 = self.0[y][x_minus].velocity.y;
        let n2 = self.0[y][x_plus].velocity.y;
        let n3 = self.0[y_minus][x].velocity.y;
        let n4 = self.0[y_plus][x].velocity.y;
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
    // grid.0[4][4].density = 20.0;
    grid.0[4][4].velocity.x = 20.0;
    grid.0[4][4].velocity.y = 20.0;
    commands.spawn().insert(grid);

    let half_cell = CELL_SIZE / 2.0;
    let half_x = WIDTH as f32 * half_cell - half_cell;
    let half_y = HEIGHT as f32 * half_cell - half_cell;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let v = 0.0;
            let cell_material = materials.add(Color::rgb(v, v, v).into());
            let arrow_material = materials.add(Color::hsl(0.0, 1.0, 0.5).into());

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

fn arrow(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
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
    arrow.set_attribute("Vertex_Color", v_color);

    let indices = vec![0, 1, 2, 3, 5, 4, 4, 5, 6];
    arrow.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    commands.spawn_bundle(MeshBundle {
        mesh: meshes.add(arrow),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]),
        transform: Transform::from_scale(Vec3::ONE * 10.0),
        ..Default::default()
    });
}

fn testing_system(time: Res<Time>, mut qg: Query<&mut Grid>) {
    if let Ok(mut grid) = qg.single_mut() {
        let dt = time.delta_seconds();
        let s = time.seconds_since_startup();
        for row in &mut grid.0 {
            for cell in row {
                cell.density = (cell.density + dt) % 1.0;

                let len = cell.velocity.length();
                // let angle = cell.velocity.angle_between(Vec2::X);
                // let angle_2 = angle + PI / 180.0 * dt;
                // println!("{} {}", angle, angle_2);
                // let angle = angle_2;
                let angle = s.sin() as f32;
                cell.velocity = Vec2::new(len * angle.cos(), len * angle.sin());
                // cell.velocity.y = (cell.velocity.y + 0.01) % 1.0;
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
                    let avg = new_grid.get_average_density(x, y);
                    new_grid.0[y][x].density = (grid.0[y][x].density + k * avg) / (1.0 + k);

                    let avg = new_grid.get_average_velocity_x(x, y);
                    new_grid.0[y][x].velocity.x = (grid.0[y][x].velocity.y + k * avg) / (1.0 + k);

                    let avg = new_grid.get_average_velocity_y(x, y);
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
    for _event in mouse_motion_events.iter() {
        // info!("{:?}", event.delta);
    }
    for _event in cursor_moved_events.iter() {
        // info!("{:?}", event);
    }
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
        .add_startup_system(arrow.system())
        .add_system(testing_system.system())
        // .add_system(diffusion_system.system())
        // .add_system(advection_system.system())
        // .add_system(density_square_system.system())
        // .add_system(velocity_arrow_direction_system.system())
        // .add_system(velocity_arrow_color_system.system())
        .add_system(print_mouse_events_system.system())
        .run();
}

// mod mesh;
// fn main() {
//     mesh::main();
// }
