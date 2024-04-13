use std::collections::BTreeSet;
use unicode_segmentation::UnicodeSegmentation;
pub struct ReactionSet {
    list: Vec<String>,
    set: BTreeSet<String>,
}

impl ReactionSet {
    pub fn new() -> Self {
        ReactionSet {
            list: Vec::new(),
            set: BTreeSet::new(),
        }
    }

    pub fn add_reactions(&mut self, reactions: &[String]) {
        let single_grapheme = reactions.concat().graphemes(true).count() == 1;
        let mut map = if single_grapheme {
            BTreeSet::from([reactions.concat()])
        } else {
            reactions.iter().map(|x| x.to_owned()).collect()
        };

        if self.set.is_disjoint(&map) {
            self.set.append(&mut map);
            if single_grapheme {
                self.list.push(reactions.concat());
            } else {
                self.list.extend_from_slice(reactions);
            }
        }
    }

    pub fn as_list(&self) -> &[String] {
        &self.list
    }
}

impl Default for ReactionSet {
    fn default() -> Self {
        ReactionSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_test() {
        let set = ReactionSet::new();
        let ret: &[String] = set.as_list();
        assert_eq!(ret.concat(), "");
    }

    #[test]
    fn identity() {
        let mut set = ReactionSet::new();
        set.add_reactions(&["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(set.as_list().concat(), "abc");
    }

    #[test]
    fn no_duplicates() {
        let mut set = ReactionSet::new();
        set.add_reactions(&["a".to_string()]);
        set.add_reactions(&["a".to_string()]);
        assert_eq!(set.as_list().concat(), "a");
    }
}
