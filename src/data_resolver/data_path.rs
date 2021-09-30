use serde::Deserialize;
use serde_yaml::Value;
use std::fs;
use std::iter;
use std::marker::PhantomData;
use std::path::PathBuf;

use super::values::{get_sub_value_at_address, value_from_file};
use super::DataResolverError;

enum DataPathIterMultiness {
	Many,
	One,
}

enum DataPathIterState {
	Dir,
	DirFiles,
	File,
}

struct DataPathIter<'a, T> {
	data_path: Option<DataPath<'a>>,
	file_iterator: Box<dyn Iterator<Item = PathBuf> + 'a>,
	multiness: DataPathIterMultiness,
	state: DataPathIterState,
	target_type: PhantomData<T>,
}

impl<'a, T> Iterator for DataPathIter<'a, T>
where
	T: for<'de> Deserialize<'de>,
{
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(ref mut data_path) = self.data_path {
			match self.file_iterator.next() {
				Some(path) => {
					if let Ok(object) = self.get_object(&path) {
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

struct DataPath<'a> {
	path: PathBuf,
	address: &'a [&'a str],
}

impl<'a> DataPath<'a> {
	pub fn next(&mut self) -> Option<&Self> {
		self.address.split_first().map(move |(head, tail)| {
			self.path.push(head);
			self.address = tail;
			&*self
		})
	}
	fn empty_iter() -> Box<dyn Iterator<Item = PathBuf> + 'a> {
		Box::new(iter::empty::<PathBuf>())
	}
	pub fn file(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
		Box::new(iter::once(self.path.with_extension("yml")))
	}
	pub fn files(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
		match fs::read_dir(&self.path) {
			Ok(reader) => Box::new(
				reader
					.filter_map(|dir_entry| dir_entry.ok())
					.map(|dir_entry| dir_entry.path()),
			),
			_ => Box::new(iter::empty::<PathBuf>()),
		}
	}
	fn get_object<T>(&self, path: PathBuf) -> Result<T, DataResolverError>
	where
		T: for<'de> Deserialize<'de>,
	{
		let value = value_from_file(&path)?;
		let value = get_sub_value_at_address(&value, &self.address)?;
		let object: T = serde_yaml::from_value(value.to_owned())?;
		Ok(object)
	}
	pub fn index(&self) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
		Box::new(iter::once(self.path.join("index.yml")))
	}
	pub fn is_complete(&self) -> bool {
		self.address.is_empty()
	}
	pub fn many_iter<T>(self) -> DataPathIter<'a, T> {
		DataPathIter {
			data_path: Some(self),
			file_iterator: Self::empty_iter(),
			multiness: DataPathIterMultiness::Many,
			state: DataPathIterState::Dir,
			target_type: PhantomData::<T>,
		}
	}
	pub fn one_iter<T>(self) -> DataPathIter<'a, T> {
		DataPathIter {
			data_path: Some(self),
			file_iterator: Self::empty_iter(),
			multiness: DataPathIterMultiness::One,
			state: DataPathIterState::Dir,
			target_type: PhantomData::<T>,
		}
	}
}
