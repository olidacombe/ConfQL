use serde::Deserialize;
use std::fs;
use std::iter;
use std::marker::PhantomData;
use std::path::PathBuf;

use super::values::{get_sub_value_at_address, value_from_file};
use super::DataResolverError;

trait LeafObject<'a, T>
where
	T: for<'de> Deserialize<'de>,
{
	fn leaf_object(&self) -> Result<T, DataResolverError>;
}

impl<'a, T> LeafObject<'a, Vec<T>> for DataPathIter<'a, Vec<T>>
where
	T: for<'de> Deserialize<'de>,
{
	fn leaf_object(&self) -> Result<Vec<T>, DataResolverError> {
		self.data_path
			.as_ref()
			.ok_or(DataResolverError::EmptyDataPathAccess)?
			.get_dir_objects()
	}
}

impl<'a, T> LeafObject<'a, T> for DataPathIter<'a, T>
where
	T: for<'de> Deserialize<'de>,
{
	default fn leaf_object(&self) -> Result<T, DataResolverError> {
		self.data_path
			.as_ref()
			.ok_or(DataResolverError::EmptyDataPathAccess)?
			.get_dir_object()
	}
}

enum DataPathIterState {
	Dir,
	File,
}

pub struct DataPathIter<'a, T> {
	data_path: Option<DataPath<'a>>,
	state: DataPathIterState,
	target_type: PhantomData<T>,
}

impl<'a, T> Iterator for DataPathIter<'a, T>
where
	T: for<'de> Deserialize<'de>,
{
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		use DataPathIterState::{Dir, File};
		while let Some(ref mut data_path) = self.data_path {
			match self.state {
				File => {
					self.state = Dir;
					if let Ok(val) = data_path.get_file_object() {
						return Some(val);
					}
				}
				Dir => {
					self.state = File;
					if data_path.is_complete() {
						let leaf = self.leaf_object();
						self.data_path = None;
						if let Ok(val) = leaf {
							return Some(val);
						}
					} else {
						let val = data_path.get_dir_object();
						data_path.next();
						if let Ok(val) = val {
							return Some(val);
						}
					}
				}
			}
		}
		None
	}
}

pub struct DataPath<'a> {
	pub path: PathBuf,
	pub address: &'a [&'a str],
}

impl<'a> DataPath<'a> {
	pub fn next(&mut self) -> Option<&Self> {
		self.address.split_first().map(move |(head, tail)| {
			self.path.push(head);
			self.address = tail;
			&*self
		})
	}
	fn file(&self) -> PathBuf {
		self.path.with_extension("yml")
	}
	fn files(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
		match fs::read_dir(&self.path) {
			Ok(reader) => Box::new(
				reader
					.filter_map(|dir_entry| dir_entry.ok())
					.map(|dir_entry| dir_entry.path()),
			),
			_ => Box::new(iter::empty::<PathBuf>()),
		}
	}
	pub fn get_dir_object<T>(&self) -> Result<T, DataResolverError>
	where
		T: for<'de> Deserialize<'de>,
	{
		self.get_object(self.index())
	}
	pub fn get_dir_objects<T>(&self) -> Result<Vec<T>, DataResolverError>
	where
		T: for<'de> Deserialize<'de>,
	{
		self.files().map(|p| self.get_object(p)).collect()
	}
	pub fn get_file_object<T>(&self) -> Result<T, DataResolverError>
	where
		T: for<'de> Deserialize<'de>,
	{
		self.get_object(self.file())
	}
	fn get_object<T>(&self, path: PathBuf) -> Result<T, DataResolverError>
	where
		T: for<'de> Deserialize<'de>,
	{
		let mut value = value_from_file(&path)?;
		let value = get_sub_value_at_address(&mut value, &self.address)?;
		let object: T = serde_yaml::from_value(value.to_owned())?;
		Ok(object)
	}
	fn index(&self) -> PathBuf {
		self.path.join("index.yml")
	}
	pub fn is_complete(&self) -> bool {
		self.address.is_empty()
	}
	pub fn iter<T>(self) -> DataPathIter<'a, T> {
		DataPathIter {
			data_path: Some(self),
			state: DataPathIterState::Dir,
			target_type: PhantomData::<T>,
		}
	}
}
