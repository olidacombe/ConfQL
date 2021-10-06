use std::fs;
use std::iter;
use std::path::PathBuf;

use super::values::{take_sub_value_at_address, value_from_file, Merge};
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

#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::Result;
	use indoc::indoc;
	use test_files::TestFiles;
	use test_utils::yaml;

	trait GetDataPath<'a> {
		fn data_path(&self, address: &'a [&'a str]) -> DataPath<'a>;
	}

	impl<'a> GetDataPath<'a> for TestFiles {
		fn data_path(&self, address: &'a [&'a str]) -> DataPath<'a> {
			DataPath {
				level: Level::Dir,
				path: self.path().to_path_buf(),
				address,
			}
		}
	}
	#[test]
	fn resolves_num_at_root() -> Result<()> {
		let mocks = TestFiles::new().unwrap();
		mocks.file(
			"index.yml",
			indoc! {"
                ---
                3
            "},
		)?;
		let v = mocks.data_path(&[]).value();
		assert_eq!(v, yaml! {"3"});
		Ok(())
	}

	// #[test]
	// fn resolves_non_nullable_int_deeper() -> Result<()> {
	// 	let mocks = TestFiles::new().unwrap();
	// 	mocks.file(
	// 		"index.yml",
	// 		indoc! {"
	//             ---
	//             a:
	//                 b:
	//                     c: 3
	//         "},
	// 	)?;
	// 	let i: u32 = mocks.resolver().get_non_nullable(&["a", "b", "c"])?;
	// 	assert_eq!(i, 3);
	// 	Ok(())
	// }

	// #[test]
	// fn resolves_non_nullable_int() -> Result<()> {
	// 	let test_cases = [
	// 		[
	// 			"a/b/c.yml",
	// 			indoc! {"
	//                 ---
	//                 3
	//             "},
	// 		],
	// 		[
	// 			"a/b/c/index.yml",
	// 			indoc! {"
	//                 ---
	//                 3
	//             "},
	// 		],
	// 		[
	// 			"a/b/index.yml",
	// 			indoc! {"
	//                 ---
	//                 c: 3
	//             "},
	// 		],
	// 		[
	// 			"a/b.yml",
	// 			indoc! {"
	//                 ---
	//                 c: 3
	//             "},
	// 		],
	// 		[
	// 			"a.yml",
	// 			indoc! {"
	//                 ---
	//                 b:
	//                     c: 3
	//             "},
	// 		],
	// 		[
	// 			"index.yml",
	// 			indoc! {"
	//                 ---
	//                 a:
	//                     b:
	//                         c: 3
	//             "},
	// 		],
	// 	];

	// 	for [file, content] in test_cases {
	// 		let mocks = TestFiles::new().unwrap();
	// 		mocks.file(file, content)?;
	// 		let i: u32 = mocks.resolver().get_non_nullable(&["a", "b", "c"])?;
	// 		assert_eq!(i, 3);
	// 	}
	// 	Ok(())
	// }

	// #[test]
	// fn resolves_non_nullable_list_int_at_root() -> Result<()> {
	// 	let mocks = TestFiles::new().unwrap();
	// 	mocks.file(
	// 		"index.yml",
	// 		indoc! {"
	//             ---
	//             - 1
	//             - 2
	//             - 3
	//         "},
	// 	)?;
	// 	let v: Vec<u32> = mocks.resolver().get_non_nullable(&[])?;
	// 	assert_eq!(v, vec![1, 2, 3]);
	// 	Ok(())
	// }

	// #[test]
	// fn resolves_non_nullable_list_int_at_index() -> Result<()> {
	// 	let mocks = TestFiles::new().unwrap();
	// 	mocks.file(
	// 		"index.yml",
	// 		indoc! {"
	//             ---
	//             a:
	//             - 4
	//             - 5
	//             - 6
	//         "},
	// 	)?;
	// 	let v: Vec<u32> = mocks.resolver().get_non_nullable(&["a"])?;
	// 	assert_eq!(v, vec![4, 5, 6]);
	// 	Ok(())
	// }

	// #[test]
	// fn resolves_non_nullable_list_int_at_root_files() -> Result<()> {
	// 	let mocks = TestFiles::new().unwrap();
	// 	mocks
	// 		.file(
	// 			"a.yml",
	// 			indoc! {"
	//             ---
	//             1
	//         "},
	// 		)?
	// 		.file(
	// 			"b.yml",
	// 			indoc! {"
	//             ---
	//             2
	//         "},
	// 		)?;
	// 	let mut v: Vec<u32> = mocks.resolver().get_non_nullable(&[])?;
	// 	v.sort();
	// 	assert_eq!(v, vec![1, 2]);
	// 	Ok(())
	// }

	// #[test]
	// fn resolves_non_nullable_list_at_bottom_files() -> Result<()> {
	// 	let mocks = TestFiles::new().unwrap();
	// 	mocks
	// 		.file(
	// 			"a/b.yml",
	// 			indoc! {"
	//             ---
	//             1
	//         "},
	// 		)?
	// 		.file(
	// 			"a/c.yml",
	// 			indoc! {"
	//             ---
	//             2
	//         "},
	// 		)?;
	// 	let mut v: Vec<u32> = mocks.resolver().get_non_nullable(&["a"])?;
	// 	v.sort();
	// 	assert_eq!(v, vec![1, 2]);
	// 	Ok(())
	// }
}
