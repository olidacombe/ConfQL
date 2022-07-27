use std::collections::HashMap;
use std::iter::FromIterator;
use std::rc::Rc;

use super::values::Value;

type Filter<'a> = Rc<dyn Fn(&'a Value) -> bool>;
pub type Filters<'a> = HashMap<&'a [&'a str], Filter<'a>>;

pub trait FilterMap {
    /// * Evicts entries whose address does not begin with `head`
    /// * Pops head from remaining entries, since we know we won't be interested in that filter again
    fn descend(&mut self, head: &str);
}

impl<'a> FilterMap for Filters<'a> {
    fn descend(&mut self, head: &str) {
        // todo: this in one pass
        self.retain(|k, _| k.first() == Some(&head));
        let mut new = Self::from_iter(self.into_iter().filter_map(|(k, v)| {
            k.split_first()
                .map(|(_, tail)| tail)
                .map(|tail| (tail, v.clone())) // should this be some kind of .take() instead of clone()?
        }));
        std::mem::swap(self, &mut new);
    }
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
