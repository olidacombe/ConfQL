use std::fs;
use std::iter;
use std::path::PathBuf;

use super::values::{take_sub_value_at_address, value_from_file};
use super::DataResolverError;

enum Level {
	Dir,
	File,
}

pub struct DataPath<'a> {
	level: Level,
	path: PathBuf,
	address: &'a [&'a str],
}

impl<'a> DataPath<'a> {
	pub fn next(&mut self) {
		use Level::{Dir, File};
		match &self.level {
			File => {
				self.level = Dir;
			}
			Dir => {
				if let Some((head, tail)) = self.address.split_first() {
					self.path.push(head);
					self.address = tail;
					self.level = File;
				}
			}
		}
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
	pub fn join(&self, tail: &'a str) -> Self {
		Self {
			level: Level::File,
			path: self.path.join(tail),
			address: self.address,
		}
	}
	pub fn value(&self) -> serde_yaml::Value {
		use Level::{Dir, File};
		match &self.level {
			Dir => self.get_value(&self.index()),
			File => self.get_value(&self.file()),
		}
		.unwrap_or(serde_yaml::Value::Null)
	}
	pub fn values(&self) -> serde_yaml::Value {
		if self.done() {
			self.files()
				.filter_map(|f| self.get_value(&f).ok())
				.collect()
		} else {
			self.value()
		}
	}
	fn get_value(&self, path: &PathBuf) -> Result<serde_yaml::Value, DataResolverError> {
		let mut value = value_from_file(&path)?;
		Ok(take_sub_value_at_address(&mut value, &self.address)?)
	}
	fn index(&self) -> PathBuf {
		self.path.join("index.yml")
	}
	pub fn done(&self) -> bool {
		use Level::{Dir, File};
		match &self.level {
			File => false,
			Dir => self.address.is_empty(),
		}
	}
}

trait Merge {
	fn merge(&mut self, mergee: Self) -> Result<&mut Self, DataResolverError>;
	fn merge_at(&mut self, key: &str, mergee: Self) -> Result<&mut Self, DataResolverError>;
}

macro_rules! merge_compat_err {
	($self:expr, $mergee:expr) => {
		Err(DataResolverError::IncompatibleYamlMerge {
			dst: $self.clone(),
			src: $mergee,
		})
	};
}

impl Merge for serde_yaml::Value {
	fn merge(&mut self, mut mergee: Self) -> Result<&mut Self, DataResolverError> {
		use serde_yaml::Value::{Bool, Mapping, Null, Number, Sequence, String};
		if let Null = mergee {
			return Ok(self);
		}
		match self {
			Null => {
				*self = mergee;
			}
			Bool(_) => {
				if !mergee.is_bool() {
					return merge_compat_err! {self, mergee};
				}
				*self = mergee;
			}
			Number(_) => {
				if !mergee.is_number() {
					return merge_compat_err! {self, mergee};
				}
				*self = mergee;
			}
			String(_) => {
				if !mergee.is_string() {
					return merge_compat_err! {self, mergee};
				}
				*self = mergee;
			}
			Sequence(list) => {
				if let Sequence(ref mut appendee) = mergee {
					list.append(appendee);
				} else {
					return merge_compat_err! {self, mergee};
				}
			}
			Mapping(mapping) => {
				if let Mapping(superimposee) = mergee {
					for (key, src) in superimposee {
						if let Some(dst) = mapping.get_mut(&key) {
							dst.merge(src)?;
						} else {
							mapping.insert(key, src);
						}
					}
				} else {
					return merge_compat_err! {self, mergee};
				}
			}
		};
		Ok(self)
	}
	fn merge_at(&mut self, key: &str, mergee: Self) -> Result<&mut Self, DataResolverError> {
		if let Self::Mapping(mapping) = self {
			let key: Self = key.into();
			match mapping.get_mut(&key) {
				Some(value) => {
					value.merge(mergee)?;
				}
				None => {
					mapping.insert(key, mergee);
				}
			}
			Ok(self)
		} else {
			Err(DataResolverError::CannotMergeIntoNonMapping(self.clone()))
		}
	}
}

trait ResolveValue {
	fn merge_properties(
		_value: &mut serde_yaml::Value,
		_data_path: &DataPath,
	) -> Result<(), DataResolverError> {
		Ok(())
	}
	fn resolve_value(mut data_path: DataPath) -> Result<serde_yaml::Value, DataResolverError> {
		let mut value = serde_yaml::Value::Null;
		if data_path.done() {
			Self::merge_properties(&mut value, &data_path)?;
		} else {
			value = data_path.value();
			data_path.next();
			value.merge(Self::resolve_value(data_path)?)?;
		}
		Ok(value)
	}
	fn resolve_values(mut data_path: DataPath) -> Result<serde_yaml::Value, DataResolverError> {
		let mut value = data_path.values();
		if !data_path.done() {
			data_path.next();
			value.merge(Self::resolve_values(data_path)?)?;
		}
		Ok(value)
	}
}

impl ResolveValue for bool {}
impl ResolveValue for f64 {}
// TODO?
// impl ResolveValue for juniper::ID {}
impl ResolveValue for String {}
impl ResolveValue for u32 {}

// TODO macro generates the below automatically for
// such types

struct MyObj {
	id: u32,
	name: String,
}

impl ResolveValue for MyObj {
	fn merge_properties(
		value: &mut serde_yaml::Value,
		data_path: &DataPath,
	) -> Result<(), DataResolverError> {
		value.merge_at("id", u32::resolve_value(data_path.join("id"))?)?;
		value.merge_at("name", String::resolve_values(data_path.join("name"))?)?;
		Ok(())
	}
}

struct MyOtherObj {
	id: u32,
	alias: String,
}

impl ResolveValue for MyOtherObj {
	fn merge_properties(
		value: &mut serde_yaml::Value,
		data_path: &DataPath,
	) -> Result<(), DataResolverError> {
		value.merge_at("id", u32::resolve_value(data_path.join("id"))?)?;
		value.merge_at("alias", String::resolve_values(data_path.join("alias"))?)?;
		Ok(())
	}
}

struct Query {
	my_obj: MyObj,
	my_list: Vec<MyOtherObj>,
}

impl ResolveValue for Query {
	fn merge_properties(
		value: &mut serde_yaml::Value,
		data_path: &DataPath,
	) -> Result<(), DataResolverError> {
		value.merge_at("my_obj", MyObj::resolve_value(data_path.join("my_obj"))?)?;
		value.merge_at(
			"my_list",
			MyOtherObj::resolve_values(data_path.join("my_list"))?,
		)?;
		Ok(())
	}
}
