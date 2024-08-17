use argh::FromArgs;
use bevy::prelude::*;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::AmbientLight,
};

mod plugins;
use plugins::nbody::{BodyBundle, Gravity, NBody};
use plugins::pan_orbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

/**
Several `--startup` options:
* figure8: stable figure-8 three-body solution

Mouse controls:
* right-click & drag to orbit the camera
* scroll to zoom

*/
#[derive(FromArgs)]
struct Flags {
    /// speed of the simulation [default: 1.0x]
    #[argh(option, default = "1.0")]
    speed: f32,

    /// enable diagnostics in the console
    #[argh(switch, short = 'd')]
    debug: bool,
}

fn no_op_system() {}

fn main() {
    let args: Flags = argh::from_env();

    let mut app = App::build();
    app.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2.0,
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins);

    if args.debug {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(LogDiagnosticsPlugin::default());
    };

    app.add_plugin(PanOrbitCameraPlugin)
        .add_plugin(NBody {
            speed_factor: args.speed,
        })
        .add_startup_system(no_op_system.system());

    app.add_startup_system(solar_system.system());
    app.run();
}

fn spawn_z_camera(commands: &mut Commands, z: f32) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 0.0, z).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PanOrbitCamera {
            radius: z,
            ..Default::default()
        });
}

/// Add the sun and all the planets of the Solar system (+ Pluto)
/// Units are scaled:
/// Mass = 10^24 kg
/// Distance = AU (= 1.5 x 10^11 m)
/// Velocity = AU / Day
/// Acceleration = AU / DAY^2
pub fn solar_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut g: ResMut<Gravity>,
) {
    // Scale for rendering: 1 unit = 0.1 AU
    const AU_TO_UNIT_SCALE: f32 = 10.0;
    const DAY: f32 = 86_400.0;

    // Scale the gravitational constant accordingly to account for the units scaling
    // ```
    // G = m^3 / kg / s^2
    // G = (1.5^3 * 10^11 / 10 m)^3 / 10^24 kg / Day^2
    // G' = G * Day^2 * 10-6 / 1.5^3
    // ```
    g.0 *= DAY * DAY * 10.0f32.powi(-6) / 1.5f32.powi(3);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 2.8,
                subdivisions: 10,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::YELLOW.into(),
                roughness: 0.6,
                emissive: Color::YELLOW,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(Light {
            color: Color::WHITE,
            intensity: 50_000.0,
            range: 2000.0,
            ..Default::default()
        });

    macro_rules! spawn_planet {
    ($name:ident, m=$mass:literal, pos=($($pos:literal),+), vel=($($vel:literal),+), r=$radius:literal, col=$col:expr $(,)?) => {
        let $name = BodyBundle::new($mass, AU_TO_UNIT_SCALE * Vec3::new($($pos),+), AU_TO_UNIT_SCALE * Vec3::new($($vel),+));
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: $radius / 10_000.0,
                    subdivisions: 5,
                })),
                material: materials.add(StandardMaterial {
                    base_color: $col.into(),
                    roughness: 0.6,
                    reflectance: 0.1,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .insert_bundle($name);
    };
}
    // Data pulled from JPL Horizons as of 2021-04-18
    // https://ssd.jpl.nasa.gov/horizons.cgi
    #[rustfmt::skip]
    spawn_planet!(
        sun,
        m=1988500.0,
        pos=(0.0, 0.0, 0.0),
        vel=(0.0, 0.0, 0.0),
        r=696_340.0,
        col=Color::YELLOW,
    );

    #[rustfmt::skip]
    spawn_planet!(
        mercury,
        m=0.330,
        pos=(3.044, 0.130, -0.017),
        vel=(-0.016, 0.027, 0.004),
        r=2440.0,
        col=Color::ORANGE_RED,
    );

    #[rustfmt::skip]
    spawn_planet!(
        venus,
        m=4.868,
        pos=(0.539, 0.482, -0.024),
        vel=(-0.014, 0.015, 0.001),
        r=6051.84,
        col=Color::ORANGE,
    );

    #[rustfmt::skip]
    spawn_planet!(
        earth,
        m=5.972,
        pos=(-0.887, -0.470, 0.000),
        vel=(0.008, -0.015, 0.000),
        r=6371.01,
        col=Color::BLUE,
    );

    #[rustfmt::skip]
    spawn_planet!(
        mars,
        m=0.642,
        pos=(-0.767, 1.438, 0.049),
        vel=(-0.012, -0.005, 0.000),
        r=3389.92,
        col=Color::RED,
    );

    #[rustfmt::skip]
    spawn_planet!(
        jupiter,
        m=1898.187,
        pos=(3.638, -3.517, -0.067),
        vel=(0.005, 0.006, -0.000),
        r=69911.0,
        col=Color::BISQUE,
    );

    #[rustfmt::skip]
    spawn_planet!(
        saturn,
        m=568.340,
        pos=(5.947, -8.001, -0.098),
        vel=(0.004, 0.003, -0.000),
        r=58232.0,
        col=Color::GOLD,
    );

    #[rustfmt::skip]
    spawn_planet!(
        uranus,
        m=86.813,
        pos=(15.079, 12.767, -0.148),
        vel=(-0.003, 0.003, 0.000),
        r=25362.0,
        col=Color::AQUAMARINE,
    );

    #[rustfmt::skip]
    spawn_planet!(
        neptune,
        m=102.413,
        pos=(29.516, -4.898, -0.579),
        vel=(0.001, 0.003, -0.000),
        r=24622.0,
        col=Color::BLUE,
    );

    #[rustfmt::skip]
    spawn_planet!(
        pluto,
        m=0.013,
        pos=(14.375, -31.090, -0.830),
        vel=(0.003, 0.001, -0.001),
        r=11880.3,
        col=Color::GRAY,
    );

    spawn_z_camera(&mut commands, 200.0);
}
