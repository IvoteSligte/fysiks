use bevy::{
    prelude::*,
    render::mesh::CircleMeshBuilder,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::SIZE;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                update,
                velocity_update,
                loop_translation_update,
                mass_update,
            )
                .chain(),
        );
    }
}

fn setup(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    for Visualisation {
        material,
        mesh,
        radius,
        color,
    } in Visualisation::ALL
    {
        meshes.insert(mesh, CircleMeshBuilder::new(radius, 5).build());
        materials.insert(material, color.into());
    }
}

#[derive(Bundle)]
pub struct ParticleBundle {
    particle: Particle,
    loop_translation: LoopTranslation,
    /// Velocity of a particle in m/s * k_e / e
    velocity: Velocity,
    /// Mass of a particle in e_v
    mass: Mass,
    /// Visualisation of the particle
    /// And transform of a particle in m * k_e / e (m * Coulomb's constant / elementary charge)
    material_mesh_2d_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

impl ParticleBundle {
    fn new(
        particle: Particle,
        visualisation: Visualisation,
        transform: Transform,
        velocity: Velocity,
    ) -> Self {
        Self {
            particle,
            loop_translation: LoopTranslation,
            mass: Mass(particle.mass),
            velocity,
            material_mesh_2d_bundle: MaterialMesh2dBundle {
                transform,
                material: visualisation.material,
                mesh: Mesh2dHandle(visualisation.mesh),
                ..default()
            },
        }
    }

    pub fn electron(transform: Transform, velocity: Velocity) -> Self {
        Self::new(
            Particle::ELECTRON,
            Visualisation::ELECTRON,
            transform,
            velocity,
        )
    }

    pub fn up_quark(transform: Transform, velocity: Velocity) -> Self {
        Self::new(
            Particle::UP_QUARK,
            Visualisation::UP_QUARK,
            transform,
            velocity,
        )
    }

    pub fn down_quark(transform: Transform, velocity: Velocity) -> Self {
        Self::new(
            Particle::DOWN_QUARK,
            Visualisation::DOWN_QUARK,
            transform,
            velocity,
        )
    }
}

pub struct Visualisation {
    material: Handle<ColorMaterial>,
    mesh: Handle<Mesh>,
    radius: f32,
    color: Color,
}

impl Visualisation {
    pub const ELECTRON: Self = Self {
        material: Handle::weak_from_u128(13652880569953508365),
        mesh: Handle::weak_from_u128(6525358019767708978),
        radius: 1.0, // not realistic
        color: Color::rgb_linear(0.3, 0.3, 1.0),
    };
    pub const UP_QUARK: Self = Self {
        material: Handle::weak_from_u128(15457461644197111779),
        mesh: Handle::weak_from_u128(15378691927712692583),
        radius: 0.5, // not realistic
        color: Color::rgb_linear(0.8, 0.3, 0.3),
    };
    pub const DOWN_QUARK: Self = Self {
        material: Handle::weak_from_u128(6991950750307369590),
        mesh: Handle::weak_from_u128(7421317758493431304),
        radius: 0.5, // not realistic
        color: Color::rgb_linear(0.3, 0.8, 0.3),
    };

    pub const ALL: [Self; 3] = [Self::ELECTRON, Self::UP_QUARK, Self::DOWN_QUARK];
}

#[derive(Component)]
pub struct LoopTranslation;

fn loop_translation_update(mut query: Query<&mut Transform>) {
    // does not account for any scaling factor or movement of the camera
    let dimensions = SIZE.xy();

    for mut transform in query.iter_mut() {
        let p = &mut transform.translation;

        if p.x > dimensions.x {
            p.x = -dimensions.x;
        } else if p.x < -dimensions.x {
            p.x = dimensions.x;
        }
        if p.y > dimensions.y {
            p.y = -dimensions.y;
        } else if p.y < -dimensions.y {
            p.y = dimensions.y;
        }
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(Vec3);

fn velocity_update(mut query: Query<(&mut Transform, &Velocity)>) {
    query.par_iter_mut().for_each(|(mut transform, velocity)| {
        transform.translation += velocity.0;
    })
}

#[derive(Component, Clone, Copy)]
pub struct Mass(f32);

fn mass_update(mut query: Query<(&mut Mass, &Particle, &Velocity)>) {
    query
        .par_iter_mut()
        .for_each(|(mut mass, particle, velocity)| {
            mass.0 = particle.mass + velocity.0.length();
        });
}

#[derive(Component, Clone, Copy)]
pub struct Particle {
    /// charge in elementary charges
    charge: f32,
    /// mass of the particle in electronvolts
    mass: f32,
}

impl Particle {
    pub const ELECTRON: Self = Self {
        charge: -1.0,
        mass: 0.51099895,
    };
    pub const UP_QUARK: Self = Self {
        charge: 2.0 / 3.0,
        mass: (3.0 + 1.8) / 2.0, // average of its upper and lower limits
    };
    pub const DOWN_QUARK: Self = Self {
        charge: -1.0 / 3.0,
        mass: (5.8 + 4.1) / 2.0, // average of its upper and lower limits
    };
}

fn calculate_impulse<'a>(
    particles: impl ParallelIterator<Item = &'a (&'a Particle, &'a Transform)>,
    properties: Particle,
    mass: Mass,
    translation: Vec3,
    delta_time: f32,
) -> Vec3 {
    let t1 = translation;

    // partially calculated force using Coulomb's law
    let semi_force = particles
        .map(
            |(
                p2,
                &Transform {
                    translation: t2, ..
                },
            )| {
                let diff = t1 - t2;
                let dist_squared = diff.length_squared();

                if dist_squared <= f32::EPSILON {
                    return Vec3::ZERO;
                }
                let dir = diff / dist_squared.sqrt();
                dir * p2.charge / (dist_squared + 1.0)
            },
        )
        .sum::<Vec3>();

    semi_force * (properties.charge / mass.0 * delta_time)
}

#[allow(clippy::type_complexity)]
pub fn update(
    mut query_mut: Query<(Entity, &Mass, &mut Velocity), (With<Particle>, With<Transform>)>,
    query: Query<(&Particle, &Transform), With<Velocity>>,
    time: Res<Time>,
) {
    let particles = query.iter().collect::<Vec<_>>();

    query_mut
        .par_iter_mut()
        .for_each(|(entity, &mass, mut vel)| {
            let (&prop, trans) = query.get(entity).unwrap();

            vel.0 += calculate_impulse(
                particles.par_iter(),
                prop,
                mass,
                trans.translation,
                time.delta_seconds(),
            );
        });
}
