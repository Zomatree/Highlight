use indexmap::IndexSet;
use stoat_models::v0::Interactions;

pub trait InteractionsExt {
    fn reactions<I: Into<IndexSet<String>>>(self, reactions: I) -> Self;
    fn restrict_reactions(self, restrict_reactions: bool) -> Self;
}

impl InteractionsExt for Interactions {
    fn reactions<I: Into<IndexSet<String>>>(mut self, reactions: I) -> Self {
        self.reactions = Some(reactions.into());
        self
    }

    fn restrict_reactions(mut self, restrict_reactions: bool) -> Self {
        self.restrict_reactions = restrict_reactions;
        self
    }
}
