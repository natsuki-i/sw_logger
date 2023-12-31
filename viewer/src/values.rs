use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

#[derive(Debug, PartialEq, Deserialize)]
pub struct Values {
    values: BTreeMap<String, VecDeque<f32>>,
    max_len: usize,
}

impl Serialize for Values {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct V {
            values: BTreeMap<String, Vec<f32>>,
            max_len: usize,
        }
        V {
            values: self
                .values
                .iter()
                .map(|(k, _)| (k.clone(), vec![]))
                .collect(),
            max_len: self.max_len,
        }
        .serialize(serializer)
    }
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

    pub fn values_for_key(&self, key: &str) -> Option<&VecDeque<f32>> {
        self.values.get(key)
    }

    pub fn get_last_value_for_key(&self, key: &str) -> Option<f32> {
        self.values
            .get(key)
            .as_ref()
            .and_then(|v| v.back())
            .cloned()
    }

    pub fn save_csv<'a, K>(&self, path: &Path, keys: K) -> Result<(), std::io::Error>
    where
        K: Iterator<Item = &'a String>,
    {
        let mut writer = BufWriter::new(File::create(path)?);
        let mut values = Vec::with_capacity(self.values.len());
        let mut first = true;
        let mut max_len = 0;
        for key in keys {
            if let Some(v) = self.values_for_key(key) {
                if first {
                    first = false
                } else {
                    writer.write_all(",".as_bytes())?;
                }
                writer.write_all(key.as_bytes())?;
                max_len = max_len.max(v.len());
                values.push(v);
            }
        }
        writer.write_all("\n".as_bytes())?;
        for index in 0..max_len {
            for (i, vec) in values.iter().enumerate() {
                let offset = max_len - vec.len();
                if offset > index {
                    writer.write_all(",".as_bytes())?;
                    continue;
                }
                if let Some(v) = vec.get(index - offset) {
                    if i == 0 {
                        writer.write_fmt(format_args!("{}", v))?;
                    } else {
                        writer.write_fmt(format_args!(",{}", v))?;
                    }
                } else {
                    writer.write_all(",".as_bytes())?;
                }
            }
            writer.write_all("\n".as_bytes())?;
        }
        writer.flush()?;
        Ok(())
    }
}
