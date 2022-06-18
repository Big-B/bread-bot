use std::collections::BTreeSet;
pub struct ReactionSet {
    list: String,
    set: BTreeSet<char>,
}

impl ReactionSet {
    pub fn new() -> Self {
        ReactionSet {
            list: String::new(),
            set: BTreeSet::new(),
        }
    }

    pub fn add_reactions(&mut self, reactions: &[String]) {
        let mut map = reactions.concat().chars().collect();
        if self.set.is_disjoint(&map) {
            self.set.append(&mut map);
            self.list.push_str(&reactions.concat());
        }
    }

    pub fn as_str(&self) -> &str {
        &self.list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_test() {
        let set = ReactionSet::new();
        let ret = set.as_str();
        assert_eq!(ret, "");
    }

    #[test]
    fn identity() {
        let mut set = ReactionSet::new();
        set.add_reactions(&["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(set.as_str(), "abc");
    }

    #[test]
    fn no_duplicates() {
        let mut set = ReactionSet::new();
        set.add_reactions(&["a".to_string()]);
        set.add_reactions(&["a".to_string()]);
        assert_eq!(set.as_str(), "a");
    }
}
