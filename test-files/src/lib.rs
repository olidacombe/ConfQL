use std::path::{Path, PathBuf};
use tempdir::TempDir;
use thiserror::Error;
use touch::file;

#[derive(Error, Debug)]
pub enum TestFilesError {
    #[error("Path error `{path:?}`")]
    PathError { path: String },
    #[error(transparent)]
    FileWriteError(#[from] touch::Error),
    #[error(transparent)]
    TempDirError(#[from] std::io::Error),
}

pub struct TestFiles(TempDir);

impl TestFiles {
    pub fn file(&self, path: &str, content: &str) -> Result<&Self, TestFilesError> {
        file::write(
            self.slash(path).to_str().ok_or(TestFilesError::PathError {
                path: path.to_string(),
            })?,
            content,
            true,
        )?;
        Ok(self)
    }

    pub fn new() -> Result<Self, TestFilesError> {
        Ok(Self(TempDir::new(env!("CARGO_PKG_NAME"))?))
    }

    pub fn path(&self) -> &Path {
        self.0.path()
    }

    fn slash(&self, relative_path: &str) -> PathBuf {
        self.path().join(relative_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use indoc::indoc;
    use std::fs;

    #[test]
    fn makes_deletes_files() -> Result<()> {
        let tmp_path: Option<&Path>;
        {
            let files = TestFiles::new().unwrap();
            tmp_path = Some(files.path());

            let content = indoc! {"
                ---
                version: 3
            "};

            files.file("a/b/index.yml", content)?;
            let file_path = tmp_path.unwrap().join("a").join("b").join("index.yml");
            let written_content = fs::read_to_string(file_path).unwrap();
            assert_eq!(written_content, content);
        }
        Ok(())
    }
}
