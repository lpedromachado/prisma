//! Simple wrapper for WriteQueries

use crate::{builders::utils, BuilderExt, ReadQuery, SingleBuilder};
use connector::mutaction::{
    DatabaseMutactionResult as MutationResult, Identifier, TopLevelDatabaseMutaction as RootMutation,
};
use graphql_parser::query::Field;
use prisma_models::ModelRef;
use std::sync::Arc;

/// A top-level write query (mutation)
#[derive(Debug, Clone)]
pub struct WriteQuery {
    /// The actual mutation object being built
    pub inner: RootMutation,

    /// The name of the WriteQuery
    pub name: String,

    /// Required to create following ReadQuery
    pub field: Field,
}

impl WriteQuery {
    pub fn model(&self) -> ModelRef {
        match self.inner {
            RootMutation::CreateNode(ref node) => Arc::clone(&node.model),
            RootMutation::UpdateNode(ref node) => node.where_.field.model.upgrade().unwrap(),
            RootMutation::DeleteNode(ref node) => node.where_.field.model.upgrade().unwrap(),
            RootMutation::UpsertNode(ref node) => node.where_.field.model.upgrade().unwrap(),
            RootMutation::UpdateNodes(ref nodes) => Arc::clone(&nodes.model),
            RootMutation::DeleteNodes(ref nodes) => Arc::clone(&nodes.model),
            _ => unimplemented!(),
        }
    }

    /// This function generates a pre-fetch `ReadQuery` for appropriate `WriteQuery` types
    pub fn generate_prefetch(&self) -> Option<ReadQuery> {
        match self.inner {
            RootMutation::DeleteNode(_) => SingleBuilder::new()
                .setup(self.model(), &self.field)
                .build()
                .ok()
                .map(|q| ReadQuery::RecordQuery(q)),
            _ => None,
        }
    }

    /// Generate a `ReadQuery` from the encapsulated `WriteQuery`
    pub fn generate_read(&self, res: MutationResult) -> Option<ReadQuery> {
        let field = match res.identifier {
            Identifier::Id(gql_id) => dbg!(utils::derive_field(&self.field, self.model(), gql_id, &self.name)),
            Identifier::Count(_) => return None, // FIXME: We need to communicate count!
            _ => unimplemented!(),
        };

        match self.inner {
            // We ignore Deletes because they were already handled
            RootMutation::DeleteNode(_) | RootMutation::DeleteNodes(_) => None,
            RootMutation::CreateNode(_) => SingleBuilder::new()
                .setup(self.model(), &field)
                .build()
                .ok()
                .map(|q| ReadQuery::RecordQuery(q)),

            RootMutation::UpdateNode(_) => SingleBuilder::new()
                .setup(self.model(), &field)
                .build()
                .ok()
                .map(|q| ReadQuery::RecordQuery(q)),
            _ => unimplemented!(),
        }
    }
}
