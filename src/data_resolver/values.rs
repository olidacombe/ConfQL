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

pub fn take_sub_value_at_address(
    value: &mut Value,
    address: &[&str],
) -> Result<Value, DataResolverError> {
    use itertools::FoldWhile::{Continue, Done};
    return address
        .iter()
        .fold_while(Ok(value), |acc, i| match acc.unwrap().get_mut(i) {
            Some(v) => Continue(Ok(v)),
            _ => Done(Err(DataResolverError::KeyNotFound(i.to_string()))),
        })
        .into_inner()
        .map(|v| std::mem::replace(v, Value::Null));
}

pub fn value_from_file(path: &PathBuf) -> Result<Value, DataResolverError> {
    let file = std::fs::File::open(&path)?;
    let value = serde_yaml::from_reader::<_, Value>(file)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use indoc::indoc;
    use test_files::TestFiles;
    use test_utils::yaml;

    // TODO beyond just a couple of happy tests

    #[test]
    fn gets_value_from_file() -> Result<()> {
        let filename = "index.yml";
        let content = indoc! {"
			---
			ok: true
			go: home
		"};
        let mocks = TestFiles::new().unwrap();
        mocks.file(filename, content)?;
        let file_path = mocks.path().join(filename);

        let read_value = value_from_file(&file_path)?;

        assert_eq!(serde_yaml::to_string(&read_value)?, content);
        Ok(())
    }

    #[test]
    fn gets_sub_value_at_address() -> Result<()> {
        let value = yaml! {"
            ---
            my:
                yaml:
                    is:
                    - hella
                    - deep
		"};

        assert_eq!(
            get_sub_value_at_address(&value, &["my", "yaml", "is"])?,
            &yaml! {"
        		---
        		- hella
        		- deep
        	"}
        );
        Ok(())
    }
}
