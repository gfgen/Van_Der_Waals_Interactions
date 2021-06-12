use super::physics;
use crate::trans_rot_complexes::*;
use bevy::prelude::Vec3;
use itertools::iproduct;
use ndarray::Array3;
use rayon::prelude::*;
use std::cmp::{max, min};

////////////////////////////////////////////////////////////
// Grid splits the space up into boxes
// Determines which particles can interact with each other
// To be used internally by State
//
#[derive(Clone, Copy)]
pub struct Grid {
    reach: usize,   // range of interactions (in grid squares) between particles
    unit_size: f32, // size of a single grid square
}

impl Grid {
    pub fn new(unit_size: f32, reach: usize) -> Self {
        Self { reach, unit_size }
    }

    // Calculate the interactions between particles using the grid approximation
    // Return (accelerations, potential energies, # of neighbors)
    pub fn calculate_force(&self, particles: &Vec<TRC>) -> (Vec<TRCInfintesimal>, Vec<f32>, Vec<usize>) {
        let (grid, particle_locations) = self.make_grid(particles);
        let (accelerations, (potential_energies, neighbors)) = particle_locations
            .par_iter()
            .enumerate() // locations and particles has matching indices
            .map(|(particle_id, &location)| {
                self.calculate_force_single(particle_id, location, particles, &grid)
            })
            .unzip();

        (accelerations, potential_energies, neighbors)
    }

    // Calculate the total force acted on a particle by all nearby particles
    // Calculate the potential energy of the system
    // Awkward return format so that it can be used by unzip
    // To be used internally
    fn calculate_force_single(
        &self,
        tpid: usize,                // target particle index
        loc: (usize, usize, usize), // target particle grid location
        particles: &Vec<TRC>,      // Set of all particle positions
        grid: &Array3<Vec<usize>>,  // division grid
    ) -> (TRCInfintesimal, (f32, usize)) {
        let relevant_grid_points = self.generate_neighbor_grid_loc(loc, grid);

        let relevant_particles = relevant_grid_points
            .into_iter()
            .flat_map(|(x, y, z)| &grid[[x, y, z]]) // retrieve particle ids from grid points
            .filter(|&&pid| pid != tpid) // remove target particle id
            .map(|&pid| particles[pid]); // retrieve particles from particle ids

        let mut total_force = TRCInfintesimal::ZERO;
        let mut total_potential = 0.0;
        let mut total_neighbor = 0;
        let target_particle = particles[tpid];
        // iterate through relevant particles, sum up forces and potentials
        for other_particle in relevant_particles {
            let range = self.unit_size * self.reach as f32;

            let (force, potential, neighbor) =
                physics::vdw_interaction(target_particle, other_particle, range);

            total_force += force;
            total_potential += potential;
            total_neighbor += neighbor;
        }

        (total_force, (total_potential, total_neighbor))
    }

    // Generate indices that satisfy:
    //   Within reach of the input index
    //   Is a valid index in the grid
    // To be used internally
    fn generate_neighbor_grid_loc(
        &self,
        loc: (usize, usize, usize),
        grid: &Array3<Vec<usize>>,
    ) -> Vec<(usize, usize, usize)> {
        let (this_x, this_y, this_z) = loc;
        let (dim_x, dim_y, dim_z) = grid.dim();

        // iterators that cover the range of possible index values
        let xs = (this_x.saturating_sub(self.reach)..=this_x + self.reach).filter(|&x| x < dim_x);

        let ys = (this_y.saturating_sub(self.reach)..=this_y + self.reach).filter(|&y| y < dim_y);

        let zs = (this_z.saturating_sub(self.reach)..=this_z + self.reach).filter(|&z| z < dim_z);

        // return the cartesian product of xs, yx, zs
        iproduct!(xs, ys, zs).collect()
    }

    // Sort particles into grid locations
    // Is used to approximate particle interactions
    // Returns a Grid object that contains a list of particle indices
    //     and a list of locations of the corresponding particles on the grid
    // to be used internally
    fn make_grid(&self, ps: &Vec<TRC>) -> (Array3<Vec<usize>>, Vec<(usize, usize, usize)>) {
        // get a list of positional indicies from the particles
        let grid_locations: Vec<_> = ps.par_iter().map(|&p| self.find_grid_location(p.translation)).collect();

        // find the smallest indexes to set the position of the origin
        let init_min = std::isize::MAX;
        let (xmin, ymin, zmin) = grid_locations.iter().fold(
            (init_min, init_min, init_min),
            |(xacc, yacc, zacc), (x, y, z)| (min(xacc, *x), min(yacc, *y), min(zacc, *z)),
        );

        // translate the coordinate so that the smallest indices are at 0
        let grid_locations: Vec<_> = grid_locations
            .par_iter()
            .map(|(x, y, z)| {
                (
                    (x - xmin) as usize,
                    (y - ymin) as usize,
                    (z - zmin) as usize,
                )
            })
            .collect();

        // find the largest indecies to find the size of the grid
        let init_max = std::usize::MIN;
        let (xmax, ymax, zmax) = grid_locations.iter().fold(
            (init_max, init_max, init_max),
            |(xacc, yacc, zacc), (x, y, z)| (max(xacc, *x), max(yacc, *y), max(zacc, *z)),
        );

        // Making and adding indicies into the grid
        let mut grid = Array3::from_elem((xmax + 1, ymax + 1, zmax + 1), Vec::with_capacity(0));
        grid_locations
            .iter()
            .enumerate()
            .for_each(|(i, (x, y, z))| grid[[*x, *y, *z]].push(i));

        (grid, grid_locations)
    }

    // find location of a position on a grid
    // to be used internally
    fn find_grid_location(&self, p: Vec3) -> (isize, isize, isize) {
        let gridx = f32::floor(p[0] / self.unit_size) as isize;
        let gridy = f32::floor(p[1] / self.unit_size) as isize;
        let gridz = f32::floor(p[2] / self.unit_size) as isize;

        (gridx, gridy, gridz)
    }
}

////////////////////////////////////////////////////////////////
// Boundary sets the limit of the simulation box
// Is responsible for keeping the particles within its border
// Box lower corner always at origin
// Box can only extend in one direction for each dimension
// To be used internally by State
//
#[derive(Clone, Copy)]
pub struct Boundary {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Boundary {
    const MIN_LEN: f32 = 2.0; // Minimum length of each side of the box
    const DEFLECT_STR: f32 = 10000.0;

    // Set up a boundary with default config
    pub fn new() -> Self {
        Self {
            x: 5.0,
            y: 5.0,
            z: 5.0,
        }
    }

    // Surface area of the boundary, useful for calculating pressure
    pub fn get_surface_area(&self) -> f32 {
        (self.x * self.y + self.y * self.z + self.z * self.x) * 2.0
    }

    // Volume inside of the boundary
    pub fn get_volume(&self) -> f32 {
        self.x * self.y * self.z
    }

    // Coordinates of the corner with higher values
    pub fn hi_corner(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    // Coordinates of the corner with lower values
    pub fn lo_corner(&self) -> Vec3 {
        Vec3::ZERO
    }

    // Coordinates of center of box
    pub fn center(&self) -> Vec3 {
        (self.hi_corner() + self.lo_corner()) / 2.0
    }

    // Check for a valid box size
    pub fn is_valid(&self) -> bool {
        self.x >= Self::MIN_LEN && self.y >= Self::MIN_LEN && self.z >= Self::MIN_LEN
    }

    // check if the position vector lies within the box
    // is used for checking the initial state of particles
    pub fn contains_position(&self, pos: Vec3) -> bool {
        let bound_check = self.bound_check(pos);
        bound_check.length_squared() == 0.0
    }

    // Return a vector of forces that keeps the particles inside the box
    pub fn calculate_force(&self, ps: &Vec<TRC>) -> Vec<TRCInfintesimal> {
        ps.par_iter()
            .map(|&p| TRCInfintesimal::new(self.calculate_force_single(p.translation), Vec3::ZERO))
            .collect()
    }

    ///////////////////////////////////////
    // Interactive utilities
    pub fn expand(&mut self, rate: f32, dt: f32) {
        self.x = (self.x + rate * dt).max(Boundary::MIN_LEN);
        self.y = (self.y + rate * dt).max(Boundary::MIN_LEN);
        self.z = (self.z + rate * dt).max(Boundary::MIN_LEN);
    }
    ///////////////////////////////////////
    // Internal Utilities
    //

    // To be used internally by calculate_force
    fn calculate_force_single(&self, p: Vec3) -> Vec3 {
        let bound_check = self.bound_check(p);
        let force = Self::DEFLECT_STR * bound_check;
        force
    }

    // return a Vec3 showing the directions
    //   in which a position is out of bounds
    //   point towards the inside of the box
    // Example: If the particle is outside of the box
    //   and crossing the higher x wall by 10 units, the return value
    //   would be: (-10, 0, 0)
    // to be used internally
    fn bound_check(&self, pos: Vec3) -> Vec3 {
        let lower_bound_check = Vec3::max(self.lo_corner() - pos, Vec3::ZERO); // zero out negative values
        let upper_bound_check = Vec3::min(self.hi_corner() - pos, Vec3::ZERO); // zero out positive values

        lower_bound_check + upper_bound_check
    }
}
