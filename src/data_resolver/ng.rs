use serde::Deserialize;
use std::fs;
use std::iter;
use std::marker::PhantomData;
use std::path::PathBuf;

use super::values::{get_sub_value_at_address, value_from_file};
use super::DataResolverError;

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
	pub fn done(&self) -> bool {
		self.address.is_empty()
	}
}

trait Merge {
	fn merge(&mut self, mergee: Self);
	fn merge_at(&mut self, key: &str, mergee: Self);
}

trait ResolveValue {
	fn resolve_value(data_path: DataPath) -> serde_yaml::Value;
	fn resolve_values(data_path: DataPath) -> serde_yaml::Value;
}

struct MyObj {
	id: u32,
	name: String,
}

struct MyOtherObj {
	id: u32,
	alias: String,
}

struct Query {
	my_obj: MyObj,
	my_list: Vec<MyOtherObj>,
}

impl ResolveValue for Query {
	fn resolve_value(data_path: DataPath) -> serde_yaml::Value {
		let mut value = serde_yaml::Value::Null;
		if data_path.done() {
			value.merge_at("my_obj", MyObj.resolve_value(data_path.join("my_obj")));
			value.merge_at("my_list", MyObj.resolve_values(data_path.join("my_list")));
		} else {
			value = data_path.value();
			data_path.next();
			value.merge(Self::resolve_value(data_path));
		}
		value
	}
	fn resolve_values(data_path: DataPath) -> serde_yaml::Value {
		let value = serde_yaml::Value::Null;
		match data_path.dir_data_paths() {
			Some(data_paths) => {
				value.merge(data_paths.map(Self::resolve_value));
			}
			None => {
				value = data_path.value();
				data_path.next();
				value.merge(Self::resolve_value(data_path));
			}
		}
		value
	}
}
