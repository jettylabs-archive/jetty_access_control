//! Functionality for the tableau write path

use crate::TableauConnector;

mod default_policies;
mod groups;
mod policies;
mod users;

#[derive(Default)]
pub(crate) struct PrioritizedPlans(
    pub(crate) Vec<String>,
    pub(crate) Vec<String>,
    pub(crate) Vec<String>,
);

impl PrioritizedPlans {
    pub(crate) fn extend(&mut self, other: &Self) {
        self.0.extend(other.0.clone());
        self.1.extend(other.1.clone());
        self.2.extend(other.2.clone());
    }
    pub(crate) fn flatten(&self) -> Vec<String> {
        [self.0.to_owned(), self.1.to_owned(), self.2.to_owned()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    }
}
