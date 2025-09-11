use itertools::Itertools;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub trait Space: Debug + Send + Sync {
    fn sample(&self) -> Option<Sample>;

    fn sample_with_seed(&self, seed: u64) -> Option<Sample>;

    fn enumerate(&self) -> Option<Vec<Sample>>;

    fn len(&self) -> Option<usize> {
        None
    }

    fn is_empty(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscreteSpace {
    pub n: i32,
    pub start: i32,
}

impl Space for DiscreteSpace {
    fn sample(&self) -> Option<Sample> {
        if self.n == 0 {
            return None;
        }

        let mut rng = StdRng::from_entropy();
        Some(Sample::Discrete(
            rng.gen_range(self.start..(self.start + self.n)),
        ))
    }

    fn sample_with_seed(&self, seed: u64) -> Option<Sample> {
        if self.n == 0 {
            return None;
        }

        let mut rng = StdRng::seed_from_u64(seed);
        Some(Sample::Discrete(
            rng.gen_range(self.start..(self.start + self.n)),
        ))
    }

    fn enumerate(&self) -> Option<Vec<Sample>> {
        if self.n == 0 {
            return None;
        }

        Some(
            (0..self.n)
                .map(|i| Sample::Discrete(i + self.start))
                .collect(),
        )
    }

    fn len(&self) -> Option<usize> {
        Some(self.n as usize)
    }

    fn is_empty(&self) -> bool {
        self.n == 0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxSpace {
    pub low: Vec<i32>,
    pub high: Vec<i32>,
}

impl Space for BoxSpace {
    fn sample(&self) -> Option<Sample> {
        if self.low.is_empty() || self.high.is_empty() {
            return None;
        }

        let mut rng = StdRng::from_entropy();
        Some(Sample::Box(
            self.low
                .iter()
                .zip(self.high.iter())
                .map(|(l, h)| rng.gen_range(*l..=*h))
                .collect(),
        ))
    }

    fn sample_with_seed(&self, seed: u64) -> Option<Sample> {
        if self.low.is_empty() || self.high.is_empty() {
            return None;
        }

        let mut rng = StdRng::seed_from_u64(seed);
        Some(Sample::Box(
            self.low
                .iter()
                .zip(self.high.iter())
                .map(|(l, h)| rng.gen_range(*l..=*h))
                .collect(),
        ))
    }

    fn enumerate(&self) -> Option<Vec<Sample>> {
        if self.low.is_empty() || self.high.is_empty() {
            return None;
        }

        if self.low.len() != self.high.len() {
            return None;
        }

        let ranges: Vec<Vec<i32>> = self
            .low
            .iter()
            .zip(self.high.iter())
            .map(|(l, h)| (*l..=*h).collect::<Vec<i32>>())
            .collect();
        if ranges.iter().any(|r| r.is_empty()) {
            return None;
        }

        let product = ranges
            .iter()
            .map(|r| r.iter().cloned())
            .multi_cartesian_product();
        Some(product.map(Sample::Box).collect())
    }

    fn len(&self) -> Option<usize> {
        Some(self.low.len())
    }

    fn is_empty(&self) -> bool {
        self.low.is_empty() || self.high.is_empty()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct TupleSpace {
    pub spaces: Vec<Box<dyn Space>>,
}

impl Space for TupleSpace {
    fn sample(&self) -> Option<Sample> {
        let mut samples = Vec::with_capacity(self.spaces.len());
        for s in &self.spaces {
            match s.sample() {
                Some(sample) => samples.push(sample),
                None => return None,
            }
        }
        Some(Sample::Tuple(samples))
    }
    fn sample_with_seed(&self, seed: u64) -> Option<Sample> {
        let mut samples = Vec::with_capacity(self.spaces.len());
        for (i, s) in self.spaces.iter().enumerate() {
            match s.sample_with_seed(seed + i as u64) {
                Some(sample) => samples.push(sample),
                None => return None,
            }
        }
        Some(Sample::Tuple(samples))
    }
    fn enumerate(&self) -> Option<Vec<Sample>> {
        let enumerations: Vec<Option<Vec<Sample>>> =
            self.spaces.iter().map(|s| s.enumerate()).collect();
        if enumerations.iter().any(|e| e.is_none()) {
            return None;
        }
        let enumerated: Vec<Vec<Sample>> = enumerations.into_iter().map(|e| e.unwrap()).collect();
        let product = enumerated
            .iter()
            .map(|v| v.iter().cloned())
            .multi_cartesian_product();
        Some(product.map(Sample::Tuple).collect())
    }
    fn len(&self) -> Option<usize> {
        Some(self.spaces.len())
    }
    fn is_empty(&self) -> bool {
        self.spaces.is_empty()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct DictSpace {
    pub spaces: HashMap<String, Box<dyn Space>>,
}

impl Space for DictSpace {
    fn sample(&self) -> Option<Sample> {
        let mut dict = HashMap::new();
        for (k, s) in &self.spaces {
            match s.sample() {
                Some(sample) => {
                    dict.insert(k.clone(), sample);
                }
                None => return None,
            }
        }
        Some(Sample::Dict(dict))
    }

    fn sample_with_seed(&self, seed: u64) -> Option<Sample> {
        let mut dict = HashMap::new();
        for (i, (k, s)) in self.spaces.iter().enumerate() {
            match s.sample_with_seed(seed + i as u64) {
                Some(sample) => {
                    dict.insert(k.clone(), sample);
                }
                None => return None,
            }
        }
        Some(Sample::Dict(dict))
    }

    fn enumerate(&self) -> Option<Vec<Sample>> {
        let keys: Vec<_> = self.spaces.keys().cloned().collect();
        let enumerations: Vec<Option<Vec<Sample>>> = keys
            .iter()
            .map(|key| self.spaces[key].enumerate())
            .collect();
        if enumerations.iter().any(|e| e.is_none()) {
            return None;
        }

        let enumerated: Vec<Vec<Sample>> = enumerations.into_iter().map(|e| e.unwrap()).collect();
        let product = enumerated
            .iter()
            .map(|v| v.iter().cloned())
            .multi_cartesian_product();

        Some(
            product
                .map(|samples| {
                    let mut dict = HashMap::new();
                    for (k, v) in keys.iter().zip(samples.into_iter()) {
                        dict.insert(k.clone(), v);
                    }
                    Sample::Dict(dict)
                })
                .collect(),
        )
    }

    fn len(&self) -> Option<usize> {
        Some(self.spaces.len())
    }

    fn is_empty(&self) -> bool {
        self.spaces.is_empty()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct OneOfSpace {
    pub spaces: Vec<Box<dyn Space>>,
}

impl Space for OneOfSpace {
    fn sample(&self) -> Option<Sample> {
        let valid_spaces: Vec<_> = self
            .spaces
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                if let Some(ds) = s.as_any().downcast_ref::<DiscreteSpace>() {
                    ds.n > 0
                } else {
                    true
                }
            })
            .collect();
        if valid_spaces.is_empty() {
            return None;
        }

        let mut rng = StdRng::from_entropy();
        let (index, sub_space) = valid_spaces[rng.gen_range(0..valid_spaces.len())];
        sub_space
            .sample()
            .map(|sample| Sample::OneOf(index as i32, Box::new(sample)))
    }

    fn sample_with_seed(&self, seed: u64) -> Option<Sample> {
        let valid_spaces: Vec<_> = self
            .spaces
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                if let Some(ds) = s.as_any().downcast_ref::<DiscreteSpace>() {
                    ds.n > 0
                } else {
                    true
                }
            })
            .collect();
        if valid_spaces.is_empty() {
            return None;
        }

        let mut rng = StdRng::seed_from_u64(seed);
        let (index, sub_space) = valid_spaces[rng.gen_range(0..valid_spaces.len())];
        sub_space
            .sample_with_seed(seed + 1)
            .map(|sample| Sample::OneOf(index as i32, Box::new(sample)))
    }

    fn enumerate(&self) -> Option<Vec<Sample>> {
        let mut all_samples = Vec::new();
        for (idx, s) in self.spaces.iter().enumerate() {
            match s.enumerate() {
                Some(sub_results) => {
                    for sample in sub_results {
                        all_samples.push(Sample::OneOf(idx as i32, Box::new(sample)));
                    }
                }
                None => continue,
            }
        }
        if all_samples.is_empty() {
            None
        } else {
            Some(all_samples)
        }
    }
    fn len(&self) -> Option<usize> {
        Some(self.spaces.len())
    }
    fn is_empty(&self) -> bool {
        self.spaces.is_empty()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sample {
    // A value sampled from a discrete space.
    Discrete(i32),

    // A value sampled from one of multiple sub-spaces.
    OneOf(i32, Box<Sample>),

    // A value sampled from a box space.
    Box(Vec<i32>),

    // A value sampled from a tuple space.
    Tuple(Vec<Sample>),

    // A value sampled from a dictionary space.
    Dict(HashMap<String, Sample>),
}

impl Hash for Sample {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Sample::Discrete(val) => val.hash(state),
            Sample::OneOf(idx, box_sample) => {
                idx.hash(state);
                box_sample.hash(state);
            }
            Sample::Box(vec) => {
                vec.hash(state);
            }
            Sample::Tuple(vec) => {
                vec.hash(state);
            }
            Sample::Dict(map) => {
                let mut sorted_entries: Vec<_> = map.iter().collect();
                sorted_entries.sort_by(|a, b| a.0.cmp(b.0));
                for (key, value) in sorted_entries {
                    key.hash(state);
                    value.hash(state);
                }
            }
        }
    }
}
