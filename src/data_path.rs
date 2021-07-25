use anyhow::Result;
use impls::impls;
use serde::Deserialize;
use std::iter::{IntoIterator, Iterator};
use std::path::{Path, PathBuf};

macro_rules! gql_single_or_collection {
    ($T:ty, $a:expr, $b:expr) => {
        match impls!($T: IntoIterator) {
            false => $a,
            true => $b,
        }
    };
}

struct DataPath {
    read_path: PathBuf,
    reverse_key_path: Vec<&'static str>,
}

impl DataPath {
    pub fn new(base_dir: &Path, key_path: Vec<&'static str>) -> Self {
        let mut dp = Self {
            read_path: PathBuf::from(base_dir),
            reverse_key_path: key_path,
        };
        dp.reverse_key_path.reverse();
        dp
    }

    // TODO doctest the shit out of this
    pub fn descend(mut self) -> Option<Self> {
        self.reverse_key_path.pop().map(|dir| {
            self.read_path.push(dir);
            self
        })
    }

    async fn get_first_object<T>(&self) -> Result<T> {
        todo!()
    }

    async fn get_all_objects<T>(&self) -> Result<T> {
        todo!()
    }

    fn filenames<T>(&self) -> Vec<&'static str> {
        gql_single_or_collection!(T, vec!["index.yml"], vec!["*.yml"])
    }

    pub async fn get_object<T>(&self) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        todo!()
    }
}

impl std::fmt::Display for DataPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut display = self.read_path.to_str().unwrap_or("???").to_owned();
        if self.reverse_key_path.len() > 0 {
            let mut path = self.reverse_key_path.clone();
            path.reverse();
            display += &format!("::{}", path.join("."));
        }
        write!(f, "{}", display)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_path_descend() {
        let mut results: Vec<String> = vec![];
        let mut dp = Some(DataPath::new(&Path::new("/tmp"), vec!["a", "b", "c"]));
        // TODO unstupid and an iterator.map.collect
        loop {
            match dp {
                Some(p) => {
                    results.push(format!("{}", p));
                    dp = p.descend();
                }
                None => {
                    break;
                }
            }
        }
        assert_eq!(
            results,
            vec!["/tmp::a.b.c", "/tmp/a::b.c", "/tmp/a/b::c", "/tmp/a/b/c",]
        );
    }
}
