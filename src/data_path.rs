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

#[derive(Clone)]
enum DataPathBuffer {
    Dir(PathBuf),
    File(PathBuf),
}

#[derive(Clone)]
struct DataPath {
    read_path: DataPathBuffer,
    reverse_key_path: Vec<&'static str>,
}

impl DataPath {
    pub fn new(base_dir: &Path, key_path: Vec<&'static str>) -> Self {
        use DataPathBuffer::{Dir, File};
        let mut dp = Self {
            read_path: Dir(PathBuf::from(base_dir)),
            reverse_key_path: key_path,
        };
        dp.reverse_key_path.reverse();
        dp
    }
    // TODO doctest the shit out of this
    pub fn descend(mut self) -> Option<Self> {
        //use DataPathBuffer::{Dir, File};
        //match self.read_path {
        //File(ref mut path) => self.reverse_key_path.pop().map(|&dir| {
        //path.push(dir);
        //self.read_path = Dir(path);
        //self
        //}),
        //Dir(path) => {
        //self.read_path = File(path);
        //Some(self)
        //}
        //}
        None
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
        use DataPathBuffer::{Dir, File};
        let mut display = match self.read_path {
            File(ref path) => format!("File({})", path.to_str().unwrap_or("???")),
            Dir(ref path) => format!("Dir({})", path.to_str().unwrap_or("???")),
        };
        if self.reverse_key_path.len() > 0 {
            let mut path = self.reverse_key_path.clone();
            path.reverse();
            display += &format!("::{}", path.join("."));
        }
        write!(f, "{}", display)
    }
}

struct DataPathIter(Option<DataPath>);

impl<'a> Iterator for DataPathIter {
    // TODO a completely different Item?
    // like struct {
    //   files: Vec<&str>,
    //   key_path: Vec<&str>
    // }
    // ???
    //
    // YEAH, totally different.  No mutation
    // on DataPath, just iter holds an index
    // and iter is keeping track of what
    // filenames to hit
    type Item = DataPath;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().map(|data_path| {
            let next = data_path.clone();
            self.0 = data_path.descend();
            next
        })
    }
}

impl IntoIterator for DataPath {
    type Item = DataPath;
    type IntoIter = DataPathIter;

    fn into_iter(self) -> Self::IntoIter {
        DataPathIter(Some(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_path_descend() {
        use DataPathBuffer::{Dir, File};
        let dp = DataPath::new(&Path::new("/tmp"), vec!["a", "b", "c"]);
        let mut results: Vec<String> = vec![];
        for p in dp {
            results.push(format!("{}", p));
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
    }
}
