use bevy::prelude::*;
use particle::{ParticleBundle, ParticlePlugin, Velocity};
use rand::Rng;

mod particle;

pub const SIZE: Vec3 = Vec3::splat(400.0);

const NUM_ELECTRONS: u32 = 1000;
const NUM_UP_QUARKS: u32 = 1000;
const NUM_DOWN_QUARKS: u32 = 1000;

struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ParticlePlugin).add_systems(Startup, setup);
    }
}

fn random_pos(min: Vec3, max: Vec3) -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(min.x..max.x);
    let y = rng.gen_range(min.y..max.y);
    let z = rng.gen_range(min.z..max.z);
    Vec3::new(x, y, z)
}

fn setup(mut commands: Commands) {
    commands.spawn_batch((0..NUM_ELECTRONS).map(|_| {
        ParticleBundle::electron(
            Transform::from_translation(random_pos(-SIZE, SIZE)),
            Velocity::default(),
        )
    }));
    commands.spawn_batch((0..NUM_UP_QUARKS).map(|_| {
        ParticleBundle::up_quark(
            Transform::from_translation(random_pos(-SIZE, SIZE)),
            Velocity::default(),
        )
    }));
    commands.spawn_batch((0..NUM_DOWN_QUARKS).map(|_| {
        ParticleBundle::down_quark(
            Transform::from_translation(random_pos(-SIZE, SIZE)),
            Velocity::default(),
        )
    }));

    commands.spawn(Camera2dBundle::default());
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(SimulationPlugin)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)));
    app.run();
}
