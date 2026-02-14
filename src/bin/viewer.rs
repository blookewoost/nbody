use bevy::prelude::*;
use threebody_sim::TrajectoryData;
use std::env;

/// Calculate the centroid and maximum distance of bodies at frame 0
fn calculate_camera_target(trajectory: &TrajectoryData) -> (Vec3, f32) {
    let mut centroid = Vec3::ZERO;
    let mut count = 0;

    // Calculate centroid of all bodies at frame 0
    for body_traj in &trajectory.bodies {
        if let Some(pos) = body_traj.get_position(0) {
            centroid += Vec3::new(pos.x, pos.y, pos.z);
            count += 1;
        }
    }

    if count > 0 {
        centroid /= count as f32;
    }

    // Calculate maximum distance from centroid
    let mut max_distance: f32 = 0.0;
    for body_traj in &trajectory.bodies {
        if let Some(pos) = body_traj.get_position(0) {
            let pos_vec = Vec3::new(pos.x, pos.y, pos.z);
            let distance = (pos_vec - centroid).length();
            max_distance = max_distance.max(distance);
        }
    }

    // Ensure minimum distance for very close systems
    if max_distance < 1e9 {
        max_distance = 1e9;
    }

    (centroid, max_distance)
}

/// Main viewer state
#[derive(Resource)]
struct ViewerState {
    trajectory: TrajectoryData,
    current_frame: usize,
    is_playing: bool,
    speed: f32, // Frames per update
}

/// Component for bodies in the 3D view
#[derive(Component)]
struct BodyVisual {
    body_index: usize,
}

/// Component to track and render the trail of a body
#[derive(Component)]
struct BodyTrail {
    body_index: usize,
    positions: Vec<Vec3>,
    max_trail_length: usize,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let trajectory_file = if args.len() > 1 {
        args[1].clone()
    } else {
        String::from("./data/results.csv")
    };

    println!("Loading trajectory from: {}", trajectory_file);
    
    let trajectory = match TrajectoryData::load_csv(&trajectory_file) {
        Ok(traj) => {
            println!("Loaded {} bodies with {} frames", traj.bodies.len(), traj.num_frames);
            traj
        }
        Err(e) => {
            eprintln!("Failed to load trajectory: {}", e);
            std::process::exit(1);
        }
    };

    let viewer_state = ViewerState {
        trajectory,
        current_frame: 0,
        is_playing: true,
        speed: 1.0,
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Three-Body Simulator Viewer".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(viewer_state)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_positions,
            update_trails,
            render_trails,
            handle_input,
            update_ui,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<ViewerState>,
) {
    // Calculate camera position based on initial body positions
    let (centroid, max_distance) = calculate_camera_target(&state.trajectory);
    let camera_distance = max_distance * 2.5; // Position camera at 2.5x the max distance
    let camera_pos = Vec3::new(
        centroid.x + camera_distance * 0.5,
        centroid.y + camera_distance * 0.8,
        centroid.z + camera_distance * 0.5,
    );

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(camera_pos.x, camera_pos.y, camera_pos.z)
            .looking_at(centroid, Vec3::Y),
        ..default()
    });

    // Add ambient light for overall illumination
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
    });

    // Primary directional light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 8000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            std::f32::consts::PI / 4.0,
            std::f32::consts::PI / 4.0,
        )),
        ..default()
    });

    // Secondary directional light for better side illumination
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            -std::f32::consts::PI / 4.0,
            std::f32::consts::PI / 4.0,
        )),
        ..default()
    });

    // Create sphere mesh for bodies
    let sphere_mesh = meshes.add(Sphere::new(5e9).mesh().ico(5).unwrap());

    // Create bodies with different colors
    let colors = [
        Color::rgb(1.0, 0.5, 0.0),  // Orange
        Color::rgb(0.5, 0.8, 1.0),  // Light blue
        Color::rgb(1.0, 0.8, 0.2),  // Yellow
        Color::rgb(0.8, 0.3, 0.5),  // Magenta
    ];

    for (idx, _body) in state.trajectory.bodies.iter().enumerate() {
        let color = colors[idx % colors.len()];
        let material = materials.add(StandardMaterial {
            base_color: color,
            ..default()
        });

        commands.spawn((
            PbrBundle {
                mesh: sphere_mesh.clone(),
                material,
                transform: Transform::default(),
                ..default()
            },
            BodyVisual { body_index: idx },
            BodyTrail {
                body_index: idx,
                positions: Vec::new(),
                max_trail_length: 500, // Keep last 500 positions
            },
        ));
    }

    println!("Controls:");
    println!("  SPACE: Play/Pause");
    println!("  LEFT:  Slow down");
    println!("  RIGHT: Speed up");
    println!("  R:     Reset to start");
}

fn update_positions(
    mut state: ResMut<ViewerState>,
    mut body_query: Query<(&BodyVisual, &mut Transform)>,
) {
    if state.is_playing && state.current_frame < state.trajectory.num_frames - 1 {
        state.current_frame = (state.current_frame as f32 + state.speed).min(state.trajectory.num_frames as f32 - 1.0) as usize;
    }

    for (body_visual, mut transform) in body_query.iter_mut() {
        if let Some(pos) = state.trajectory.bodies[body_visual.body_index]
            .get_position(state.current_frame)
        {
            transform.translation = Vec3::new(pos.x, pos.y, pos.z);
        }
    }
}

/// Update trail positions for each body
fn update_trails(
    state: Res<ViewerState>,
    mut trail_query: Query<(&Transform, &mut BodyTrail)>,
) {
    for (transform, mut trail) in trail_query.iter_mut() {
        trail.positions.push(transform.translation);
        
        // Keep only the most recent positions
        if trail.positions.len() > trail.max_trail_length {
            trail.positions.remove(0);
        }
    }
}

/// Render trails as line meshes
fn render_trails(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    trail_query: Query<&BodyTrail, Changed<BodyTrail>>,
) {
    let trail_colors = [
        Color::rgba(1.0, 0.5, 0.0, 0.6),    // Orange with transparency
        Color::rgba(0.5, 0.8, 1.0, 0.6),    // Light blue
        Color::rgba(1.0, 0.8, 0.2, 0.6),    // Yellow
        Color::rgba(0.8, 0.3, 0.5, 0.6),    // Magenta
    ];

    for trail in trail_query.iter() {
        if trail.positions.len() < 2 {
            continue;
        }

        // Create a line mesh for the trail
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        for pos in &trail.positions {
            positions.push([pos.x, pos.y, pos.z]);
        }

        // Create line segments connecting consecutive points
        for i in 0..positions.len() - 1 {
            indices.push(i as u32);
            indices.push((i + 1) as u32);
        }

        let mut mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::LineList,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

        let color = trail_colors[trail.body_index % trail_colors.len()];
        let material = materials.add(StandardMaterial {
            base_color: color,
            unlit: false,
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material,
            ..default()
        });
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ViewerState>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state.is_playing = !state.is_playing;
        println!("Playback: {}", if state.is_playing { "Playing" } else { "Paused" });
    }

    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        state.speed = (state.speed - 0.5).max(0.1);
        println!("Speed: {:.1}x", state.speed);
    }

    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        state.speed = state.speed + 0.5;
        println!("Speed: {:.1}x", state.speed);
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        state.current_frame = 0;
        println!("Reset to frame 0");
    }
}

fn update_ui(
    state: Res<ViewerState>,
) {
    // Print status occasionally (every 60 frames)
    if state.current_frame % 60 == 0 {
        let percent = (state.current_frame as f32 / state.trajectory.num_frames as f32) * 100.0;
        println!(
            "Frame: {}/{} ({:.1}%) | Speed: {:.1}x | Status: {}",
            state.current_frame,
            state.trajectory.num_frames,
            percent,
            state.speed,
            if state.is_playing { "Playing" } else { "Paused" }
        );
    }
}
