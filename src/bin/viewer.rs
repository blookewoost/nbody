use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
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

/// Calculate camera position from spherical coordinates around a target
fn calculate_camera_position(target: Vec3, distance: f32, yaw: f32, pitch: f32) -> Vec3 {
    let x = target.x + distance * pitch.cos() * yaw.sin();
    let y = target.y + distance * pitch.sin();
    let z = target.z + distance * pitch.cos() * yaw.cos();
    Vec3::new(x, y, z)
}

/// Main viewer state
#[derive(Resource)]
struct ViewerState {
    trajectory: TrajectoryData,
    current_frame: usize,
    is_playing: bool,
    speed: f32, // Frames per update
    centroid: Vec3,
    camera_distance: f32,
}

/// Camera control state for mouse-based rotation
#[derive(Resource)]
struct CameraState {
    yaw: f32,   // Horizontal rotation
    pitch: f32, // Vertical rotation
    is_dragging: bool,
    last_mouse_pos: Vec2,
    zoom: f32,  // Zoom multiplier (1.0 = default, <1.0 = zoomed in, >1.0 = zoomed out)
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            yaw: 0.45,   // Initial angle
            pitch: 0.64, // Initial angle
            is_dragging: false,
            last_mouse_pos: Vec2::ZERO,
            zoom: 1.0,
        }
    }
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

    let (centroid, max_distance) = calculate_camera_target(&trajectory);
    let camera_distance = max_distance * 2.5;

    let viewer_state = ViewerState {
        trajectory,
        current_frame: 0,
        is_playing: true,
        speed: 1.0,
        centroid,
        camera_distance,
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
        .insert_resource(CameraState::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_mouse_input,
            update_camera,
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
    camera_state: Res<CameraState>,
) {
    // Calculate initial camera position using stored centroid and distance with initial angles
    let camera_pos = calculate_camera_position(
        state.centroid,
        state.camera_distance,
        camera_state.yaw,
        camera_state.pitch,
    );

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(camera_pos.x, camera_pos.y, camera_pos.z)
            .looking_at(state.centroid, Vec3::Y),
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

        // Initialize trail with the first position
        let mut initial_positions = Vec::new();
        if let Some(pos) = state.trajectory.bodies[idx].get_position(0) {
            initial_positions.push(Vec3::new(pos.x, pos.y, pos.z));
        }

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
                positions: initial_positions,
                max_trail_length: 500, // Keep last 500 positions
            },
        ));
    }

    println!("Controls:");
    println!("  Mouse Drag: Rotate the view");
    println!("  Mouse Wheel: Zoom in/out");
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

/// Render trails as tube meshes for visibility
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

        let tube_radius = 2e9; // Radius of the tube along the trail
        let segments_per_section = 8; // Number of segments around the tube

        let mut positions = Vec::new();
        let mut indices = Vec::new();

        // Create tube mesh from the trail positions
        for (seg_idx, segment) in trail.positions.windows(2).enumerate() {
            let p1 = segment[0];
            let p2 = segment[1];
            let direction = (p2 - p1).normalize();

            // Create perpendicular vectors for the tube cross-section
            let up = if direction.dot(Vec3::Y).abs() < 0.9 {
                Vec3::Y
            } else {
                Vec3::X
            };
            let right = direction.cross(up).normalize();
            let local_up = direction.cross(right).normalize();

            // Create ring of vertices around p1
            let ring_start = positions.len() as u32;
            for i in 0..segments_per_section {
                let angle = (i as f32 / segments_per_section as f32) * std::f32::consts::TAU;
                let offset = (right * angle.cos() + local_up * angle.sin()) * tube_radius;
                positions.push([p1.x + offset.x, p1.y + offset.y, p1.z + offset.z]);
            }

            // Create ring of vertices around p2
            let next_ring_start = positions.len() as u32;
            for i in 0..segments_per_section {
                let angle = (i as f32 / segments_per_section as f32) * std::f32::consts::TAU;
                let offset = (right * angle.cos() + local_up * angle.sin()) * tube_radius;
                positions.push([p2.x + offset.x, p2.y + offset.y, p2.z + offset.z]);
            }

            // Connect the two rings with triangles
            for i in 0..segments_per_section {
                let next_i = (i + 1) % segments_per_section;

                // First triangle
                indices.push(ring_start + i as u32);
                indices.push(ring_start + next_i as u32);
                indices.push(next_ring_start + i as u32);

                // Second triangle
                indices.push(ring_start + next_i as u32);
                indices.push(next_ring_start + next_i as u32);
                indices.push(next_ring_start + i as u32);
            }
        }

        if positions.is_empty() {
            continue;
        }

        let mut mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
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

/// Handle mouse input for camera control
fn handle_mouse_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_state: ResMut<CameraState>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        camera_state.is_dragging = true;
        
        for event in mouse_motion_events.read() {
            // Sensitivity factor for camera rotation
            let sensitivity = 0.01;
            
            // Update yaw (horizontal) and pitch (vertical)
            camera_state.yaw += event.delta.x * sensitivity;
            camera_state.pitch = (camera_state.pitch + event.delta.y * sensitivity)
                .clamp(-std::f32::consts::PI / 2.0 + 0.1, std::f32::consts::PI / 2.0 - 0.1);
        }
    } else {
        camera_state.is_dragging = false;
        // Consume remaining events even when not dragging
        for _ in mouse_motion_events.read() {}
    }

    // Handle mouse wheel for zoom
    for event in mouse_wheel_events.read() {
        match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                let zoom_factor = 1.1;
                if event.y > 0.0 {
                    camera_state.zoom /= zoom_factor; // Zoom in
                    camera_state.zoom = camera_state.zoom.max(0.1); // Min zoom
                } else {
                    camera_state.zoom *= zoom_factor; // Zoom out
                    camera_state.zoom = camera_state.zoom.min(10.0); // Max zoom
                }
            }
            _ => {}
        }
    }
}

/// Update camera position based on angles and zoom
fn update_camera(
    state: Res<ViewerState>,
    camera_state: Res<CameraState>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        let effective_distance = state.camera_distance * camera_state.zoom;
        let camera_pos = calculate_camera_position(
            state.centroid,
            effective_distance,
            camera_state.yaw,
            camera_state.pitch,
        );
        
        *camera_transform = Transform::from_xyz(camera_pos.x, camera_pos.y, camera_pos.z)
            .looking_at(state.centroid, Vec3::Y);
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ViewerState>,
    mut trail_query: Query<&mut BodyTrail>,
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
        
        // Clear all trails
        for mut trail in trail_query.iter_mut() {
            // Keep only the first position (the initial position)
            if !trail.positions.is_empty() {
                let first_pos = trail.positions[0];
                trail.positions.clear();
                trail.positions.push(first_pos);
            }
        }
        
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
