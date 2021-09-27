use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataResolverError {
	#[error("Data not found")]
	DataNotFound,
}

pub struct DataResolver {}

impl DataResolver {}

impl From<&Path> for DataResolver {
	fn from(path: &Path) -> Self {
		Self {}
	}
}

impl DataResolver {
	pub fn get_non_nullable<T>(&self, address: &[&str]) -> Result<T, DataResolverError> {
		Err(DataResolverError::DataNotFound)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::{Error, Result};
	use indoc::indoc;
	use tempdir::TempDir;
	use touch::file;

	struct Mocks(TempDir);

	impl Mocks {
		fn file(&self, path: &str, content: &str) -> Result<&Self> {
			file::write(
				self.slash(path)
					.to_str()
					.ok_or(Error::msg("mock path build failure"))?,
				content,
				true,
			)?;
			Ok(self)
		}

		fn new() -> Self {
			Self(TempDir::new(env!("CARGO_PKG_NAME")).unwrap())
		}

		fn path(&self) -> &Path {
			self.0.path()
		}

		fn resolver(&self) -> DataResolver {
			DataResolver::from(self.0.path())
		}

		fn slash(&self, relative_path: &str) -> PathBuf {
			self.path().join(relative_path)
		}
	}

	#[test]
	fn resolves_non_nullable_int_at_root() -> Result<()> {
		let mocks = Mocks::new();
		mocks.file(
			"index.yml",
			indoc! {"
				---
				version: 3
			"},
		)?;
		let i: u32 = mocks.resolver().get_non_nullable(&["version"])?;
		assert_eq!(i, 3);
		Ok(())
	}
}
