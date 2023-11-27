use std::collections::{BTreeMap, HashMap};

type TensorSize = Vec<usize>;

pub struct Tensor {
    _vec: Vec<u32>,
}

impl Tensor {
    pub fn new(v: Vec<u32>) -> Self {
        return Self { _vec: v };
    }

    pub fn shape(&self) -> TensorSize {
        vec![self._vec.len()]
    }

    pub fn sum(&self) -> u32 {
        self._vec.iter().sum()
    }

    pub fn mean(&self) -> f64 {
        self.sum() as f64 / self._vec.len() as f64
    }

    pub fn unique(&self) -> (Vec<u32>, Vec<usize>) {
        let mut counts: BTreeMap<u32, usize> = BTreeMap::new();

        for i in 0..self._vec.len() {
            if !counts.contains_key(&self._vec[i]) {
                counts.insert(self._vec[i], 0);
            }

            *counts.get_mut(&self._vec[i]).unwrap() += 1;
        }

        let mut keys = Vec::new();
        let mut values = Vec::new();

        for (k, v) in counts {
            keys.push(k);
            values.push(v);
        }

        (keys, values)
    }
}
