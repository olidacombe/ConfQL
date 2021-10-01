use serde::Deserialize;
use std::fs;
use std::iter;
use std::marker::PhantomData;
use std::path::PathBuf;

use super::values::{get_sub_value_at_address, value_from_file};
use super::DataResolverError;

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
					// if data_path.is_complete() {
					// 	if let Some(val) = self.get_leaf_value() {
					// 	} else {
					// 		self.data_path = None;
					// 	}
					// } else {
					if let Ok(val) = data_path.get_file_object() {
						return Some(val);
					}
					// }
				}
				Dir => {
					self.state = File;
					if let Ok(val) = data_path.get_dir_object() {
						return Some(val);
					}
					if let None = data_path.next() {
						self.data_path = None;
					}
				}
			}
			// match self.file_iterator.next() {
			// 	Some(path) => {
			// 		if let Ok(object) = self.get_object(&path) {
			// 			return Some(object);
			// 		}
			// 	}
			// 	None => match data_path.next() {
			// 		Some(data_path) => {
			// 			self.file_iterator = data_path.files();
			// 		}
			// 		None => self.data_path = None,
			// 	},
			// }
		}
		None
	}
}

// impl<'a, T> Iterator for DataPathIter<'a, T>
// where
// 	T: for<'de> Deserialize<'de>,
// {
// 	type Item = T;

// 	fn next(&mut self) -> Option<Self::Item> {
// 		while let Some(ref mut data_path) = self.data_path {
// 			match self.file_iterator.next() {
// 				Some(path) => {
// 					if let Ok(object) = self.get_object(&path) {
// 						return Some(object);
// 					}
// 				}
// 				None => match data_path.next() {
// 					Some(data_path) => {
// 						self.file_iterator = data_path.files();
// 					}
// 					None => self.data_path = None,
// 				},
// 			}
// 		}
// 		None
// 	}
// }

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
		let value = value_from_file(&path)?;
		let value = get_sub_value_at_address(&value, &self.address)?;
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
