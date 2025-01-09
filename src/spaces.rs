use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub enum Space {
    /// A discrete space with a range of values.
    Discrete { n: i32, start: i32 },

    /// A space that represents one of multiple possible sub-spaces.
    OneOf { spaces: Vec<Space> },

    /// A box space defined by lower and upper bounds.
    Box { low: Vec<i32>, high: Vec<i32> },

    /// A tuple space containing multiple sub-spaces.
    Tuple { spaces: Vec<Space> },

    /// A dictionary space containing multiple sub-spaces.
    Dict { spaces: HashMap<String, Space> },

    /// A vector space containing multiple sub-spaces.
    Vector { spaces: Vec<Space> },
}

impl Display for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, 0)
    }
}

impl Space {
    pub fn new_discrete(n: i32, start: i32) -> Self {
        Space::Discrete { n, start }
    }

    pub fn new_one_of(spaces: Vec<Space>) -> Self {
        Space::OneOf { spaces }
    }

    pub fn new_tuple(spaces: Vec<Space>) -> Self {
        Space::Tuple { spaces }
    }

    pub fn new_box(low: Vec<i32>, high: Vec<i32>) -> Self {
        Space::Box { low, high }
    }

    pub fn new_dict(spaces: HashMap<String, Space>) -> Self {
        Space::Dict { spaces }
    }

    pub fn new_vector(spaces: Vec<Space>) -> Self {
        Space::Vector { spaces }
    }

    // get the length of the space
    pub fn len(&self) -> usize {
        match self {
            Space::Discrete { n: _, start: _ } => panic!("Cannot call len on Discrete space"),
            Space::Box { low: _, high: _ } => panic!("Cannot call len on Box space"),
            Space::Tuple { spaces } | Space::OneOf { spaces } | Space::Vector { spaces } => spaces.len(),
            Space::Dict { spaces } => spaces.len(),
        }
    }

    ///  Sample a single value from the space.
    pub fn sample(&self) -> Sample {
        let mut rng = StdRng::from_entropy();

        let result = match self {
            Space::Discrete { n, start } => {
                if *n == 0 {
                    panic!("Cannot sample from empty discrete space")
                }

                Sample::Discrete(rng.gen_range(*start..(*start + *n)))
            }
            Space::Box { low, high } => {
                Sample::Box(low.iter().zip(high.iter()).map(|(l, h)| rng.gen_range(*l..=*h)).collect())
            }
            Space::OneOf { spaces } => {
                let valid_spaces: Vec<_> = spaces
                    .iter()
                    .enumerate()
                    .filter(|(_, space)| match space {
                        Space::Discrete { n, .. } => *n > 0,
                        _ => true,
                    })
                    .collect();

                if valid_spaces.is_empty() {
                    panic!("Cannot sample from empty OneOf space")
                }

                let (index, sub_space) = valid_spaces[rng.gen_range(0..valid_spaces.len())];
                Sample::OneOf(index as i32, Box::new(sub_space.sample()))
            }
            Space::Tuple { spaces } => Sample::Tuple(spaces.iter().map(|space| space.sample()).collect()),
            Space::Dict { spaces } => {
                Sample::Dict(spaces.iter().map(|(key, space)| (key.clone(), space.sample())).collect())
            }
            _ => panic!("Cannot call sample on vector space"),
        };

        result
    }

    ///  Sample a single value from the space with a fixed seed.
    pub fn sample_with_seed(&self, seed: u64) -> Sample {
        let mut rng = StdRng::seed_from_u64(seed);

        let result = match self {
            Space::Discrete { n, start } => {
                if *n == 0 {
                    panic!("Cannot sample from empty discrete space")
                }

                Sample::Discrete(rng.gen_range(*start..(*start + *n)))
            }
            Space::Box { low, high } => {
                Sample::Box(low.iter().zip(high.iter()).map(|(l, h)| rng.gen_range(*l..=*h)).collect())
            }
            Space::OneOf { spaces } => {
                let valid_spaces: Vec<_> = spaces
                    .iter()
                    .enumerate()
                    .filter(|(_, space)| match space {
                        Space::Discrete { n, .. } => *n > 0,
                        _ => true,
                    })
                    .collect();

                if valid_spaces.is_empty() {
                    panic!("Cannot sample from empty OneOf space")
                }

                let (index, sub_space) = valid_spaces[rng.gen_range(0..valid_spaces.len())];
                Sample::OneOf(index as i32, Box::new(sub_space.sample_with_seed(seed + 1)))
            }
            Space::Tuple { spaces } => Sample::Tuple(
                spaces.iter().enumerate().map(|(index, space)| space.sample_with_seed(seed + index as u64)).collect(),
            ),
            Space::Dict { spaces } => Sample::Dict(
                spaces
                    .iter()
                    .enumerate()
                    .map(|(index, (key, space))| (key.clone(), space.sample_with_seed(seed + index as u64)))
                    .collect(),
            ),
            _ => panic!("Cannot call sample on vector space"),
        };

        result
    }

    /// Sample a single value from each of the nested spaces.
    pub fn sample_nested(&self) -> Vec<Sample> {
        match self {
            Space::Vector { spaces } => spaces.iter().map(|space| space.sample()).collect(),
            _ => panic!("Cannot call sample_nested on non-vector space"),
        }
    }

    /// Sample a single value from each of the nested spaces with a fixed seed.
    pub fn sample_nested_with_seed(&self, seed: u64) -> Vec<Sample> {
        match self {
            Space::Vector { spaces } => {
                spaces.iter().enumerate().map(|(index, space)| space.sample_with_seed(seed + index as u64)).collect()
            }
            _ => panic!("Cannot call sample_nested on non-vector space"),
        }
    }

    /// Enumerate all possible values in the space.
    pub fn enumerate(&self) -> Vec<Sample> {
        match self {
            Space::Discrete { n, start } => (0..*n).map(|i| Sample::Discrete(i + *start)).collect(),
            Space::Box { low, high } => low
                .iter()
                .zip(high.iter())
                .fold(vec![vec![]], |acc, (l, h)| {
                    let range = (*l..=*h).collect::<Vec<i32>>();

                    acc.into_iter()
                        .flat_map(|sample| {
                            range.iter().map(move |i| {
                                let mut new_sample = sample.clone();
                                new_sample.push(*i);
                                new_sample
                            })
                        })
                        .collect()
                })
                .into_iter()
                .map(Sample::Box)
                .collect(),
            Space::OneOf { spaces } => spaces
                .iter()
                .enumerate()
                .flat_map(|(idx, space)| {
                    let sub_results = space.enumerate();
                    sub_results.into_iter().map(move |sample| Sample::OneOf(idx as i32, Box::new(sample)))
                })
                .collect(),
            Space::Tuple { spaces } => spaces
                .iter()
                .fold(vec![vec![]], |acc, space| {
                    let sub_results = space.enumerate();
                    acc.into_iter()
                        .flat_map(|prefix| {
                            sub_results.iter().map(move |sample| {
                                let mut new_tuple = prefix.clone();
                                new_tuple.push(sample.clone());
                                new_tuple
                            })
                        })
                        .collect()
                })
                .into_iter()
                .map(Sample::Tuple)
                .collect(),
            Space::Dict { spaces } => {
                let keys: Vec<_> = spaces.keys().cloned().collect();
                let enumerations: Vec<_> = keys.iter().map(|key| spaces[key].enumerate()).collect();

                enumerations
                    .iter()
                    .fold(vec![HashMap::new()], |acc, sub_enumeration| {
                        acc.into_iter()
                            .flat_map(|partial_dict| {
                                // Capture a reference to `keys`
                                let keys_ref = &keys;

                                sub_enumeration.iter().map(move |sample| {
                                    let mut new_dict = partial_dict.clone();
                                    let key = &keys_ref[partial_dict.len()]; // Use partial_dict.len() for the index
                                    new_dict.insert(key.clone(), sample.clone());
                                    new_dict
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                    .into_iter()
                    .map(Sample::Dict)
                    .collect()
            }
            _ => panic!("Cannot call enumerate on vector space"),
        }
    }

    /// Enumerate all possible values in the nested spaces.
    pub fn enumerate_nested(&self) -> Vec<Vec<Sample>> {
        match self {
            Space::Vector { spaces } => spaces.iter().map(|space| space.enumerate()).collect(),
            _ => panic!("Cannot call enumerate_nested on non-vector space"),
        }
    }

    /// Format the space as a string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
        let indent = "\t".repeat(level);

        match self {
            Space::Discrete { .. } => write!(f, "{}{:?}", indent, self),
            Space::OneOf { spaces } => {
                writeln!(f, "{}OneOf {{ spaces=[", indent)?;
                for space in spaces {
                    space.fmt(f, level + 1)?;
                    writeln!(f)?;
                }
                write!(f, "{}])", indent)
            }
            Space::Box { .. } => write!(f, "{}{:?}", indent, self),
            Space::Tuple { spaces } => {
                write!(f, "{}Tuple {{ spaces=[", indent)?;
                for space in spaces {
                    space.fmt(f, level + 1)?;
                    writeln!(f)?;
                }
                write!(f, "{}]}}", indent)
            }
            Space::Dict { spaces } => {
                write!(f, "{}Dict {{ spaces={{", indent)?;
                for (key, space) in spaces {
                    write!(f, "{}{}: ", indent, key)?;
                    space.fmt(f, level + 1)?;
                }
                write!(f, "{}}}}}", indent)
            }
            Space::Vector { spaces } => {
                writeln!(f, "{}Vector {{ spaces=[", indent)?;
                for space in spaces {
                    space.fmt(f, level + 1)?;
                    writeln!(f)?;
                }
                write!(f, "{}]}}", indent)
            }
        }
    }
}

#[pymethods]
impl Space {
    fn __repr__(&self) -> String {
        format!("{}", self)
    }

    fn __str__(&self) -> String {
        format!("{}", self)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

    fn __len__(&self) -> usize {
        self.len()
    }

    #[pyo3(name = "sample")]
    fn py_sample(&self) -> PyObject {
        Python::with_gil(|py| Sample::into_py(self.sample(), py))
    }

    #[pyo3(name = "sample_with_seed")]
    fn py_sample_with_seed(&self, seed: u64) -> PyObject {
        Python::with_gil(|py| Sample::into_py(self.sample_with_seed(seed), py))
    }

    #[pyo3(name = "sample_nested")]
    fn py_sample_nested(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| Sample::to_python_nested(&self.sample_nested(), py))
    }

    #[pyo3(name = "sample_nested_with_seed")]
    fn py_sample_nested_with_seed(&self, seed: u64) -> PyResult<PyObject> {
        Python::with_gil(|py| Sample::to_python_nested(&self.sample_nested_with_seed(seed), py))
    }

    #[pyo3(name = "enumerate")]
    fn py_enumerate(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| Sample::to_python_nested(&self.enumerate(), py))
    }

    #[pyo3(name = "enumerate_nested")]
    fn py_enumerate_nested(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| Sample::to_python_nested_nested(&self.enumerate_nested(), py))
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
                box_sample.hash(state); // Hash the inner sample
            }
            Sample::Box(vec) => {
                // Hash each element in the Vec
                vec.hash(state);
            }
            Sample::Tuple(vec) => {
                // Hash each element in the tuple
                vec.hash(state);
            }
            Sample::Dict(map) => {
                // Hash each key-value pair in the HashMap
                let mut sorted_entries: Vec<_> = map.iter().collect();
                sorted_entries.sort_by(|a, b| a.0.cmp(b.0));
                for (key, value) in sorted_entries {
                    key.hash(state);
                    value.hash(state); // Hash the value (which is a `Sample`)
                }
            }
        }
    }
}

impl IntoPy<PyObject> for Sample {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            Sample::Discrete(val) => val.into_py(py),
            Sample::Box(values) => {
                let py_list = PyList::new_bound(py, values);
                py_list.into()
            }
            Sample::OneOf(index, sample) => {
                let py_list = PyList::new_bound(py, &[index.into_py(py), sample.into_py(py)]);
                py_list.into()
            }
            Sample::Tuple(samples) => {
                let py_list = PyList::new_bound(py, samples.into_iter().map(|s| s.into_py(py)).collect::<Vec<_>>());
                py_list.into()
            }
            Sample::Dict(map) => {
                let py_dict = PyDict::new_bound(py);
                for (key, value) in map {
                    py_dict.set_item(key.into_py(py), value.into_py(py)).expect("Failed to set item");
                }
                py_dict.into()
            }
        }
    }
}

impl Sample {
    fn to_python(sample: Sample, py: Python<'_>) -> PyObject {
        match sample {
            Sample::Discrete(val) => val.into_py(py),
            Sample::Box(values) => {
                let py_list = PyList::new_bound(py, values);
                py_list.into()
            }
            Sample::OneOf(index, sample) => {
                let py_list = PyList::new_bound(py, &[index.into_py(py), sample.into_py(py)]);
                py_list.into()
            }
            Sample::Tuple(samples) => {
                let py_list = PyList::new_bound(py, samples.into_iter().map(|s| s.into_py(py)).collect::<Vec<_>>());
                py_list.into()
            }
            Sample::Dict(map) => {
                let py_dict = PyDict::new_bound(py);
                for (key, value) in map {
                    py_dict.set_item(key.into_py(py), value.into_py(py)).expect("Failed to set item");
                }
                py_dict.into()
            }
        }
    }

    fn to_python_nested(nested_sample: &Vec<Sample>, py: Python<'_>) -> PyResult<PyObject> {
        let py_list: Vec<_> = nested_sample.iter().map(|s| Sample::to_python(s.clone(), py)).collect();
        Ok(PyList::new_bound(py, py_list).into())
    }

    fn to_python_nested_nested(nested_sample: &Vec<Vec<Sample>>, py: Python<'_>) -> PyResult<PyObject> {
        let py_list =
            nested_sample.iter().map(|s| Self::to_python_nested(s, py)).collect::<Result<Vec<PyObject>, _>>()?;
        Ok(PyList::new_bound(py, py_list).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paste::paste;
    use std::collections::HashSet;

    macro_rules! panic_test_space {
        ($name:ident, $init:expr, $( $method:ident ),+) => {
            $(
                paste! {
                    #[test]
                    #[should_panic(expected = "Cannot call " $method " on non-vector space")]
                    fn [<test_ $name _throws_with_ $method>]() {
                        $init.$method();
                    }
                }
            )+
        };

        (nested $name:ident, $init:expr, $( $method:ident ),+) => {
            $(
                paste! {
                    #[test]
                    #[should_panic(expected = "Cannot call " $method " on vector space")]
                    fn [<test_ $name _throws_with_ $method>]() {
                        $init.$method();
                    }
                }
            )+
        };

        ($name:ident, $message:expr, $init:expr, $method:ident) => {
            paste! {
                #[test]
                #[should_panic(expected = $message)]
                fn [<test_ $name _throws_with_ $method>]() {
                    $init.$method();
                }
            }
        };
    }

    #[test]
    fn test_oneof_space_len() {
        let space = Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);
        assert_eq!(space.len(), 2);
    }

    #[test]
    fn test_dict_space_len() {
        let space = Space::new_dict(
            vec![("a".to_string(), Space::new_discrete(3, 5)), ("b".to_string(), Space::new_discrete(2, 10))]
                .into_iter()
                .collect(),
        );

        assert_eq!(space.len(), 2);
    }

    #[test]
    fn test_tuple_space_len() {
        let space = Space::new_tuple(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);

        assert_eq!(space.len(), 2);
    }

    #[test]
    fn test_vector_space_len() {
        let space = Space::new_vector(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);

        assert_eq!(space.len(), 2);
    }

    #[test]
    fn test_discrete_space_sample() {
        let space = Space::new_discrete(5, 10);

        // Sample without a fixed seed
        let Sample::Discrete(sample) = space.sample() else {
            panic!("Sample is not of type Sample::Discrete");
        };

        assert!(sample >= 10 && sample < 15);

        // Sample with a fixed seed
        let seed = 42;
        let Sample::Discrete(sample_with_seed) = space.sample_with_seed(seed) else {
            panic!("Sample is not of type Sample::Discrete");
        };
        assert!(sample_with_seed >= 10 && sample_with_seed < 15);

        // Consistency check: repeat sampling with the same seed
        let Sample::Discrete(repeated_sample) = space.sample_with_seed(seed) else {
            panic!("Sample is not of type Sample::Discrete");
        };
        assert_eq!(sample_with_seed, repeated_sample);
    }

    #[test]
    fn test_box_space_sample() {
        let space = Space::new_box(vec![0, 0, 0, 0], vec![1, 2, 3, 4]);

        // Sample without a fixed seed
        let Sample::Box(sample) = space.sample() else {
            panic!("Sample is not of type Sample::Box");
        };
        assert!(sample[0] > 0 || sample[0] <= 1);
        assert!(sample[1] > 0 || sample[1] <= 2);
        assert!(sample[2] > 0 || sample[2] <= 3);
        assert!(sample[3] > 0 || sample[3] <= 4);

        // Sample with a fixed seed
        let seed = 42;
        let Sample::Box(sample_with_seed) = space.sample_with_seed(seed) else {
            panic!("Sample is not of type Sample::Box");
        };
        assert!(sample_with_seed[0] > 0 || sample_with_seed[0] <= 1);
        assert!(sample_with_seed[1] > 0 || sample_with_seed[1] <= 2);
        assert!(sample_with_seed[2] > 0 || sample_with_seed[2] <= 3);
        assert!(sample_with_seed[3] > 0 || sample_with_seed[3] <= 4);

        // Consistency check: repeat sampling with the same seed
        let Sample::Box(repeated_sample) = space.sample_with_seed(seed) else {
            panic!("Sample is not of type Sample::Box");
        };
        assert_eq!(sample_with_seed, repeated_sample);
    }

    #[test]
    fn test_oneof_space_sample() {
        let space = Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);

        // Sample without a fixed seed
        let Sample::OneOf(index, sample) = space.sample() else {
            panic!("Sample is not of type Sample::OneOf");
        };
        let Sample::Discrete(sample) = *sample else {
            panic!("Inner sample is not of type Sample::Discrete");
        };
        assert!((index == 0 && sample >= 5 && sample < 8) || (index == 1 && sample >= 10 && sample < 12));

        // Sample with a fixed seed
        let seed = 42;
        let Sample::OneOf(index, sample_with_seed) = space.sample_with_seed(seed) else {
            panic!("Sample is not of type Sample::OneOf");
        };
        let Sample::Discrete(sample_with_seed) = *sample_with_seed else {
            panic!("Inner sample is not of type Sample::Discrete");
        };

        assert!(
            (index == 0 && sample_with_seed >= 5 && sample_with_seed < 8)
                || (index == 1 && sample_with_seed >= 10 && sample_with_seed < 12)
        );

        // Consistency check: repeat sampling with the same seed
        let Sample::OneOf(repeated_index, repeated_sample) = space.sample_with_seed(seed) else {
            panic!("Sample is not of type Sample::OneOf");
        };
        let Sample::Discrete(repeated_sample) = *repeated_sample else {
            panic!("Inner sample is not of type Sample::Discrete");
        };

        assert_eq!(index, repeated_index);
        assert_eq!(sample_with_seed, repeated_sample);
    }

    #[test]
    fn test_dict_space_sample() {
        let space = Space::new_dict(
            vec![("first".to_string(), Space::new_discrete(3, 5)), ("second".to_string(), Space::new_discrete(2, 10))]
                .into_iter()
                .collect(),
        );

        // Sample without a fixed seed
        let Sample::Dict(sample) = space.sample() else {
            panic!("Sample is not of type Sample::Dict");
        };

        let first_sample = sample.get("first").unwrap();
        let second_sample = sample.get("second").unwrap();
        let Sample::Discrete(first_sample) = first_sample else {
            panic!("First sample is not of type Sample::Discrete");
        };
        assert!(*first_sample >= 5 && *first_sample < 8);

        let Sample::Discrete(second_sample) = second_sample else {
            panic!("Second sample is not of type Sample::Discrete");
        };
        assert!(*second_sample >= 10 && *second_sample < 12);

        // Sample with a fixed seed
        let seed = 42;
        let Sample::Dict(sample_with_seed) = space.sample_with_seed(seed) else {
            panic!("Sample is not of type Sample::Dict");
        };

        let Sample::Dict(repeated_sample_with_seed) = space.sample_with_seed(seed) else {
            panic!("Sample is not of type Sample::Dict");
        };

        assert_eq!(sample_with_seed, repeated_sample_with_seed);
    }

    #[test]
    fn test_tuple_space_sample_nested() {
        let space = Space::new_tuple(vec![Space::new_discrete(5, 10), Space::new_discrete(2, 20)]);

        // Test sampling without a fixed seed
        let Sample::Tuple(sample) = space.sample() else {
            panic!("Sample is not of type Sample::Tuple");
        };

        let Sample::Discrete(first_sample) = sample[0] else {
            panic!("First sample is not of type Sample::Discrete");
        };

        let Sample::Discrete(second_sample) = sample[1] else {
            panic!("Second sample is not of type Sample::Discrete");
        };

        assert!(first_sample >= 10 && first_sample < 15);
        assert!(second_sample >= 20 && second_sample < 22);

        // Test nested sampling with a fixed seed
        let seed = 42;
        let sample_with_seed = space.sample_with_seed(seed);
        let Sample::Tuple(sample_with_seed) = sample_with_seed else {
            panic!("Sample is not of type Sample::Tuple");
        };

        // Consistency check: repeat sampling with the same seed
        let Sample::Tuple(repeated_sample) = space.sample_with_seed(seed) else {
            panic!("Sample is not of type Sample::Tuple");
        };

        assert_eq!(sample_with_seed, repeated_sample);
    }

    #[test]
    fn test_vector_space_sample_nested() {
        let space = Space::new_vector(vec![Space::new_discrete(5, 10), Space::new_discrete(2, 20)]);

        // Test nested sampling without a fixed seed
        let nested_sample = space.sample_nested();
        assert_eq!(nested_sample.len(), 2);

        let Sample::Discrete(first_sample) = nested_sample[0] else {
            panic!("First sample is not of type Sample::Discrete");
        };

        let Sample::Discrete(second_sample) = nested_sample[1] else {
            panic!("Second sample is not of type Sample::Discrete");
        };

        assert!(first_sample >= 10 && first_sample < 15);
        assert!(second_sample >= 20 && second_sample < 22);

        // Test nested sampling with a fixed seed
        let seed = 42;
        let nested_sample_with_seed = space.sample_nested_with_seed(seed);
        assert_eq!(nested_sample_with_seed.len(), 2);

        let repeated_nested_sample = space.sample_nested_with_seed(seed);

        assert_eq!(nested_sample_with_seed, repeated_nested_sample);
    }

    #[test]
    fn test_discrete_space_enumerate() {
        let space = Space::new_discrete(5, 10);
        let enumerated = space.enumerate();

        assert_eq!(enumerated.len(), 5);

        for (i, sample) in enumerated.iter().enumerate() {
            assert_eq!(sample, &Sample::Discrete(i as i32 + 10));
        }
    }

    #[test]
    fn test_box_space_enumerate() {
        let space = Space::new_box(vec![0, 0, 0], vec![1, 2, 3]);

        let result = space.enumerate();

        assert_eq!(result.len(), 24);

        let mut seen = HashSet::new();
        for sample in result.iter() {
            let Sample::Box(sample) = sample else { panic!("Sample is not of type Sample::Box") };

            assert!(sample[0] >= 0 && sample[0] <= 1);
            assert!(sample[1] >= 0 && sample[1] <= 2);
            assert!(sample[2] >= 0 && sample[2] <= 3);

            assert!(seen.insert(sample.clone()), "Duplicate enumeration found: {:?}", sample)
        }
    }

    #[test]
    fn test_dict_space_enumerate() {
        let space = Space::new_dict(
            vec![("first".to_string(), Space::new_discrete(3, 5)), ("second".to_string(), Space::new_discrete(2, 10))]
                .into_iter()
                .collect(),
        );

        let result = space.enumerate();

        assert_eq!(result.len(), 6);

        let mut seen = HashSet::new();

        for sample in result.iter() {
            let Sample::Dict(sample) = sample else { panic!("Sample is not of type Sample::Dict") };

            let first_sample = sample.get("first").unwrap();
            let second_sample = sample.get("second").unwrap();

            let Sample::Discrete(first_sample) = first_sample else {
                panic!("First sample is not of type Sample::Discrete")
            };

            let Sample::Discrete(second_sample) = second_sample else {
                panic!("Second sample is not of type Sample::Discrete")
            };

            assert!(*first_sample >= 5 && *first_sample < 8);
            assert!(*second_sample >= 10 && *second_sample < 12);

            assert!(seen.insert(Sample::Dict(sample.clone())), "Duplicate enumeration found: {:?}", sample)
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

        let expected_first_space: Vec<Sample> = (10..15).map(|i| Sample::Discrete(i)).collect();
        let expected_second_space: Vec<Sample> = (20..22).map(|i| Sample::Discrete(i)).collect();

        assert_eq!(result[0], expected_first_space);
        assert_eq!(result[1], expected_second_space);
    }

    panic_test_space!(discrete, Space::new_discrete(5, 10), sample_nested, enumerate_nested);
    panic_test_space!(discrete, "Cannot call len on Discrete space", Space::new_discrete(5, 10), len);

    panic_test_space!(box, Space::new_box(vec![0, 0, 0], vec![1, 2, 3]), sample_nested, enumerate_nested);
    panic_test_space!(box, "Cannot call len on Box space", Space::new_box(vec![0, 0, 0], vec![1, 2, 3]), len);

    panic_test_space!(tuple, Space::new_tuple(vec![Space::new_discrete(5, 10)]), sample_nested, enumerate_nested);

    panic_test_space!(dict, Space::new_dict(HashMap::new()), sample_nested, enumerate_nested);

    panic_test_space!(oneof, Space::new_one_of(vec![Space::new_discrete(3, 5)]), sample_nested, enumerate_nested);

    panic_test_space!(nested vector, Space::new_vector(vec![Space::new_discrete(5, 10)]), sample, enumerate);
}
