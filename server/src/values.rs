use serde::Serialize;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Serialize)]
pub struct Values {
    values: HashMap<String, VecDeque<f32>>,
    #[serde(skip)]
    max_len: usize,
}

impl Default for Values {
    fn default() -> Self {
        Self::with_capacity(3600)
    }
}

impl Values {
    pub fn with_capacity(max_len: usize) -> Self {
        Self {
            values: Default::default(),
            max_len,
        }
    }

    pub fn push(&mut self, key: String, values: &[f32]) {
        let vec = self
            .values
            .entry(key)
            .or_insert_with(|| VecDeque::with_capacity(self.max_len));

        if vec.len() + values.len() > self.max_len {
            vec.drain(0..(vec.len() + values.len() - self.max_len));
        }
        vec.extend(values)
    }
}
