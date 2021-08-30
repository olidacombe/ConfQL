//TODO try this https://www.reddit.com/r/learnrust/comments/l8nqwr/blanket_trait_implementation_for_types_that/
use anyhow::{Context, Error, Result};
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use serde::Deserialize;
use serde_yaml::Value;
use std::alloc;
use std::fs;
use std::iter;
use std::iter::{IntoIterator, Iterator};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

macro_rules! typename {
    ($T:ty) => {
        std::any::type_name::<$T>()
    };
}

/// Returns reference to sub-value of a deserialized Value
fn get_sub_value_reverse_index<'a>(value: &'a Value, reverse_index: &[&str]) -> Result<&'a Value> {
    return reverse_index
        .iter()
        .rev()
        .fold_while(Ok(value), |acc, i| match acc.unwrap().get(i) {
            Some(v) => Continue(Ok(v)),
            _ => Done(Err(Error::msg(format!("Key {} not found", i,)))),
        })
        .into_inner();
}

pub struct DataPath<'a, T>
where
    T: for<'de> Deserialize<'de>,
{
    read_path: PathBuf,
    reverse_key_path: Vec<&'a str>,
    node_type: NodeType,
    t: PhantomData<T>,
}

trait LeafItems<'a, T>
where
    T: for<'de> Deserialize<'de>,
{
    fn leaf_files(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a>;
    fn leaf_object_from_value(&self, value: Value) -> Result<T>;
}

impl<'a, T> LeafItems<'a, Vec<T>> for DataPath<'a, Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    fn leaf_files(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        match fs::read_dir(&self.read_path) {
            Ok(reader) => Box::new(
                reader
                    .filter_map(|dir_entry| dir_entry.ok())
                    .map(|dir_entry| dir_entry.path()),
            ),
            _ => Box::new(iter::empty::<PathBuf>()),
        }
    }

    fn leaf_object_from_value(&self, value: Value) -> Result<Vec<T>> {
        let value = serde_yaml::from_value(value).context(format!(
            "Failed to deserialize {} to construct Vec<{}>",
            typename!(T),
            typename!(T)
        ))?;
        Ok(vec![value])
    }
}

impl<'a, T> LeafItems<'a, T> for DataPath<'a, T>
where
    T: for<'de> Deserialize<'de>,
{
    default fn leaf_files(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        self.dir_index()
    }
    default fn leaf_object_from_value(&self, value: Value) -> Result<T> {
        self.object_from_value(value)
    }
}

impl<'a, T> DataPath<'a, T>
where
    T: for<'de> Deserialize<'de>,
{
    pub fn new(base_dir: &Path, key_path: Vec<&'a str>) -> Result<Self> {
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
            t: PhantomData,
        };
        dp.reverse_key_path.reverse();
        Ok(dp)
    }

    fn dir_index(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        Box::new(iter::once(self.read_path.join("index.yml")))
    }

    fn is_leaf(&self) -> bool {
        match self.node_type {
            NodeType::Dir => self.reverse_key_path.is_empty(),
            _ => false,
        }
    }

    pub fn files(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        match self.is_leaf() {
            true => self.leaf_files(),
            false => match self.node_type {
                NodeType::Dir => self.dir_index(),
                NodeType::File => Box::new(iter::once(self.read_path.with_extension("yml"))),
            },
        }
    }

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

    fn object_from_value(&self, value: Value) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_yaml::from_value(value).context(format!("Failed to deserialize to {}", typename!(T)))
    }

    fn get_object(&self, path: PathBuf) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let file = std::fs::File::open(&path)?;
        let value = serde_yaml::from_reader::<_, Value>(file)
            .with_context(|| format!("Failed to parse {}", &self))?;
        let value = get_sub_value_reverse_index(&value, &self.key_path())?.to_owned();
        let object: T = match self.is_leaf() {
            false => self.object_from_value(value),
            true => self.leaf_object_from_value(value),
        }?;
        Ok(object)
    }

    pub fn key_path(&self) -> Vec<&'a str> {
        let mut key_path = self.reverse_key_path.clone();
        key_path.reverse();
        key_path
    }
}

pub struct DataPathIter<'a, T>
where
    T: for<'de> Deserialize<'de>,
{
    data_path: Option<DataPath<'a, T>>,
    file_iterator: Box<dyn Iterator<Item = PathBuf> + 'a>,
}

impl<'a, T> DataPathIter<'a, T>
where
    T: for<'de> Deserialize<'de>,
{
    pub fn new(base_dir: &Path, key_path: Vec<&'a str>) -> Self {
        match DataPath::new(base_dir, key_path) {
            Ok(data_path) => Self {
                file_iterator: data_path.files(),
                data_path: Some(data_path),
            },
            _ => Self {
                file_iterator: Box::new(iter::empty::<PathBuf>()),
                data_path: None,
            },
        }
    }
}

impl<'a, T> Iterator for DataPathIter<'a, T>
where
    T: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ref mut data_path) = self.data_path {
            match self.file_iterator.next() {
                Some(path) => {
                    if let Ok(object) = data_path.get_object(path) {
                        return Some(object);
                    }
                }
                None => match data_path.next() {
                    Some(data_path) => {
                        self.file_iterator = data_path.files();
                    }
                    None => self.data_path = None,
                },
            }
        }
        None
    }
}

impl<'a, T> std::fmt::Display for DataPath<'a, T>
where
    T: for<'de> Deserialize<'de>,
{
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

#[cfg(test)]
mod tests {
    extern crate fixtures;

    use super::*;
    use fixtures::models::Hero;
    use fixtures::DATA_PATH;
    use std::fs;
    use tempdir::TempDir;
    use touch::file;

    fn datatree(trunk: &[&str]) -> Result<TempDir> {
        let temp = TempDir::new(env!("CARGO_PKG_NAME"))?;
        let mut path_buf = temp.path().to_path_buf();
        let path_buf = trunk
            .iter()
            .fold(temp.path().to_path_buf(), |buf, dir| buf.join(dir));
        fs::create_dir_all(&path_buf)?;
        Ok(temp)
    }

    /// Makes some empty data files in a temporary directory:
    /// <tmp_dir>
    /// ├── a
    /// │   ├── b
    /// │   │   └── c
    /// │   │       ├── 1.yml
    /// │   │       ├── 2.yml
    /// │   │       └── 3.yml
    /// │   ├── b.yml
    /// │   └── index.yml
    /// └── index.yml
    fn data_path_test_files() -> Result<TempDir> {
        let temp = datatree(&["a", "b", "c"])?;
        let mut path_buf = temp.path().to_path_buf();
        file::create(path_buf.join("index.yml").to_str().unwrap(), false)?;
        path_buf.push("a");
        file::create(path_buf.join("index.yml").to_str().unwrap(), false)?;
        file::create(path_buf.join("b.yml").to_str().unwrap(), false)?;
        path_buf.push("b");
        path_buf.push("c");
        file::create(path_buf.join("1.yml").to_str().unwrap(), false)?;
        file::create(path_buf.join("2.yml").to_str().unwrap(), false)?;
        file::create(path_buf.join("3.yml").to_str().unwrap(), false)?;
        Ok(temp)
    }

    #[test]
    fn data_path_next() -> Result<()> {
        let mut results = Vec::<String>::new();
        let temp = datatree(&["a", "b", "c"])?;
        let base = temp.path().to_str().unwrap();
        let mut dp = DataPath::<bool>::new(&temp.path(), vec!["a", "b", "c"])?;
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
            .iter()
            .map(|s| s.replace("/tmp", base))
            .collect::<Vec<String>>()
        );
        Ok(())
    }

    macro_rules! assert_data_path_files_iterator_result {
        ($data_path:expr, $expected:expr) => {
            assert_data_path_files_iterator_result!($data_path, $expected, &$data_path.read_path);
        };
        ($data_path:expr, $expected:expr, $base:expr) => {
            let mut vec_to_compare = $data_path
                .files()
                .map(|ref f| {
                    (f.strip_prefix($base).unwrap_or(f))
                        .to_str()
                        .unwrap()
                        .to_owned()
                })
                .collect::<Vec<String>>();
            vec_to_compare.sort();
            assert_eq!(vec_to_compare, $expected);
        };
    }

    #[test]
    fn data_path_files_high_dir() -> Result<()> {
        let temp = data_path_test_files()?;
        let path_buf = temp.path().to_path_buf();

        let data_path = DataPath::<bool> {
            read_path: path_buf.clone(),
            reverse_key_path: vec!["a", "b"],
            node_type: NodeType::Dir,
            t: PhantomData,
        };
        assert_data_path_files_iterator_result!(data_path, vec!["index.yml"]);

        let data_path = DataPath::<Vec<bool>> {
            read_path: path_buf.clone(),
            reverse_key_path: vec!["a", "b"],
            node_type: NodeType::Dir,
            t: PhantomData,
        };
        assert_data_path_files_iterator_result!(data_path, vec!["index.yml"]);

        Ok(())
    }

    #[test]
    fn data_path_files_file() -> Result<()> {
        let temp = data_path_test_files()?;
        let path_buf = temp.path();

        let data_path = DataPath::<bool> {
            read_path: path_buf.join("a"),
            reverse_key_path: vec!["b"],
            node_type: NodeType::File,
            t: PhantomData,
        };
        assert_data_path_files_iterator_result!(data_path, vec!["a.yml"], &path_buf);

        let data_path = DataPath::<Vec<bool>> {
            read_path: path_buf.join("a"),
            reverse_key_path: vec!["b"],
            node_type: NodeType::File,
            t: PhantomData,
        };
        assert_data_path_files_iterator_result!(data_path, vec!["a.yml"], &path_buf);

        Ok(())
    }

    #[test]
    fn data_path_files_leaf_dir_non_array_type() -> Result<()> {
        let temp = data_path_test_files()?;
        let path_buf = temp.path().join("a").join("b").join("c");

        let data_path = DataPath::<bool> {
            read_path: path_buf.clone(),
            reverse_key_path: vec![],
            node_type: NodeType::Dir,
            t: PhantomData,
        };

        assert_data_path_files_iterator_result!(data_path, vec!["index.yml"]);

        Ok(())
    }

    // FIXME
    #[test]
    fn data_path_files_leaf_dir_array_type() -> Result<()> {
        let temp = data_path_test_files()?;
        let path_buf = temp.path().join("a").join("b").join("c");

        let data_path = DataPath::<Vec<bool>> {
            read_path: path_buf.clone(),
            reverse_key_path: vec![],
            node_type: NodeType::Dir,
            t: PhantomData,
        };

        assert_data_path_files_iterator_result!(data_path, vec!["1.yml", "2.yml", "3.yml"]);

        Ok(())
    }

    #[test]
    fn test_get_sub_value_reverse_index() -> Result<()> {
        use crate::yaml;
        let value = yaml!(
            r"#---
            A:
                B:
                    C:
                        presence: welcome
            #"
        );

        let sub_value = get_sub_value_reverse_index(&value, &vec![])?;
        assert_eq!(*sub_value, value);

        let sub_value = get_sub_value_reverse_index(&value, &vec!["A"])?;
        assert_eq!(
            *sub_value,
            yaml!(
                r#"---
                B:
                    C:
                        presence: welcome
                "#
            )
        );

        let sub_value = get_sub_value_reverse_index(&value, &vec!["B", "A"])?;
        assert_eq!(
            *sub_value,
            yaml!(
                r#"---
                C:
                    presence: welcome
                "#
            )
        );

        let sub_value = get_sub_value_reverse_index(&value, &vec!["C", "B", "A"])?;
        assert_eq!(
            *sub_value,
            yaml!(
                r#"---
                presence: welcome
                "#
            )
        );
        Ok(())
    }

    #[test]
    fn test_data_path_iter_multi() -> Result<()> {
        let mut result: Vec<Vec<Hero>> = DataPathIter::new(&DATA_PATH, vec!["heroes"]).collect();
        result.sort();
            let expected = vec![
                vec![Hero {
                    // index.yml
                    name: "Andy Anderson".to_owned(),
                    id: 1,
                    powers: vec!["eating".to_owned(), "sleeping".to_owned()]
                }],
                vec![Hero {
                    // heroes/charles.yml
                    name: "Charles Charleston".to_owned(),
                    id: 3,
                    powers: vec!["moaning".to_owned(), "cheating".to_owned()]
                }],
                vec![Hero {
                    // heroes/kevin.yml
                    name: "Kevin Kevinson".to_owned(),
                    id: 2,
                    powers: vec!["hunting".to_owned(), "fighting".to_owned()]
                }],
            ];
        assert_eq!(
            result,
            expected
        );
        Ok(())
    }

    #[test]
    fn test_is_leaf() -> Result<()> {
        let temp = data_path_test_files()?;

        macro_rules! assert_leaf_result {
            ($reverse_key_path:expr, $node_type:expr, $expected:expr) => {
                let data_path = DataPath::<bool> {
                    read_path: temp.path().to_path_buf(),
                    reverse_key_path: $reverse_key_path,
                    node_type: $node_type,
                    t: PhantomData,
                };
                assert_eq!(data_path.is_leaf(), $expected);
            };
        }
        assert_leaf_result!(vec!["a"], NodeType::File, false);
        assert_leaf_result!(vec!["a"], NodeType::Dir, false);
        assert_leaf_result!(vec![], NodeType::File, false);
        // Only empty reverse_key_path on NodeType::Dir is a leaf
        assert_leaf_result!(vec![], NodeType::Dir, true);

        Ok(())
    }
}