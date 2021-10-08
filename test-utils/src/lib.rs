#[macro_export]
macro_rules! yaml {
    ($e:literal) => {{
        use indoc::indoc;
        use serde_yaml;
        serde_yaml::from_str::<serde_yaml::Value>(indoc! {$e}).unwrap()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn yaml_parses() -> Result<()> {
        yaml! {"
            ---
            my:
                yaml:
                    is:
                    - hella
                    - deep
        "};
        Ok(())
    }
}
