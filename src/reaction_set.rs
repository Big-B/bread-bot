use std::collections::BTreeSet;
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

    pub fn add_reactions(&mut self, reactions: &mut Vec<String>) {
        let mut map = reactions.clone().into_iter().collect();
        if self.set.is_disjoint(&map) {
            self.set.append(&mut map);
            self.list.append(reactions);
        }
    }

    pub fn get_reaction_str(&self) -> String {
        self.list.concat()
    }
}
