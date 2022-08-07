use std::collections::HashMap;

type Filter<T, E> = Box<dyn Fn(T) -> Result<(), E>>;

struct Filters<'a, T, E> {
    filter: Option<Filter<T, E>>,
    children: HashMap<&'a str, Filters<'a, T, E>>,
}

impl<'a, T, E> Filters<'a, T, E>
where
    T: Sized,
{
    pub fn remove(&mut self, k: &str) -> Option<Self> {
        self.remove(k)
    }

    pub fn apply(&self, to: T) -> Result<(), E> {
        match &self.filter {
            None => Ok(()),
            Some(f) => f(to),
        }
    }

    pub fn new(filter: Filter<T, E>) -> Self {
        Self {
            filter: Some(filter),
            children: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::{eyre::eyre, Result};

    #[test]
    fn apply_success() -> Result<()> {
        let f = Box::new(|_: u32| -> Result<()> { Ok(()) });
        let filters = Filters::new(f);
        filters.apply(1)?;
        Ok(())
    }

    #[test]
    fn apply_fail() {
        let f = Box::new(|_: u32| -> Result<()> { Err(eyre!("oh dear")) });
        let filters = Filters::new(f);
        assert!(filters.apply(1).is_err());
    }
}
