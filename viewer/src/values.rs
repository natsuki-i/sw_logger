use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, PartialEq)]
pub struct Values {
    values: BTreeMap<String, VecDeque<f32>>,
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

    pub fn max_len(&self) -> usize {
        self.max_len
    }

    pub fn set_max_len(&mut self, max_len: usize) {
        self.max_len = max_len;
        for v in self.values.values_mut() {
            if v.len() < max_len {
                v.reserve(max_len - v.len());
            }
            if v.len() > max_len {
                v.drain(0..(v.len() - max_len));
            }
        }
    }

    pub fn push(&mut self, key: String, values: Vec<f32>) {
        let v = self
            .values
            .entry(key)
            .or_insert_with(|| VecDeque::with_capacity(self.max_len));
        if v.len() + values.len() > self.max_len {
            v.drain(0..(v.len() + values.len() - self.max_len));
        }
        v.extend(values);
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }

    pub fn iter_for_key(
        &self,
        key: &str,
    ) -> Option<impl Iterator<Item = &f32> + ExactSizeIterator + DoubleEndedIterator> {
        self.values.get(key).map(|v| v.iter())
    }

    pub fn get_last_value_for_key(&self, key: &str) -> Option<f32> {
        self.values
            .get(key)
            .as_ref()
            .and_then(|v| v.back())
            .cloned()
    }
}
