use super::particle::Particle;
use super::sim_space::Boundary;
use super::SimulationPrototype;
use bevy::prelude::Vec3;
use rand::Rng;
use rand_distr::StandardNormal;

pub trait Initialize: Sized {
    fn get_bound(&self) -> Boundary;
    fn set_particles(self, particles: Vec<Particle>) -> Self;
    fn initialize_spherical_cloud(self, n: usize, sigma: f32, temp: f32) -> Self {
        let bound = self.get_bound();
        let mut rng = rand::thread_rng();
        let mut particles = vec![];

        for _i in 0..n {
            let mut pos = Vec3::new(
                rng.sample(StandardNormal),
                rng.sample(StandardNormal),
                rng.sample(StandardNormal),
            );
            pos = (pos * sigma) + bound.center(); // control spread and move to center of boundary

            // Trim invalid positions
            pos = pos.min(bound.hi_corner());
            pos = pos.max(bound.lo_corner());

            particles.push(
                Particle::new(rng.gen_bool(0.5))
                    .set_pos(pos.x, pos.y, pos.z)
                    .set_vel(
                        rng.sample::<f32, _>(StandardNormal) * temp,
                        rng.sample::<f32, _>(StandardNormal) * temp,
                        rng.sample::<f32, _>(StandardNormal) * temp,
                    ),
            );
        }
        self.set_particles(prune(particles))
    }
}

impl Initialize for SimulationPrototype {
    fn get_bound(&self) -> Boundary {
        self.bound
    }

    fn set_particles(mut self, particles: Vec<Particle>) -> Self {
        self.particles = particles;
        self
    }
}

// Delete particles that are too close to each other
fn prune(particles: Vec<Particle>) -> Vec<Particle> {
    let mut ret: Vec<Particle> = vec![];
    for p1 in particles.iter() {
        let mut qual = true;
        for p2 in ret.iter() {
            let r = p1.get_pos() - p2.get_pos();
            let rnorm = r.length();
            if rnorm == 0.0 {
                continue;
            }
            qual = qual && rnorm >= 0.15
        }
        if qual {
            ret.push(p1.clone());
        }
    }
    ret
}
