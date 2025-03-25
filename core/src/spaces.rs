use std::collections::HashMap;
use std::sync::Arc;

pub trait Space: Send + Sync {
    fn len(&self) -> usize;
    fn sample(&self) -> Arc<dyn Sample>;
    fn sample_with_seed(&self, seed: u64) -> Arc<dyn Sample>;
    fn enumerate(&self) -> Vec<Arc<dyn Sample>>;
}

use std::any::Any;

pub trait Sample: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_tuple(&self) -> Option<&[Arc<dyn Sample>]> {
        None
    }
    fn as_dict(&self) -> Option<&HashMap<String, Arc<dyn Sample>>> {
        None
    }
    fn as_vector(&self) -> Option<&[Arc<dyn Sample>]> {
        None
    }
    fn as_discrete(&self) -> Option<&DiscreteSample> {
        None
    }
    fn as_box(&self) -> Option<&BoxSample> {
        None
    }
    fn as_one_of(&self) -> Option<&OneOfSample> {
        None
    }
}

// Discrete
pub struct Discrete {
    pub n: i32,
    pub start: i32,
}
pub struct DiscreteSample(pub i32);
impl Sample for DiscreteSample {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_discrete(&self) -> Option<&DiscreteSample> {
        Some(self)
    }
}
impl Space for Discrete {
    fn len(&self) -> usize {
        self.n as usize
    }
    fn sample(&self) -> Arc<dyn Sample> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::from_entropy();
        Arc::new(DiscreteSample(
            rng.gen_range(self.start..self.start + self.n),
        ))
    }
    fn sample_with_seed(&self, seed: u64) -> Arc<dyn Sample> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        Arc::new(DiscreteSample(
            rng.gen_range(self.start..self.start + self.n),
        ))
    }
    fn enumerate(&self) -> Vec<Arc<dyn Sample>> {
        (0..self.n)
            .map(|i| Arc::new(DiscreteSample(self.start + i)) as Arc<dyn Sample>)
            .collect()
    }
}

// OneOf
pub struct OneOf {
    pub spaces: Vec<Arc<dyn Space>>,
}
pub struct OneOfSample(pub usize, pub Arc<dyn Sample>);
impl Sample for OneOfSample {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_one_of(&self) -> Option<&OneOfSample> {
        Some(self)
    }
}
impl Space for OneOf {
    fn len(&self) -> usize {
        self.spaces.len()
    }
    fn sample(&self) -> Arc<dyn Sample> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::from_entropy();
        let idx = rng.gen_range(0..self.spaces.len());
        Arc::new(OneOfSample(idx, self.spaces[idx].sample()))
    }
    fn sample_with_seed(&self, seed: u64) -> Arc<dyn Sample> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let idx = rng.gen_range(0..self.spaces.len());
        Arc::new(OneOfSample(
            idx,
            self.spaces[idx].sample_with_seed(seed + 1),
        ))
    }
    fn enumerate(&self) -> Vec<Arc<dyn Sample>> {
        self.spaces
            .iter()
            .enumerate()
            .flat_map(|(i, s)| {
                s.enumerate()
                    .into_iter()
                    .map(move |sample| Arc::new(OneOfSample(i, sample)) as Arc<dyn Sample>)
            })
            .collect()
    }
}

pub struct Box {
    pub low: Vec<i32>,
    pub high: Vec<i32>,
}

pub struct BoxSample(pub Vec<i32>);

impl Space for Box {
    fn len(&self) -> usize {
        self.low.len()
    }
    fn sample(&self) -> Arc<dyn Sample> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::from_entropy();
        let v = self
            .low
            .iter()
            .zip(self.high.iter())
            .map(|(l, h)| rng.gen_range(*l..=*h))
            .collect();
        Arc::new(BoxSample(v))
    }
    fn sample_with_seed(&self, seed: u64) -> Arc<dyn Sample> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let v = self
            .low
            .iter()
            .zip(self.high.iter())
            .map(|(l, h)| rng.gen_range(*l..=*h))
            .collect();
        Arc::new(BoxSample(v))
    }
    fn enumerate(&self) -> Vec<Arc<dyn Sample>> {
        fn enumerate_rec(bounds: &[(i32, i32)], prefix: Vec<i32>, acc: &mut Vec<Vec<i32>>) {
            if bounds.is_empty() {
                acc.push(prefix);
                return;
            }
            let (l, h) = bounds[0];
            for v in l..=h {
                let mut next = prefix.clone();
                next.push(v);
                enumerate_rec(&bounds[1..], next, acc);
            }
        }

        let mut acc = Vec::new();
        enumerate_rec(
            &self
                .low
                .iter()
                .zip(&self.high)
                .map(|(l, h)| (*l, *h))
                .collect::<Vec<_>>(),
            vec![],
            &mut acc,
        );

        acc.into_iter()
            .map(|v| Arc::new(BoxSample(v)) as Arc<dyn Sample>)
            .collect()
    }
}

impl Sample for BoxSample {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_box(&self) -> Option<&BoxSample> {
        Some(self)
    }
}

// TupleSpace
pub struct TupleSpace {
    pub spaces: Vec<Arc<dyn Space>>,
}
pub struct TupleSample(pub Vec<Arc<dyn Sample>>);

impl TupleSample {
    pub fn from_concrete<T: Sample + 'static>(v: Vec<Arc<T>>) -> Self {
        TupleSample(v.into_iter().map(|x| x as Arc<dyn Sample>).collect())
    }
}
impl VectorSample {
    pub fn from_concrete<T: Sample + 'static>(v: Vec<Arc<T>>) -> Self {
        VectorSample(v.into_iter().map(|x| x as Arc<dyn Sample>).collect())
    }
}
impl DictSample {
    pub fn from_concrete<T: Sample + 'static>(m: HashMap<String, Arc<T>>) -> Self {
        DictSample(
            m.into_iter()
                .map(|(k, v)| (k, v as Arc<dyn Sample>))
                .collect(),
        )
    }
}

impl Sample for TupleSample {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_tuple(&self) -> Option<&[Arc<dyn Sample>]> {
        Some(&self.0)
    }
}
impl Space for TupleSpace {
    fn len(&self) -> usize {
        self.spaces.len()
    }

    fn sample(&self) -> Arc<dyn Sample> {
        Arc::new(TupleSample(
            self.spaces.iter().map(|s| s.sample()).collect(),
        ))
    }

    fn sample_with_seed(&self, seed: u64) -> Arc<dyn Sample> {
        Arc::new(TupleSample(
            self.spaces
                .iter()
                .enumerate()
                .map(|(i, s)| s.sample_with_seed(seed + i as u64))
                .collect(),
        ))
    }

    fn enumerate(&self) -> Vec<Arc<dyn Sample>> {
        fn enumerate_rec(
            spaces: &[Arc<dyn Space>],
            prefix: Vec<Arc<dyn Sample>>,
            acc: &mut Vec<Vec<Arc<dyn Sample>>>,
        ) {
            if spaces.is_empty() {
                acc.push(prefix);
                return;
            }

            for s in spaces[0].enumerate() {
                let mut next = prefix.clone();
                next.push(s);
                enumerate_rec(&spaces[1..], next, acc);
            }
        }

        let mut acc = Vec::new();
        enumerate_rec(&self.spaces, vec![], &mut acc);
        acc.into_iter()
            .map(|v| Arc::new(TupleSample(v)) as Arc<dyn Sample>)
            .collect()
    }
}

// DictSpace
pub struct DictSpace {
    pub spaces: HashMap<String, Arc<dyn Space>>,
}

pub struct DictSample(pub HashMap<String, Arc<dyn Sample>>);

impl Sample for DictSample {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_dict(&self) -> Option<&HashMap<String, Arc<dyn Sample>>> {
        Some(&self.0)
    }
}

impl Space for DictSpace {
    fn len(&self) -> usize {
        self.spaces.len()
    }

    fn sample(&self) -> Arc<dyn Sample> {
        Arc::new(DictSample(
            self.spaces
                .iter()
                .map(|(k, s)| (k.clone(), s.sample()))
                .collect(),
        ))
    }

    fn sample_with_seed(&self, seed: u64) -> Arc<dyn Sample> {
        Arc::new(DictSample(
            self.spaces
                .iter()
                .enumerate()
                .map(|(i, (k, s))| (k.clone(), s.sample_with_seed(seed + i as u64)))
                .collect(),
        ))
    }

    fn enumerate(&self) -> Vec<Arc<dyn Sample>> {
        let keys: Vec<_> = self.spaces.keys().cloned().collect();
        let enums: Vec<_> = keys.iter().map(|k| self.spaces[k].enumerate()).collect();
        fn enumerate_rec(
            keys: &[String],
            enums: &[Vec<Arc<dyn Sample>>],
            prefix: HashMap<String, Arc<dyn Sample>>,
            acc: &mut Vec<HashMap<String, Arc<dyn Sample>>>,
        ) {
            if enums.is_empty() {
                acc.push(prefix);
                return;
            }
            for s in &enums[0] {
                let mut next = prefix.clone();
                next.insert(keys[0].clone(), s.clone());
                enumerate_rec(&keys[1..], &enums[1..], next, acc);
            }
        }
        let mut acc = Vec::new();
        enumerate_rec(&keys, &enums, HashMap::new(), &mut acc);
        acc.into_iter()
            .map(|m| Arc::new(DictSample(m)) as Arc<dyn Sample>)
            .collect()
    }
}

// VectorSpace
pub struct VectorSpace {
    pub spaces: Vec<Arc<dyn Space>>,
}

pub struct VectorSample(pub Vec<Arc<dyn Sample>>);
impl Sample for VectorSample {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_vector(&self) -> Option<&[Arc<dyn Sample>]> {
        Some(&self.0)
    }
}

impl Space for VectorSpace {
    fn len(&self) -> usize {
        self.spaces.len()
    }

    fn sample(&self) -> Arc<dyn Sample> {
        Arc::new(VectorSample(
            self.spaces.iter().map(|s| s.sample()).collect(),
        ))
    }
    fn sample_with_seed(&self, seed: u64) -> Arc<dyn Sample> {
        Arc::new(VectorSample(
            self.spaces
                .iter()
                .enumerate()
                .map(|(i, s)| s.sample_with_seed(seed + i as u64))
                .collect(),
        ))
    }
    fn enumerate(&self) -> Vec<Arc<dyn Sample>> {
        // Cartesian product of all subspaces, as VectorSample
        fn enumerate_rec(
            spaces: &[Arc<dyn Space>],
            prefix: Vec<Arc<dyn Sample>>,
            acc: &mut Vec<Vec<Arc<dyn Sample>>>,
        ) {
            if spaces.is_empty() {
                acc.push(prefix);
                return;
            }
            for s in spaces[0].enumerate() {
                let mut next = prefix.clone();
                next.push(s);
                enumerate_rec(&spaces[1..], next, acc);
            }
        }
        let mut acc = Vec::new();
        enumerate_rec(&self.spaces, vec![], &mut acc);
        acc.into_iter()
            .map(|v| Arc::new(VectorSample(v)) as Arc<dyn Sample>)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_discrete_len() {
        let space = Discrete { n: 5, start: 10 };

        assert_eq!(space.len(), 5);
    }

    #[test]
    fn test_discrete_sample() {
        let space = Discrete { n: 5, start: 10 };

        let sample = space.sample();
        let val = sample.as_discrete().unwrap().0;

        assert!((10..15).contains(&val));
    }

    #[test]
    fn test_discrete_sample_with_seed() {
        let space = Discrete { n: 5, start: 10 };
        let seed = 42;

        let sample_with_seed = space.sample_with_seed(seed);
        let val2 = sample_with_seed.as_discrete().unwrap().0;
        let repeated = space.sample_with_seed(seed);
        let val3 = repeated.as_discrete().unwrap().0;

        assert!((10..15).contains(&val2));
        assert_eq!(val2, val3);
    }

    #[test]
    fn test_discrete_enumerate() {
        let space = Discrete { n: 5, start: 10 };

        let all = space.enumerate();
        let vals: Vec<i32> = all.iter().map(|s| s.as_discrete().unwrap().0).collect();

        assert_eq!(vals, vec![10, 11, 12, 13, 14]);
    }

    #[test]
    fn test_boxspace_len() {
        let space = Box {
            low: vec![0, 0],
            high: vec![1, 2],
        };

        let len = space.len();

        assert_eq!(len, 2);
    }

    #[test]
    fn test_boxspace_sample() {
        let space = Box {
            low: vec![0, 0],
            high: vec![1, 2],
        };

        let sample = space.sample();
        let vals = &sample.as_box().unwrap().0;

        assert_eq!(vals.len(), 2);
        assert!(vals[0] >= 0 && vals[0] <= 1);
        assert!(vals[1] >= 0 && vals[1] <= 2);
    }

    #[test]
    fn test_boxspace_enumerate() {
        let space = Box {
            low: vec![0, 0],
            high: vec![1, 2],
        };
        let mut seen = std::collections::HashSet::new();

        for s in space.enumerate() {
            let v = &s.as_box().unwrap().0;
            assert!(v[0] >= 0 && v[0] <= 1);
            assert!(v[1] >= 0 && v[1] <= 2);
            assert!(seen.insert((v[0], v[1])));
        }

        assert_eq!(seen.len(), 6);
    }

    #[test]
    fn test_oneof_len() {
        let space = OneOf {
            spaces: vec![
                Arc::new(Discrete { n: 2, start: 5 }),
                Arc::new(Discrete { n: 3, start: 10 }),
            ],
        };

        assert_eq!(space.len(), 2);
    }

    #[test]
    fn test_oneof_sample() {
        let s1 = Arc::new(Discrete { n: 2, start: 5 });
        let s2 = Arc::new(Discrete { n: 3, start: 10 });
        let space = OneOf {
            spaces: vec![s1.clone(), s2.clone()],
        };

        let sample = space.sample();
        let (idx, inner) = {
            let o = sample.as_one_of().unwrap();
            (&o.0, &o.1)
        };

        assert!(*idx == 0 || *idx == 1);
        let _ = inner.as_discrete().unwrap();
    }

    #[test]
    fn test_oneof_enumerate() {
        let space = OneOf {
            spaces: vec![
                Arc::new(Discrete { n: 2, start: 5 }),
                Arc::new(Discrete { n: 3, start: 10 }),
            ],
        };

        let mut seen = std::collections::HashSet::new();
        for s in space.enumerate() {
            let o = s.as_one_of().unwrap();
            let idx = o.0;
            let val = o.1.as_discrete().unwrap().0;

            seen.insert((idx, val));
        }

        assert_eq!(seen.len(), 5);
    }

    #[test]
    fn test_tuple_len() {
        let space = TupleSpace {
            spaces: vec![
                Arc::new(Discrete { n: 2, start: 1 }),
                Arc::new(Discrete { n: 2, start: 10 }),
            ],
        };

        assert_eq!(space.len(), 2);
    }

    #[test]
    fn test_tuple_sample() {
        let space = TupleSpace {
            spaces: vec![
                Arc::new(Discrete { n: 2, start: 1 }),
                Arc::new(Discrete { n: 2, start: 10 }),
            ],
        };

        let sample = space.sample();
        let vals = sample.as_tuple().unwrap();

        assert_eq!(vals.len(), 2);
        assert!(vals[0].as_tuple().is_none());
        assert!(vals[0].as_dict().is_none());
        assert!(vals[0].as_vector().is_none());
        let _ = vals[0].as_discrete().unwrap();
        let _ = vals[1].as_discrete().unwrap();
    }

    #[test]
    fn test_tuple_enumerate() {
        let space = TupleSpace {
            spaces: vec![
                Arc::new(Discrete { n: 2, start: 1 }),
                Arc::new(Discrete { n: 2, start: 10 }),
            ],
        };

        let mut seen = std::collections::HashSet::new();
        for s in space.enumerate() {
            let v = s.as_tuple().unwrap();
            let a = v[0].as_discrete().unwrap().0;
            let b = v[1].as_discrete().unwrap().0;

            seen.insert((a, b));
        }

        assert_eq!(seen.len(), 4);
    }

    #[test]
    fn test_dict_len() {
        let mut map: HashMap<String, Arc<dyn Space>> = HashMap::new();
        map.insert("a".to_string(), Arc::new(Discrete { n: 2, start: 1 }));
        map.insert("b".to_string(), Arc::new(Discrete { n: 2, start: 10 }));

        let space = DictSpace { spaces: map };

        assert_eq!(space.len(), 2);
    }

    #[test]
    fn test_dict_sample() {
        let mut map: HashMap<String, Arc<dyn Space>> = HashMap::new();
        map.insert("a".to_string(), Arc::new(Discrete { n: 2, start: 1 }));

        let mut map: HashMap<String, Arc<dyn Space>> = HashMap::new();
        map.insert("a".to_string(), Arc::new(Discrete { n: 2, start: 1 }));
        map.insert("b".to_string(), Arc::new(Discrete { n: 2, start: 10 }));
        let space = DictSpace { spaces: map };

        let sample = space.sample();
        let vals = sample.as_dict().unwrap();

        assert_eq!(vals.len(), 2);
        let _ = vals["a"].as_discrete().unwrap();
        let _ = vals["b"].as_discrete().unwrap();
    }

    #[test]
    fn test_dict_enumerate() {
        let mut map: HashMap<String, Arc<dyn Space>> = HashMap::new();
        map.insert("a".to_string(), Arc::new(Discrete { n: 2, start: 1 }));
        map.insert("b".to_string(), Arc::new(Discrete { n: 2, start: 10 }));
        let space = DictSpace { spaces: map };

        let mut seen = std::collections::HashSet::new();
        for s in space.enumerate() {
            let m = s.as_dict().unwrap();
            let a = m["a"].as_discrete().unwrap().0;
            let b = m["b"].as_discrete().unwrap().0;

            seen.insert((a, b));
        }

        assert_eq!(seen.len(), 4);
    }

    #[test]
    fn test_vector_len() {
        let s1 = Arc::new(Discrete { n: 2, start: 1 });
        let s2 = Arc::new(Discrete { n: 2, start: 10 });
        let space = VectorSpace {
            spaces: vec![s1, s2],
        };

        assert_eq!(space.len(), 2);
    }

    #[test]
    fn test_vector_sample() {
        let space = VectorSpace {
            spaces: vec![
                Arc::new(Discrete { n: 2, start: 1 }),
                Arc::new(Discrete { n: 2, start: 10 }),
            ],
        };

        let sample = space.sample();
        let vals = sample.as_vector().unwrap();

        assert_eq!(vals.len(), 2);
        let _ = vals[0].as_ref().as_discrete().unwrap();
        let _ = vals[1].as_ref().as_discrete().unwrap();
    }

    #[test]
    fn test_as_tuple() {
        use std::sync::Arc;
        let tuple = TupleSample(vec![
            Arc::new(DiscreteSample(1)),
            Arc::new(DiscreteSample(2)),
        ]);
        let dict = DictSample(std::collections::HashMap::new());
        let vector = VectorSample(vec![]);
        let discrete = DiscreteSample(7);

        assert!(tuple.as_tuple().is_some());
        assert_eq!(tuple.as_tuple().unwrap().len(), 2);
        assert!(dict.as_tuple().is_none());
        assert!(vector.as_tuple().is_none());
        assert!(discrete.as_tuple().is_none());
    }

    #[test]
    fn test_as_dict() {
        use std::collections::HashMap;
        let dict = DictSample::from_concrete(HashMap::from([
            ("a".to_string(), std::sync::Arc::new(DiscreteSample(3))),
            ("b".to_string(), std::sync::Arc::new(DiscreteSample(4))),
        ]));
        let tuple = TupleSample(vec![]);
        let vector = VectorSample(vec![]);
        let discrete = DiscreteSample(7);

        assert!(dict.as_dict().is_some());
        assert_eq!(dict.as_dict().unwrap().len(), 2);
        assert!(tuple.as_dict().is_none());
        assert!(vector.as_dict().is_none());
        assert!(discrete.as_dict().is_none());
    }

    #[test]
    fn test_as_vector() {
        use std::sync::Arc;
        let vector = VectorSample::from_concrete(vec![
            Arc::new(DiscreteSample(5)),
            Arc::new(DiscreteSample(6)),
        ]);
        let tuple = TupleSample(vec![]);
        let dict = DictSample(std::collections::HashMap::new());
        let discrete = DiscreteSample(7);

        assert!(vector.as_vector().is_some());
        assert_eq!(vector.as_vector().unwrap().len(), 2);
        assert!(tuple.as_vector().is_none());
        assert!(dict.as_vector().is_none());
        assert!(discrete.as_vector().is_none());
    }

    #[test]
    fn test_vector_enumerate() {
        let s1 = Arc::new(Discrete { n: 2, start: 1 });
        let s2 = Arc::new(Discrete { n: 2, start: 10 });
        let space = VectorSpace {
            spaces: vec![s1, s2],
        };
        let mut seen = std::collections::HashSet::new();

        let all = space.enumerate();
        for s in all {
            let v = s.as_vector().unwrap();
            let a = v[0].as_discrete().unwrap().0;
            let b = v[1].as_discrete().unwrap().0;

            seen.insert((a, b));
        }

        assert_eq!(seen.len(), 4);
    }
}

//#[pyclass]
//#[derive(Debug, Clone, PartialEq)]
//pub enum Space {
//    /// A discrete space with a range of values.
//    Discrete { n: i32, start: i32 },
//
//    /// A space that represents one of multiple possible sub-spaces.
//    OneOf { spaces: Vec<Space> },
//
//    /// A box space defined by lower and upper bounds.
//    Box { low: Vec<i32>, high: Vec<i32> },
//
//    /// A tuple space containing multiple sub-spaces.
//    Tuple { spaces: Vec<Space> },
//
//    /// A dictionary space containing multiple sub-spaces.
//    Dict { spaces: HashMap<String, Space> },
//
//    /// A vector space containing multiple sub-spaces.
//    Vector { spaces: Vec<Space> },
//}
//
//impl Display for Space {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        self.fmt(f, 0)
//    }
//}
//
//impl Space {
//    pub fn new_discrete(n: i32, start: i32) -> Self {
//        Space::Discrete { n, start }
//    }
//
//    pub fn new_one_of(spaces: Vec<Space>) -> Self {
//        Space::OneOf { spaces }
//    }
//
//    pub fn new_tuple(spaces: Vec<Space>) -> Self {
//        Space::Tuple { spaces }
//    }
//
//    pub fn new_box(low: Vec<i32>, high: Vec<i32>) -> Self {
//        Space::Box { low, high }
//    }
//
//    pub fn new_dict(spaces: HashMap<String, Space>) -> Self {
//        Space::Dict { spaces }
//    }
//
//    pub fn new_vector(spaces: Vec<Space>) -> Self {
//        Space::Vector { spaces }
//    }
//
//    // get the length of the space
//    pub fn len(&self) -> usize {
//        match self {
//            Space::Discrete { n: _, start: _ } => panic!("Cannot call len on Discrete space"),
//            Space::Box { low: _, high: _ } => panic!("Cannot call len on Box space"),
//            Space::Tuple { spaces } | Space::OneOf { spaces } | Space::Vector { spaces } => spaces.len(),
//            Space::Dict { spaces } => spaces.len(),
//        }
//    }
//
//    ///  Sample a single value from the space.
//    pub fn sample(&self) -> Sample {
//        let mut rng = StdRng::from_entropy();
//
//        let result = match self {
//            Space::Discrete { n, start } => {
//                if *n == 0 {
//                    panic!("Cannot sample from empty discrete space")
//                }
//
//                Sample::Discrete(rng.gen_range(*start..(*start + *n)))
//            }
//            Space::Box { low, high } => {
//                Sample::Box(low.iter().zip(high.iter()).map(|(l, h)| rng.gen_range(*l..=*h)).collect())
//            }
//            Space::OneOf { spaces } => {
//                let valid_spaces: Vec<_> = spaces
//                    .iter()
//                    .enumerate()
//                    .filter(|(_, space)| match space {
//                        Space::Discrete { n, .. } => *n > 0,
//                        _ => true,
//                    })
//                    .collect();
//
//                if valid_spaces.is_empty() {
//                    panic!("Cannot sample from empty OneOf space")
//                }
//
//                let (index, sub_space) = valid_spaces[rng.gen_range(0..valid_spaces.len())];
//                Sample::OneOf(index as i32, Box::new(sub_space.sample()))
//            }
//            Space::Tuple { spaces } => Sample::Tuple(spaces.iter().map(|space| space.sample()).collect()),
//            Space::Dict { spaces } => {
//                Sample::Dict(spaces.iter().map(|(key, space)| (key.clone(), space.sample())).collect())
//            }
//            _ => panic!("Cannot call sample on vector space"),
//        };
//
//        result
//    }
//
//    ///  Sample a single value from the space with a fixed seed.
//    pub fn sample_with_seed(&self, seed: u64) -> Sample {
//        let mut rng = StdRng::seed_from_u64(seed);
//
//        let result = match self {
//            Space::Discrete { n, start } => {
//                if *n == 0 {
//                    panic!("Cannot sample from empty discrete space")
//                }
//
//                Sample::Discrete(rng.gen_range(*start..(*start + *n)))
//            }
//            Space::Box { low, high } => {
//                Sample::Box(low.iter().zip(high.iter()).map(|(l, h)| rng.gen_range(*l..=*h)).collect())
//            }
//            Space::OneOf { spaces } => {
//                let valid_spaces: Vec<_> = spaces
//                    .iter()
//                    .enumerate()
//                    .filter(|(_, space)| match space {
//                        Space::Discrete { n, .. } => *n > 0,
//                        _ => true,
//                    })
//                    .collect();
//
//                if valid_spaces.is_empty() {
//                    panic!("Cannot sample from empty OneOf space")
//                }
//
//                let (index, sub_space) = valid_spaces[rng.gen_range(0..valid_spaces.len())];
//                Sample::OneOf(index as i32, Box::new(sub_space.sample_with_seed(seed + 1)))
//            }
//            Space::Tuple { spaces } => Sample::Tuple(
//                spaces.iter().enumerate().map(|(index, space)| space.sample_with_seed(seed + index as u64)).collect(),
//            ),
//            Space::Dict { spaces } => Sample::Dict(
//                spaces
//                    .iter()
//                    .enumerate()
//                    .map(|(index, (key, space))| (key.clone(), space.sample_with_seed(seed + index as u64)))
//                    .collect(),
//            ),
//            _ => panic!("Cannot call sample on vector space"),
//        };
//
//        result
//    }
//
//    /// Sample a single value from each of the nested spaces.
//    pub fn sample_nested(&self) -> Vec<Sample> {
//        match self {
//            Space::Vector { spaces } => spaces.iter().map(|space| space.sample()).collect(),
//            _ => panic!("Cannot call sample_nested on non-vector space"),
//        }
//    }
//
//    /// Sample a single value from each of the nested spaces with a fixed seed.
//    pub fn sample_nested_with_seed(&self, seed: u64) -> Vec<Sample> {
//        match self {
//            Space::Vector { spaces } => {
//                spaces.iter().enumerate().map(|(index, space)| space.sample_with_seed(seed + index as u64)).collect()
//            }
//            _ => panic!("Cannot call sample_nested on non-vector space"),
//        }
//    }
//
//    /// Enumerate all possible values in the space.
//    pub fn enumerate(&self) -> Vec<Sample> {
//        match self {
//            Space::Discrete { n, start } => (0..*n).map(|i| Sample::Discrete(i + *start)).collect(),
//            Space::Box { low, high } => low
//                .iter()
//                .zip(high.iter())
//                .fold(vec![vec![]], |acc, (l, h)| {
//                    let range = (*l..=*h).collect::<Vec<i32>>();
//
//                    acc.into_iter()
//                        .flat_map(|sample| {
//                            range.iter().map(move |i| {
//                                let mut new_sample = sample.clone();
//                                new_sample.push(*i);
//                                new_sample
//                            })
//                        })
//                        .collect()
//                })
//                .into_iter()
//                .map(Sample::Box)
//                .collect(),
//            Space::OneOf { spaces } => spaces
//                .iter()
//                .enumerate()
//                .flat_map(|(idx, space)| {
//                    let sub_results = space.enumerate();
//                    sub_results.into_iter().map(move |sample| Sample::OneOf(idx as i32, Box::new(sample)))
//                })
//                .collect(),
//            Space::Tuple { spaces } => spaces
//                .iter()
//                .fold(vec![vec![]], |acc, space| {
//                    let sub_results = space.enumerate();
//                    acc.into_iter()
//                        .flat_map(|prefix| {
//                            sub_results.iter().map(move |sample| {
//                                let mut new_tuple = prefix.clone();
//                                new_tuple.push(sample.clone());
//                                new_tuple
//                            })
//                        })
//                        .collect()
//                })
//                .into_iter()
//                .map(Sample::Tuple)
//                .collect(),
//            Space::Dict { spaces } => {
//                let keys: Vec<_> = spaces.keys().cloned().collect();
//                let enumerations: Vec<_> = keys.iter().map(|key| spaces[key].enumerate()).collect();
//
//                enumerations
//                    .iter()
//                    .fold(vec![HashMap::new()], |acc, sub_enumeration| {
//                        acc.into_iter()
//                            .flat_map(|partial_dict| {
//                                // Capture a reference to `keys`
//                                let keys_ref = &keys;
//
//                                sub_enumeration.iter().map(move |sample| {
//                                    let mut new_dict = partial_dict.clone();
//                                    let key = &keys_ref[partial_dict.len()]; // Use partial_dict.len() for the index
//                                    new_dict.insert(key.clone(), sample.clone());
//                                    new_dict
//                                })
//                            })
//                            .collect::<Vec<_>>()
//                    })
//                    .into_iter()
//                    .map(Sample::Dict)
//                    .collect()
//            }
//            _ => panic!("Cannot call enumerate on vector space"),
//        }
//    }
//
//    /// Enumerate all possible values in the nested spaces.
//    pub fn enumerate_nested(&self) -> Vec<Vec<Sample>> {
//        match self {
//            Space::Vector { spaces } => spaces.iter().map(|space| space.enumerate()).collect(),
//            _ => panic!("Cannot call enumerate_nested on non-vector space"),
//        }
//    }
//
//    /// Format the space as a string.
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
//        let indent = "\t".repeat(level);
//
//        match self {
//            Space::Discrete { .. } => write!(f, "{}{:?}", indent, self),
//            Space::OneOf { spaces } => {
//                writeln!(f, "{}OneOf {{ spaces=[", indent)?;
//                for space in spaces {
//                    space.fmt(f, level + 1)?;
//                    writeln!(f)?;
//                }
//                write!(f, "{}])", indent)
//            }
//            Space::Box { .. } => write!(f, "{}{:?}", indent, self),
//            Space::Tuple { spaces } => {
//                write!(f, "{}Tuple {{ spaces=[", indent)?;
//                for space in spaces {
//                    space.fmt(f, level + 1)?;
//                    writeln!(f)?;
//                }
//                write!(f, "{}]}}", indent)
//            }
//            Space::Dict { spaces } => {
//                write!(f, "{}Dict {{ spaces={{", indent)?;
//                for (key, space) in spaces {
//                    write!(f, "{}{}: ", indent, key)?;
//                    space.fmt(f, level + 1)?;
//                }
//                write!(f, "{}}}}}", indent)
//            }
//            Space::Vector { spaces } => {
//                writeln!(f, "{}Vector {{ spaces=[", indent)?;
//                for space in spaces {
//                    space.fmt(f, level + 1)?;
//                    writeln!(f)?;
//                }
//                write!(f, "{}]}}", indent)
//            }
//        }
//    }
//}
//
//#[pymethods]
//impl Space {
//    fn __repr__(&self) -> String {
//        format!("{}", self)
//    }
//
//    fn __str__(&self) -> String {
//        format!("{}", self)
//    }
//
//    fn __eq__(&self, other: &Self) -> bool {
//        self == other
//    }
//
//    fn __len__(&self) -> usize {
//        self.len()
//    }
//
//    #[pyo3(name = "sample")]
//    fn py_sample(&self) -> PyObject {
//        Python::with_gil(|py| Sample::into_py(self.sample(), py))
//    }
//
//    #[pyo3(name = "sample_with_seed")]
//    fn py_sample_with_seed(&self, seed: u64) -> PyObject {
//        Python::with_gil(|py| Sample::into_py(self.sample_with_seed(seed), py))
//    }
//
//    #[pyo3(name = "sample_nested")]
//    fn py_sample_nested(&self) -> PyResult<PyObject> {
//        Python::with_gil(|py| Sample::to_python_nested(&self.sample_nested(), py))
//    }
//
//    #[pyo3(name = "sample_nested_with_seed")]
//    fn py_sample_nested_with_seed(&self, seed: u64) -> PyResult<PyObject> {
//        Python::with_gil(|py| Sample::to_python_nested(&self.sample_nested_with_seed(seed), py))
//    }
//
//    #[pyo3(name = "enumerate")]
//    fn py_enumerate(&self) -> PyResult<PyObject> {
//        Python::with_gil(|py| Sample::to_python_nested(&self.enumerate(), py))
//    }
//
//    #[pyo3(name = "enumerate_nested")]
//    fn py_enumerate_nested(&self) -> PyResult<PyObject> {
//        Python::with_gil(|py| Sample::to_python_nested_nested(&self.enumerate_nested(), py))
//    }
//}
//
//#[derive(Debug, Clone, PartialEq, Eq)]
//pub enum Sample {
//    // A value sampled from a discrete space.
//    Discrete(i32),
//
//    // A value sampled from one of multiple sub-spaces.
//    OneOf(i32, Box<Sample>),
//
//    // A value sampled from a box space.
//    Box(Vec<i32>),
//
//    // A value sampled from a tuple space.
//    Tuple(Vec<Sample>),
//
//    // A value sampled from a dictionary space.
//    Dict(HashMap<String, Sample>),
//}
//
//impl Hash for Sample {
//    fn hash<H: Hasher>(&self, state: &mut H) {
//        match self {
//            Sample::Discrete(val) => val.hash(state),
//            Sample::OneOf(idx, box_sample) => {
//                idx.hash(state);
//                box_sample.hash(state); // Hash the inner sample
//            }
//            Sample::Box(vec) => {
//                // Hash each element in the Vec
//                vec.hash(state);
//            }
//            Sample::Tuple(vec) => {
//                // Hash each element in the tuple
//                vec.hash(state);
//            }
//            Sample::Dict(map) => {
//                // Hash each key-value pair in the HashMap
//                let mut sorted_entries: Vec<_> = map.iter().collect();
//                sorted_entries.sort_by(|a, b| a.0.cmp(b.0));
//                for (key, value) in sorted_entries {
//                    key.hash(state);
//                    value.hash(state); // Hash the value (which is a `Sample`)
//                }
//            }
//        }
//    }
//}
//
//impl IntoPy<PyObject> for Sample {
//    fn into_py(self, py: Python<'_>) -> PyObject {
//        match self {
//            Sample::Discrete(val) => val.into_py(py),
//            Sample::Box(values) => {
//                let py_list = PyList::new_bound(py, values);
//                py_list.into()
//            }
//            Sample::OneOf(index, sample) => {
//                let py_list = PyList::new_bound(py, &[index.into_py(py), sample.into_py(py)]);
//                py_list.into()
//            }
//            Sample::Tuple(samples) => {
//                let py_list = PyList::new_bound(py, samples.into_iter().map(|s| s.into_py(py)).collect::<Vec<_>>());
//                py_list.into()
//            }
//            Sample::Dict(map) => {
//                let py_dict = PyDict::new_bound(py);
//                for (key, value) in map {
//                    py_dict.set_item(key.into_py(py), value.into_py(py)).expect("Failed to set item");
//                }
//                py_dict.into()
//            }
//        }
//    }
//}
//
//impl Sample {
//    fn to_python(sample: Sample, py: Python<'_>) -> PyObject {
//        match sample {
//            Sample::Discrete(val) => val.into_py(py),
//            Sample::Box(values) => {
//                let py_list = PyList::new_bound(py, values);
//                py_list.into()
//            }
//            Sample::OneOf(index, sample) => {
//                let py_list = PyList::new_bound(py, &[index.into_py(py), sample.into_py(py)]);
//                py_list.into()
//            }
//            Sample::Tuple(samples) => {
//                let py_list = PyList::new_bound(py, samples.into_iter().map(|s| s.into_py(py)).collect::<Vec<_>>());
//                py_list.into()
//            }
//            Sample::Dict(map) => {
//                let py_dict = PyDict::new_bound(py);
//                for (key, value) in map {
//                    py_dict.set_item(key.into_py(py), value.into_py(py)).expect("Failed to set item");
//                }
//                py_dict.into()
//            }
//        }
//    }
//
//    fn to_python_nested(nested_sample: &Vec<Sample>, py: Python<'_>) -> PyResult<PyObject> {
//        let py_list: Vec<_> = nested_sample.iter().map(|s| Sample::to_python(s.clone(), py)).collect();
//        Ok(PyList::new_bound(py, py_list).into())
//    }
//
//    fn to_python_nested_nested(nested_sample: &Vec<Vec<Sample>>, py: Python<'_>) -> PyResult<PyObject> {
//        let py_list =
//            nested_sample.iter().map(|s| Self::to_python_nested(s, py)).collect::<Result<Vec<PyObject>, _>>()?;
//        Ok(PyList::new_bound(py, py_list).into())
//    }
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use paste::paste;
//    use std::collections::HashSet;
//
//    macro_rules! panic_test_space {
//        ($name:ident, $init:expr, $( $method:ident ),+) => {
//            $(
//                paste! {
//                    #[test]
//                    #[should_panic(expected = "Cannot call " $method " on non-vector space")]
//                    fn [<test_ $name _throws_with_ $method>]() {
//                        $init.$method();
//                    }
//                }
//            )+
//        };
//
//        (nested $name:ident, $init:expr, $( $method:ident ),+) => {
//            $(
//                paste! {
//                    #[test]
//                    #[should_panic(expected = "Cannot call " $method " on vector space")]
//                    fn [<test_ $name _throws_with_ $method>]() {
//                        $init.$method();
//                    }
//                }
//            )+
//        };
//
//        ($name:ident, $message:expr, $init:expr, $method:ident) => {
//            paste! {
//                #[test]
//                #[should_panic(expected = $message)]
//                fn [<test_ $name _throws_with_ $method>]() {
//                    $init.$method();
//                }
//            }
//        };
//    }
//
//    #[test]
//    fn test_oneof_space_len() {
//        let space = Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);

//    }
//
//    #[test]
//    fn test_dict_space_len() {
//        let space = Space::new_dict(
//            vec![("a".to_string(), Space::new_discrete(3, 5)), ("b".to_string(), Space::new_discrete(2, 10))]
//                .into_iter()
//                .collect(),
//        );
//

//    }
//
//    #[test]
//    fn test_tuple_space_len() {
//        let space = Space::new_tuple(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);
//

//    }
//
//    #[test]
//    fn test_vector_space_len() {
//        let space = Space::new_vector(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);
//

//    }
//
//    #[test]
//    fn test_discrete_space_sample() {
//        let space = Space::new_discrete(5, 10);
//
//        // Sample without a fixed seed
//        let Sample::Discrete(sample) = space.sample() else {
//            panic!("Sample is not of type Sample::Discrete");
//        };
//

//
//        // Sample with a fixed seed
//        let seed = 42;
//        let Sample::Discrete(sample_with_seed) = space.sample_with_seed(seed) else {
//            panic!("Sample is not of type Sample::Discrete");
//        };

//
//        // Consistency check: repeat sampling with the same seed
//        let Sample::Discrete(repeated_sample) = space.sample_with_seed(seed) else {
//            panic!("Sample is not of type Sample::Discrete");
//        };

//    }
//
//    #[test]
//    fn test_box_space_sample() {
//        let space = Space::new_box(vec![0, 0, 0, 0], vec![1, 2, 3, 4]);
//
//        // Sample without a fixed seed
//        let Sample::Box(sample) = space.sample() else {
//            panic!("Sample is not of type Sample::Box");
//        };

//
//        // Sample with a fixed seed
//        let seed = 42;
//        let Sample::Box(sample_with_seed) = space.sample_with_seed(seed) else {
//            panic!("Sample is not of type Sample::Box");
//        };

//
//        // Consistency check: repeat sampling with the same seed
//        let Sample::Box(repeated_sample) = space.sample_with_seed(seed) else {
//            panic!("Sample is not of type Sample::Box");
//        };

//    }
//
//    #[test]
//    fn test_oneof_space_sample() {
//        let space = Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);
//
//        // Sample without a fixed seed
//        let Sample::OneOf(index, sample) = space.sample() else {
//            panic!("Sample is not of type Sample::OneOf");
//        };
//        let Sample::Discrete(sample) = *sample else {
//            panic!("Inner sample is not of type Sample::Discrete");
//        };

//
//        // Sample with a fixed seed
//        let seed = 42;
//        let Sample::OneOf(index, sample_with_seed) = space.sample_with_seed(seed) else {
//            panic!("Sample is not of type Sample::OneOf");
//        };
//        let Sample::Discrete(sample_with_seed) = *sample_with_seed else {
//            panic!("Inner sample is not of type Sample::Discrete");
//        };
//

//            (index == 0 && sample_with_seed >= 5 && sample_with_seed < 8)
//                || (index == 1 && sample_with_seed >= 10 && sample_with_seed < 12)
//        );
//
//        // Consistency check: repeat sampling with the same seed
//        let Sample::OneOf(repeated_index, repeated_sample) = space.sample_with_seed(seed) else {
//            panic!("Sample is not of type Sample::OneOf");
//        };
//        let Sample::Discrete(repeated_sample) = *repeated_sample else {
//            panic!("Inner sample is not of type Sample::Discrete");
//        };
//

//    }
//
//    #[test]
//    fn test_dict_space_sample() {
//        let space = Space::new_dict(
//            vec![("first".to_string(), Space::new_discrete(3, 5)), ("second".to_string(), Space::new_discrete(2, 10))]
//                .into_iter()
//                .collect(),
//        );
//
//        // Sample without a fixed seed
//        let Sample::Dict(sample) = space.sample() else {
//            panic!("Sample is not of type Sample::Dict");
//        };
//
//        let first_sample = sample.get("first").unwrap();
//        let second_sample = sample.get("second").unwrap();
//        let Sample::Discrete(first_sample) = first_sample else {
//            panic!("First sample is not of type Sample::Discrete");
//        };

//
//        let Sample::Discrete(second_sample) = second_sample else {
//            panic!("Second sample is not of type Sample::Discrete");
//        };

//
//        // Sample with a fixed seed
//        let seed = 42;
//        let Sample::Dict(sample_with_seed) = space.sample_with_seed(seed) else {
//            panic!("Sample is not of type Sample::Dict");
//        };
//
//        let Sample::Dict(repeated_sample_with_seed) = space.sample_with_seed(seed) else {
//            panic!("Sample is not of type Sample::Dict");
//        };
//

//    }
//
//    #[test]
//    fn test_tuple_space_sample_nested() {
//        let space = Space::new_tuple(vec![Space::new_discrete(5, 10), Space::new_discrete(2, 20)]);
//
//        // Test sampling without a fixed seed
//        let Sample::Tuple(sample) = space.sample() else {
//            panic!("Sample is not of type Sample::Tuple");
//        };
//
//        let Sample::Discrete(first_sample) = sample[0] else {
//            panic!("First sample is not of type Sample::Discrete");
//        };
//
//        let Sample::Discrete(second_sample) = sample[1] else {
//            panic!("Second sample is not of type Sample::Discrete");
//        };
//

//
//        // Test nested sampling with a fixed seed
//        let seed = 42;
//        let sample_with_seed = space.sample_with_seed(seed);
//        let Sample::Tuple(sample_with_seed) = sample_with_seed else {
//            panic!("Sample is not of type Sample::Tuple");
//        };
//
//        // Consistency check: repeat sampling with the same seed
//        let Sample::Tuple(repeated_sample) = space.sample_with_seed(seed) else {
//            panic!("Sample is not of type Sample::Tuple");
//        };
//

//    }
//
//    #[test]
//    fn test_vector_space_sample_nested() {
//        let space = Space::new_vector(vec![Space::new_discrete(5, 10), Space::new_discrete(2, 20)]);
//
//        // Test nested sampling without a fixed seed
//        let nested_sample = space.sample_nested();

//
//        let Sample::Discrete(first_sample) = nested_sample[0] else {
//            panic!("First sample is not of type Sample::Discrete");
//        };
//
//        let Sample::Discrete(second_sample) = nested_sample[1] else {
//            panic!("Second sample is not of type Sample::Discrete");
//        };
//

//
//        // Test nested sampling with a fixed seed
//        let seed = 42;
//        let nested_sample_with_seed = space.sample_nested_with_seed(seed);

//
//        let repeated_nested_sample = space.sample_nested_with_seed(seed);
//

//    }
//
//    #[test]
//    fn test_discrete_space_enumerate() {
//        let space = Space::new_discrete(5, 10);
//        let enumerated = space.enumerate();
//

//
//        for (i, sample) in enumerated.iter().enumerate() {

//        }
//    }
//
//    #[test]
//    fn test_box_space_enumerate() {
//        let space = Space::new_box(vec![0, 0, 0], vec![1, 2, 3]);
//
//        let result = space.enumerate();
//
//        assert_eq!(result.len(), 24);
//
//        let mut seen = HashSet::new();
//        for sample in result.iter() {
//            let Sample::Box(sample) = sample else { panic!("Sample is not of type Sample::Box") };
//
//            assert!(sample[0] >= 0 && sample[0] <= 1);
//            assert!(sample[1] >= 0 && sample[1] <= 2);
//            assert!(sample[2] >= 0 && sample[2] <= 3);
//
//            assert!(seen.insert(sample.clone()), "Duplicate enumeration found: {:?}", sample)
//        }
//    }
//
//    #[test]
//    fn test_dict_space_enumerate() {
//        let space = Space::new_dict(
//            vec![("first".to_string(), Space::new_discrete(3, 5)), ("second".to_string(), Space::new_discrete(2, 10))]
//                .into_iter()
//                .collect(),
//        );
//
//        let result = space.enumerate();
//
//        assert_eq!(result.len(), 6);
//
//        let mut seen = HashSet::new();
//
//        for sample in result.iter() {
//            let Sample::Dict(sample) = sample else { panic!("Sample is not of type Sample::Dict") };
//
//            let first_sample = sample.get("first").unwrap();
//            let second_sample = sample.get("second").unwrap();
//
//            let Sample::Discrete(first_sample) = first_sample else {
//                panic!("First sample is not of type Sample::Discrete")
//            };
//
//            let Sample::Discrete(second_sample) = second_sample else {
//                panic!("Second sample is not of type Sample::Discrete")
//            };
//
//            assert!(*first_sample >= 5 && *first_sample < 8);
//            assert!(*second_sample >= 10 && *second_sample < 12);
//
//            assert!(seen.insert(Sample::Dict(sample.clone())), "Duplicate enumeration found: {:?}", sample)
//        }
//    }
//
//    #[test]
//    fn test_oneof_space_enumerate() {
//        let space = Space::new_one_of(vec![Space::new_discrete(3, 5), Space::new_discrete(2, 10)]);
//
//        let result = space.enumerate();
//
//        assert_eq!(result.len(), 5);
//
//        let mut seen = HashSet::new();
//        for sample in result.iter() {
//            assert!(seen.insert(sample.clone()), "Duplicate enumeration found: {:?}", sample)
//        }
//    }
//
//    #[test]
//    fn test_vector_space_nested_enumerate() {
//        let space = Space::new_vector(vec![Space::new_discrete(5, 10), Space::new_discrete(2, 20)]);
//
//        let result = space.enumerate_nested();
//
//        assert_eq!(result.len(), 2);
//
//        let expected_first_space: Vec<Sample> = (10..15).map(|i| Sample::Discrete(i)).collect();
//        let expected_second_space: Vec<Sample> = (20..22).map(|i| Sample::Discrete(i)).collect();
//
//        assert_eq!(result[0], expected_first_space);
//        assert_eq!(result[1], expected_second_space);
//    }
//
//    panic_test_space!(discrete, Space::new_discrete(5, 10), sample_nested, enumerate_nested);
//    panic_test_space!(discrete, "Cannot call len on Discrete space", Space::new_discrete(5, 10), len);
//
//    panic_test_space!(box, Space::new_box(vec![0, 0, 0], vec![1, 2, 3]), sample_nested, enumerate_nested);
//    panic_test_space!(box, "Cannot call len on Box space", Space::new_box(vec![0, 0, 0], vec![1, 2, 3]), len);
//
//    panic_test_space!(tuple, Space::new_tuple(vec![Space::new_discrete(5, 10)]), sample_nested, enumerate_nested);
//
//    panic_test_space!(dict, Space::new_dict(HashMap::new()), sample_nested, enumerate_nested);
//
//    panic_test_space!(oneof, Space::new_one_of(vec![Space::new_discrete(3, 5)]), sample_nested, enumerate_nested);
//
//    panic_test_space!(nested vector, Space::new_vector(vec![Space::new_discrete(5, 10)]), sample, enumerate);
//}
