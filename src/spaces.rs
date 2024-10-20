use pyo3::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rayon::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub enum Space {
    /// A discrete space with a range of values.
    Discrete { n: i32, start: i32 },

    /// A space that represents one of multiple possible sub-spaces.
    OneOf { spaces: Vec<Space> },

    /// A box space defined by lower and upper bounds.
    Box { low: Vec<i32>, high: Vec<i32> },

    /// A vector space containing multiple sub-spaces.
    Vector { spaces: Vec<Space> },
}

impl Space {
    pub fn new_discrete(n: i32, start: i32) -> Self {
        Space::Discrete { n, start }
    }

    pub fn new_one_of(spaces: Vec<Space>) -> Self {
        Space::OneOf { spaces }
    }

    pub fn new_vector(spaces: Vec<Space>) -> Self {
        Space::Vector { spaces }
    }

    pub fn new_box(low: Vec<i32>, high: Vec<i32>) -> Self {
        Space::Box { low, high }
    }
}

#[pymethods]
impl Space {
    #[new]
    #[pyo3(signature=(variant, n=None, start=None, spaces=None, low=None, high = None))]
    fn py_new(
        variant: &str,
        n: Option<i32>,
        start: Option<i32>,
        spaces: Option<Vec<Space>>,
        low: Option<Vec<i32>>,
        high: Option<Vec<i32>>,
    ) -> Self {
        match variant {
            "discrete" => Space::Discrete {
                n: n.expect("Discrete space requires n"),
                start: start.unwrap_or(0),
            },
            "box" => {
                let high = high.expect("Box space requires high");
                Space::Box {
                    low: low.unwrap_or(vec![0; high.len()]),
                    high,
                }
            }
            "oneof" => Space::OneOf {
                spaces: spaces.expect("OneOf spaces requires spaces"),
            },
            "vector" => Space::Vector {
                spaces: spaces.expect("OneOf spaces requires spaces"),
            },
            _ => panic!("Unknown variant: {}", variant),
        }
    }

    pub fn sample(&self) -> Vec<i32> {
        //!  Sample a single value from the space.
        let mut rng = StdRng::from_entropy();

        let result = match self {
            Space::Discrete { n, start, .. } => vec![rng.gen_range(*start..(*start + *n))],
            Space::Box { low, high } => low
                .iter()
                .zip(high.iter())
                .map(|(l, h)| rng.gen_range(*l..=*h))
                .collect(),
            Space::OneOf { spaces, .. } => {
                let index = rng.gen_range(0..spaces.len());
                let sub_sample = &spaces[index].sample();
                let mut result = vec![index as i32];
                result.extend(sub_sample);
                result
            }
            _ => panic!("Cannot call sample on vector space"),
        };

        result
    }

    ///  Sample a single value from the space with a fixed seed.
    pub fn sample_with_seed(&self, seed: u64) -> Vec<i32> {
        let mut rng = StdRng::seed_from_u64(seed);

        match self {
            Space::Discrete { n, start, .. } => vec![rng.gen_range(*start..(*start + *n))],
            Space::Box { low, high } => low
                .iter()
                .zip(high.iter())
                .map(|(l, h)| rng.gen_range(*l..=*h))
                .collect(),
            Space::OneOf { spaces, .. } => {
                let index = rng.gen_range(0..spaces.len());
                let mut result = spaces[index].sample_with_seed(seed + 1);
                result.insert(0, index as i32);
                result
            }
            _ => panic!("Cannot call sample on vector space"),
        }
    }

    /// Sample a single value from each of the nested spaces.
    pub fn sample_nested(&self) -> Vec<Vec<i32>> {
        match self {
            Space::Vector { spaces } => spaces.par_iter().map(|space| space.sample()).collect(),
            _ => panic!("Cannot call sample_nested on non-vector space"),
        }
    }

    /// Sample a single value from each of the nested spaces with a fixed seed.
    pub fn sample_nested_with_seed(&self, seed: u64) -> Vec<Vec<i32>> {
        match self {
            Space::Vector { spaces } => spaces.par_iter().map(|space| space.sample_with_seed(seed)).collect(),
            _ => panic!("Cannot call sample_nested on non-vector space"),
        }
    }

    /// Enumerate all possible values in the space.
    pub fn enumerate(&self) -> Vec<Vec<i32>> {
        match self {
            Space::Discrete { n, start } => (0..*n).map(|i| vec![i + *start]).collect(),
            Space::Box { low, high } => {
                let ranges: Vec<Vec<i32>> = low.iter().zip(high.iter()).map(|(&l, &h)| (l..=h).collect()).collect();

                ranges.iter().fold(vec![vec![]], |acc, range| {
                    acc.into_par_iter() // Parallel iterator for the accumulated combinations
                        .flat_map(|prefix| {
                            range.par_iter().map(move |&val| {
                                let mut new_combination = prefix.clone();
                                new_combination.push(val);
                                new_combination
                            })
                        })
                        .collect()
                })
            }
            Space::OneOf { spaces } => {
                spaces
                    .par_iter()
                    .enumerate()
                    .flat_map(|(idx, space)| {
                        let sub_results = space.enumerate(); // Get the combinations for each subspace
                        sub_results.into_par_iter().map(move |mut sample| {
                            let mut with_index = vec![idx as i32];
                            with_index.append(&mut sample);
                            with_index
                        })
                    })
                    .collect()
            }
            _ => panic!("Cannot call enumerate on vector space"),
        }
    }

    pub fn enumerate_nested(&self) -> Vec<Vec<Vec<i32>>> {
        match self {
            Space::Vector { spaces } => spaces.par_iter().map(|space| space.enumerate()).collect(),
            _ => panic!("Cannot call enumerate_nested on non-vector space"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_discrete_space_sample() {
        let space = Space::new_discrete(5, 10);

        // Sample without a fixed seed
        let sample = space.sample();
        assert!(sample[0] >= 10 && sample[0] < 15);

        // Sample with a fixed seed
        let seed = 42;
        let sample_with_seed = space.sample_with_seed(seed);
        assert!(sample_with_seed[0] >= 10 && sample_with_seed[0] < 15);

        // Consistency check: repeat sampling with the same seed
        let repeated_sample = space.sample_with_seed(seed);
        assert_eq!(sample_with_seed, repeated_sample);
    }

    #[test]
    fn test_box_space_sample() {
        let space = Space::new_box(vec![0, 0, 0, 0], vec![1, 2, 3, 4]);

        // Sample without a fixed seed
        let sample = space.sample();
        assert!(sample[0] > 0 || sample[0] <= 1);
        assert!(sample[1] > 0 || sample[1] <= 2);
        assert!(sample[2] > 0 || sample[2] <= 3);
        assert!(sample[3] > 0 || sample[3] <= 4);

        // Sample with a fixed seed
        let seed = 42;
        let sample_with_seed = space.sample_with_seed(seed);
        assert!(sample_with_seed[0] > 0 || sample_with_seed[0] <= 1);
        assert!(sample_with_seed[1] > 0 || sample_with_seed[1] <= 2);
        assert!(sample_with_seed[2] > 0 || sample_with_seed[2] <= 3);
        assert!(sample_with_seed[3] > 0 || sample_with_seed[3] <= 4);

        // Consistency check: repeat sampling with the same seed
        let repeated_sample = space.sample_with_seed(seed);
        assert_eq!(sample_with_seed, repeated_sample);
    }

    #[test]
    fn test_oneof_space_sample() {
        let space = Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);

        // Sample without a fixed seed
        let sample = space.sample();
        assert!(
            (sample[0] == 0 && sample[1] >= 5 && sample[1] < 8)
                || (sample[0] == 1 && sample[1] >= 10 && sample[1] < 12)
        );

        // Sample with a fixed seed
        let seed = 42;
        let sample_with_seed = space.sample_with_seed(seed);
        assert!(
            (sample_with_seed[0] == 0 && sample_with_seed[1] >= 5 && sample_with_seed[1] < 8)
                || (sample_with_seed[0] == 1 && sample_with_seed[1] >= 10 && sample_with_seed[1] < 12)
        );

        // Consistency check: repeat sampling with the same seed
        let repeated_sample = space.sample_with_seed(seed);
        assert_eq!(sample_with_seed, repeated_sample);
    }

    #[test]
    fn test_vector_space_sample_nested() {
        let space = Space::new_vector(vec![Space::new_discrete(5, 10), Space::new_discrete(2, 20)]);

        // Test nested sampling without a fixed seed
        let nested_sample = space.sample_nested();
        assert_eq!(nested_sample.len(), 2);
        assert!(nested_sample[0][0] >= 10 && nested_sample[0][0] < 15);
        assert!(nested_sample[1][0] >= 20 && nested_sample[1][0] < 22);

        // Test nested sampling with a fixed seed
        let seed = 42;
        let nested_sample_with_seed = space.sample_nested_with_seed(seed);
        assert_eq!(nested_sample_with_seed.len(), 2);
        assert!(nested_sample_with_seed[0][0] >= 10 && nested_sample_with_seed[0][0] < 15);
        assert!(nested_sample_with_seed[1][0] >= 20 && nested_sample_with_seed[1][0] < 22);

        // Consistency check: repeat nested sampling with the same seed
        let repeated_nested_sample = space.sample_nested_with_seed(seed);
        assert_eq!(nested_sample_with_seed, repeated_nested_sample);
    }

    #[test]
    #[should_panic(expected = "Cannot call sample_nested on non-vector space")]
    fn test_oneof_throws_with_nested_sample() {
        let space = Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);
        space.sample_nested();
    }

    #[test]
    #[should_panic(expected = "Cannot call sample_nested on non-vector space")]
    fn test_discrete_throws_with_nested_sample() {
        let space = Space::new_discrete(5, 10);
        space.sample_nested();
    }

    #[test]
    #[should_panic(expected = "Cannot call sample_nested on non-vector space")]
    fn test_box_throws_with_nested_sample() {
        let space = Space::new_box(vec![0, 0], vec![1, 1]);
        space.sample_nested();
    }

    #[test]
    #[should_panic(expected = "Cannot call sample on vector space")]
    fn test_vector_throws_with_sample() {
        let space = Space::new_vector(vec![
            Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]),
            Space::new_discrete(5, 15),
        ]);
        space.sample();
    }

    #[test]
    fn test_discrete_space_enumerate() {
        let space = Space::new_discrete(5, 10);
        let enumerated = space.enumerate();

        assert_eq!(enumerated.len(), 5);

        for (i, sample) in enumerated.iter().enumerate() {
            assert_eq!(sample, &[i as i32 + 10]);
        }
    }

    #[test]
    fn test_box_space_enumerate() {
        let space = Space::new_box(vec![0, 0, 0], vec![1, 2, 3]);

        let result = space.enumerate();

        println!("{:#?}", result);

        assert_eq!(result.len(), 24);

        let mut seen = HashSet::new();
        for sample in result.iter() {
            assert!(sample[0] >= 0 && sample[0] <= 1);
            assert!(sample[1] >= 0 && sample[1] <= 2);
            assert!(sample[2] >= 0 && sample[2] <= 3);

            assert!(seen.insert(sample.clone()), "Duplicate enumeration found: {:?}", sample)
        }
    }

    #[test]
    fn test_oneof_space_enumerate() {
        let space = Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);

        let result = space.enumerate();

        assert_eq!(result.len(), 5);

        let mut seen = HashSet::new();
        for sample in result.iter() {
            assert!(seen.insert(sample.clone()), "Duplicate enumeration found: {:?}", sample)
        }
    }

    #[test]
    fn test_vector_space_nested_enumerate() {
        let space = Space::new_vector(vec![Space::new_discrete(5, 10), Space::new_discrete(2, 20)]);

        let result = space.enumerate_nested();

        assert_eq!(result.len(), 2);

        let expected_first_space: Vec<Vec<i32>> = (10..15).map(|i| vec![i]).collect();
        let expected_second_space: Vec<Vec<i32>> = (20..22).map(|i| vec![i]).collect();

        assert_eq!(result[0], expected_first_space);
        assert_eq!(result[1], expected_second_space);
    }

    #[test]
    #[should_panic(expected = "Cannot call enumerate_nested on non-vector space")]
    fn test_discrete_throws_with_nested_enumerate() {
        let space = Space::new_discrete(5, 10);
        space.enumerate_nested();
    }

    #[test]
    #[should_panic(expected = "Cannot call enumerate_nested on non-vector space")]
    fn test_box_throws_with_nested_enumerate() {
        let space = Space::new_box(vec![0, 0, 0], vec![1, 2, 3]);
        space.enumerate_nested();
    }

    #[test]
    #[should_panic(expected = "Cannot call enumerate_nested on non-vector space")]
    fn test_oneof_throws_with_nested_enumerate() {
        let space = Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);
        space.enumerate_nested();
    }

    #[test]
    #[should_panic(expected = "Cannot call enumerate on vector space")]
    fn test_vector_throws_with_enumerate() {
        let space = Space::new_vector(vec![
            Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]),
            Space::new_discrete(5, 15),
        ]);
        space.enumerate();
    }
}
