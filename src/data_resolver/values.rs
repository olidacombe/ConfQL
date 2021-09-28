use itertools::Itertools;
use serde_yaml::Value;
use std::path::PathBuf;

use super::DataResolverError;

pub fn get_sub_value_at_address<'a>(
	value: &'a Value,
	address: &[&str],
) -> Result<&'a Value, DataResolverError> {
	use itertools::FoldWhile::{Continue, Done};
	return address
		.iter()
		.fold_while(Ok(value), |acc, i| match acc.unwrap().get(i) {
			Some(v) => Continue(Ok(v)),
			_ => Done(Err(DataResolverError::KeyNotFound(i.to_string()))),
		})
		.into_inner();
}

pub fn value_from_file(path: &PathBuf) -> Result<Value, DataResolverError> {
	let file = std::fs::File::open(&path)?;
	let value = serde_yaml::from_reader::<_, Value>(file)?;
	Ok(value)
}

#[cfg(test)]
mod tests {
	// TODO move super::tests::Mocks to a file_mocks crate (possibly publish)
	// and use that here and above.
	// Above can wrap that to add the .resolver() method
}
