use bevy::prelude::*;
use threebody_sim::TrajectoryData;
use std::env;

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
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 3e11).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
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
