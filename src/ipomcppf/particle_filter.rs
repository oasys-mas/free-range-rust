use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug)]
pub struct ParticleFilter<T> {
    num_particles: usize,

    particles: Vec<T>,
    weights: Vec<f64>,
}

impl<T: Copy + Send + Sync> ParticleFilter<T> {
    pub fn new(num_particles: usize, particles: Vec<T>, weights: Vec<f64>) -> Self {
        let mut new = ParticleFilter {
            num_particles,
            particles,
            weights,
        };

        new.normalize();
        new.resample();

        return new;
    }

    pub fn resample(&mut self) {
        if self.particles.is_empty() || self.weights.is_empty() {
            panic!("Somehow the particle filter is empty.")
        }

        let dist = WeightedIndex::new(&self.weights).unwrap();
        let mut rng = thread_rng();

        let mut indices: Vec<usize> = Vec::with_capacity(self.num_particles as usize);
        for _ in 0..self.num_particles {
            indices.push(dist.sample(&mut rng));
        }

        let new_particles: Vec<T> = indices.par_iter().map(|&i| self.particles[i]).collect();
        let new_weights = vec![1.0 / self.num_particles as f64; self.num_particles];

        self.particles = new_particles;
        self.weights = new_weights;
    }

    pub fn normalize(&mut self) {
        let sum: f64 = self.weights.iter().sum();

        if f64::abs(sum - 1.0) > 1e-3 {
            self.weights.iter_mut().for_each(|w| *w /= sum);
        }
    }
}

impl<T: Display + Eq + Hash + Ord + Copy> Display for ParticleFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let unique_agg: HashMap<T, f64> =
            self.particles
                .iter()
                .zip(self.weights.iter())
                .fold(HashMap::new(), |mut acc, (particle, weight)| {
                    *acc.entry(*particle).or_insert(0.0) += *weight;
                    acc
                });

        let mut sorted_keys: Vec<_> = unique_agg.keys().collect();
        sorted_keys.sort();

        write!(f, "{{ ")?;
        for key in sorted_keys {
            let value = unique_agg.get(key).unwrap();
            write!(f, "{}: {:.4} ", key, value)?;
        }

        write!(f, "}}")
    }
}
