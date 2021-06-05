use na::Vector3;
use rayon::prelude::*;
use ndarray::Array3;
use itertools::iproduct;
use std::cmp::{max, min};
use super::particle::Particle;

////////////////////////////////////////////////////////////
// Grid splits the space up into boxes
// Determines which particles can interact with each other
// To be used internally by State
//
#[derive(Clone, Copy)]
pub struct Grid {
    reach: usize,               // range of interactions (in grid squares) between particles
    unit_size: f64,             // size of a single grid square
}

impl Grid {
    pub fn new(unit_size: f64, reach: usize) -> Self {
        Self {
            reach,
            unit_size,
        }
    }

    // Calculate the interactions between particles using the grid approximation
    pub fn calculate_force(
        &self, 
        particles: &Vec<Particle>, 
    ) -> Vec<Vector3<f64>> 
    {

        let (grid, locations) = self.make_grid(particles);
        locations.par_iter()
            .enumerate()      // locations and particles has matching indices
            .map(|(particle_id, &location)| {
                self.calculate_force_single(particle_id, location, particles, &grid)
            })
            .collect()
    }

    // Calculate the total force acted on a particle by all nearby particles
    // Calculate the potential energy of the system 
    // To be used internally
    fn calculate_force_single(
        &self, 
        tpid: usize,                                        // target particle index
        loc: (usize, usize, usize),                         // target particle grid location
        particles: &Vec<Particle>,                          // Set of all particle
        grid: &Array3<Vec<usize>>,                          // division grid
    ) -> Vector3<f64> 
    {

        let relevant_grid_points = self.generate_neighbor_grid_loc(loc, grid);

        let relevant_particles = relevant_grid_points.into_iter()
            .flat_map(|(x, y, z)| &grid[[x, y, z]])
            .filter(|&&pid| pid != tpid)
            .map(|&pid| &particles[pid]);
        
        let mut total_f = Vector3::from_element(0.0);
        let mut total_p = 0.0;
        let target_particle = &particles[tpid];
        for particle in relevant_particles {
            // TODO: implement
        }

        total_f
    }

    // Generate indices that satisfy:
    //   Within reach of the input index
    //   Is a valid index in the grid
    // To be used internally
    fn generate_neighbor_grid_loc(
        &self, 
        loc: (usize, usize, usize), 
        grid: &Array3<Vec<usize>> 
    ) -> Vec<(usize, usize, usize)> {

        let (this_x, this_y, this_z) = loc;
        let (dim_x, dim_y, dim_z) = grid.dim();

        // iterators that cover the range of possible index values
        let xs = (this_x.saturating_sub(self.reach) ..= this_x + self.reach)
            .filter(|&x| x < dim_x);

        let ys = (this_y.saturating_sub(self.reach) ..= this_y + self.reach)
            .filter(|&y| y < dim_y);

        let zs = (this_z.saturating_sub(self.reach) ..= this_z + self.reach)
            .filter(|&z| z < dim_z);

        // return the cartesian product of xs, yx, zs
        iproduct!(xs, ys, zs).collect()
    }

    // Sort particles into grid locations
    // Is used to approximate particle interactions
    // Returns a Grid object that contains a list of particle indices
    //     and a list of location of the corresponding particle on the grid
    // to be used internally
    fn make_grid(&self, ps: &Vec<Particle>) -> (Array3<Vec<usize>>, Vec<(usize, usize, usize)>) {
        // get a list of positional indicies from the particles
        let locations: Vec<_> = ps.par_iter()
            .map(|p| self.find_grid_location(p.get_pos()))
            .collect();

        // find the smallest indexes to set the position of the origin
        let init_min = std::isize::MAX;
        let (xmin, ymin, zmin) = locations.iter()
            .fold((init_min, init_min, init_min), |(xacc, yacc, zacc), (x, y, z)| {
                (min(xacc, *x), min(yacc, *y), min(zacc, *z))
            });

        // translate the coordinate so that the smallest indices are at 0
        let locations: Vec<_> = locations.par_iter()
            .map(|(x, y, z)| 
                ((x - xmin) as usize, 
                 (y - ymin) as usize, 
                 (z - zmin) as usize))
            .collect();

        // find the largest indecies to find the size of the grid
        let init_max = std::usize::MIN;
        let (xmax, ymax, zmax) = locations.iter()
            .fold((init_max, init_max, init_max), |(xacc, yacc, zacc), (x, y, z)| {
                (max(xacc, *x), max(yacc, *y), max(zacc, *z))
            });

        // Making and adding indicies into the grid
        let mut grid = Array3::from_elem((xmax + 1, ymax + 1, zmax + 1), Vec::with_capacity(0));
        locations.iter()
            .enumerate()
            .for_each(|(i, (x, y, z))| grid[[*z, *y, *x]].push(i));

        (grid, locations)
    }

    // find location of a position on a grid
    // to be used internally
    fn find_grid_location(&self, p: &Vector3<f64>) -> (isize, isize, isize) {
        let gridx = f64::floor(p[0] / self.unit_size) as isize;
        let gridy = f64::floor(p[1] / self.unit_size) as isize;
        let gridz = f64::floor(p[2] / self.unit_size) as isize;

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
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Boundary {
    // Minimum length of each side of the box
    const MIN_LEN: f64 = 2.0; 
    const DEFLECT_STR: f64 = 10000.0;

    // Set up a boundary with default config
    pub fn new() -> Self {
        Self {
            x: 5.0,
            y: 5.0,
            z: 5.0,
        }
    }

    // Check for a valid box size
    pub fn is_valid(&self) -> bool {
        self.x >= Self::MIN_LEN && 
            self.y >= Self::MIN_LEN && 
            self.z >= Self::MIN_LEN
    }

    // check if the position vector lies within the box
    // is used for checking the initial state of particles
    pub fn contains_position(&self, pos: &Vector3<f64>) -> bool {
        let bound_check = self.bound_check(pos);
        bound_check.lp_norm(1) == 0.0
    }


    // Return a vector of forces that keeps the particles inside the box
    pub fn calculate_force(&self, ps: &Vec<Particle>) -> Vec<Vector3<f64>> {
        ps.par_iter()
            .map(|p| {
                self.calculate_force_single(p)
            })
            .collect()
    }

    ///////////////////////////////////////
    // Internal Utilities
    //

    // To be used internally by calculate_force
    fn calculate_force_single(&self, p: &Particle) -> Vector3<f64> {
        let bound_check = self.bound_check(p.get_pos());
        let force = Self::DEFLECT_STR * bound_check;
        force
    }

    // return a Vector3 showing the directions
    //   in which a position is out of bounds
    //   point towards the inside of the box
    // Example: If the particle is outside of the box
    //   and crossing the higher x wall by 10 units, the return value
    //   would be: (-10, 0, 0)
    // to be used internally
    fn bound_check(&self, pos: &Vector3<f64>) -> Vector3<f64> {
        let lower_bound = &[0.0, 0.0, 0.0];
        let upper_bound = &[self.x, self.y, self.z];
        
        let lower_bound_check = Vector3::from_iterator(
            pos.iter().zip(lower_bound)
                .map(|(x, x_lo_bnd)|
                    // zero out negative values
                    f64::max(x_lo_bnd - x, 0.0)));

        let upper_bound_check = Vector3::from_iterator(
            pos.iter().zip(upper_bound)
                .map(|(x, x_hi_bnd)| 
                    // zero out positive values
                    f64::min(x_hi_bnd - x, 0.0)));

        lower_bound_check + upper_bound_check 
    }
}
