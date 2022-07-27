use std::collections::HashMap;
use std::rc::Rc;

use super::values::{Value, ValueFilter};
use super::DataResolverError;

type Filter<'a> = Rc<dyn Fn(&'a Value) -> bool>;
pub type Filters<'a> = HashMap<&'a [&'a str], Filter<'a>>;

pub trait FilterMap {
    /// * Evicts entries whose address does not begin with `head`
    /// * Pops head from remaining entries, since we know we won't be interested in that filter again
    fn descend(&mut self, head: &str);
}

impl<'a> FilterMap for Filters<'a> {
    fn descend(&mut self, head: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_by_head() {
        let mut filters = Filters::new();
        filters.insert(&["a", "b"], Rc::new(|_| false));

        assert!(!filters.is_empty());
        filters.descend("c");
        assert!(filters.is_empty());
    }

    #[test]
    fn pops_head() {
        let mut filters = Filters::new();
        filters.insert(&["a", "b"], Rc::new(|_| false));

        filters.descend("a");
        let expected_key: &[&str] = &["b"];
        assert!(filters.contains_key(expected_key));
    }
}
