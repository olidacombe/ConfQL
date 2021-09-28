use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

mod values;
use values::{get_sub_value_at_address, value_from_file};

#[derive(Error, Debug)]
pub enum DataResolverError {
	#[error("Key `{0}` not found")]
	KeyNotFound(String),
	#[error("Data not found")]
	DataNotFound,
	#[error(transparent)]
	YamlError(#[from] serde_yaml::Error),
	#[error(transparent)]
	IOError(#[from] std::io::Error),
}

pub struct DataResolver<'a> {
	root: &'a Path,
}

impl<'a> DataResolver<'a> {
	pub fn get_non_nullable<T>(&self, address: &[&str]) -> Result<T, DataResolverError>
	where
		T: for<'de> Deserialize<'de>,
	{
		self.get_nullable(address)?
			.ok_or(DataResolverError::DataNotFound)
	}

	pub fn get_non_nullable_list<T>(&self, address: &[&str]) -> Result<Vec<T>, DataResolverError>
	where
		T: for<'de> Deserialize<'de>,
	{
		self.get_nullable_list(address)?
			.ok_or(DataResolverError::DataNotFound)
	}

	pub fn get_nullable<T>(&self, address: &[&str]) -> Result<Option<T>, DataResolverError>
	where
		T: for<'de> Deserialize<'de>,
	{
		let path = self.root.join("index.yml");
		let value = value_from_file(&path)?;
		let value = get_sub_value_at_address(&value, address)?;
		let value = serde_yaml::from_value(value.to_owned())?;
		Ok(value)
	}

	pub fn get_nullable_list<T>(
		&self,
		address: &[&str],
	) -> Result<Option<Vec<T>>, DataResolverError>
	where
		T: for<'de> Deserialize<'de>,
	{
		Err(DataResolverError::DataNotFound)
	}
}

impl<'a> From<&'a Path> for DataResolver<'a> {
	fn from(path: &'a Path) -> Self {
		Self { root: path }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::{Error, Result};
	use indoc::indoc;
	use std::path::PathBuf;
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
