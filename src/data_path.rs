use anyhow::{Error, Result};
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
    node_type: NodeType,
}

impl DataPath {
    pub fn new(base_dir: &Path, key_path: Vec<&'static str>) -> Result<Self> {
        if !base_dir.is_dir() {
            return Err(Error::msg(format!(
                "DataPath base {} is not a directory",
                base_dir.to_str().unwrap_or("None")
            )));
        }
        let mut dp = Self {
            read_path: PathBuf::from(base_dir),
            reverse_key_path: key_path,
            node_type: NodeType::Dir,
        };
        dp.reverse_key_path.reverse();
        Ok(dp)
    }

    // TODO doctest the shit out of this
    pub fn next(&mut self) -> Option<&Self> {
        match self.node_type {
            NodeType::Dir => self.reverse_key_path.pop().map(move |dir| {
                self.read_path.push(dir);
                self.node_type = NodeType::File;
                &*self
            }),
            NodeType::File => {
                if !self.read_path.is_dir() {
                    return None;
                }
                self.node_type = NodeType::Dir;
                Some(&*self)
            }
        }
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

struct DataPathIter {
    data_path: Option<DataPath>,
}

impl Iterator for DataPathIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut data_path) = self.data_path {
            data_path.next();
        }
        Some("balls".to_owned())
    }
}

impl std::fmt::Display for DataPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut display = format!(
            "{}({})",
            match self.node_type {
                NodeType::Dir => "Dir",
                NodeType::File => "File",
            },
            self.read_path.to_str().unwrap_or("None")
        );
        if !self.reverse_key_path.is_empty() {
            let mut path = self.reverse_key_path.clone();
            path.reverse();
            display += &format!("::{}", path.join("."));
        }
        write!(f, "{}", display)
    }
}

enum NodeType {
    File,
    Dir,
}

//struct DataPathWalker {
//data_path: DataPath,
//node_type: NodeType,
//}

//impl Iterator for DataPathWalker {
// //TODO Item is an iterator of files
//type Item = String;

//fn next(&mut self) -> Option<Self::Item> {
//type NT = NodeType;
//match self.node_type {
//NT::Dir => {
//self.data_path = self.data_path.next();
//self.node_type = NT::File;
//}
//NT::File => {}
//}
//Some("Fuck".to_owned())
//}
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_path_next() -> Result<()> {
        let mut results: Vec<String> = vec![];
        let mut dp = DataPath::new(&Path::new("/tmp"), vec!["a", "b", "c"])?;
        // TODO unstupid and an iterator.map.collect
        loop {
            results.push(format!("{}", dp));
            if let None = dp.next() {
                break;
            }
        }
        assert_eq!(
            results,
            vec![
                "Dir(/tmp)::a.b.c",
                "File(/tmp/a)::b.c",
                "Dir(/tmp/a)::b.c",
                "File(/tmp/a/b)::c",
                "Dir(/tmp/a/b)::c",
                "File(/tmp/a/b/c)",
                "Dir(/tmp/a/b/c)",
            ]
        );
        Ok(())
    }
}
