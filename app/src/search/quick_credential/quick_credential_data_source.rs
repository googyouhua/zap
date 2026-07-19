use itertools::Itertools;
use warpui::AppContext;

use crate::search::data_source::{Query, QueryResult};
use crate::search::mixer::{DataSourceRunErrorWrapper, SyncDataSource};

use super::quick_credential_search_item::QuickCredentialSearchItem;
use super::searcher::QuickCredentialSearchItemAction;

pub struct QuickCredentialDataSource {
    credentials: Vec<warp_quick_credential::QuickCredential>,
}

impl QuickCredentialDataSource {
    pub fn new() -> Result<Self, String> {
        let credentials = warp_quick_credential::find_all().map_err(|e| e.to_string())?;
        Ok(Self { credentials })
    }
}

impl SyncDataSource for QuickCredentialDataSource {
    type Action = QuickCredentialSearchItemAction;

    fn run_query(
        &self,
        query: &Query,
        _app: &AppContext,
    ) -> Result<Vec<QueryResult<Self::Action>>, DataSourceRunErrorWrapper> {
        let query_str = query.text.as_str();
        if query_str.is_empty() {
            return Ok(self
                .credentials
                .clone()
                .into_iter()
                .map(|credential| {
                    QuickCredentialSearchItem { credential }.into()
                })
                .collect_vec());
        }

        Ok(self
            .credentials
            .clone()
            .into_iter()
            .filter_map(|credential| {
                let label_match =
                    fuzzy_match::match_indices_case_insensitive(&credential.label, query_str);
                let username_match =
                    fuzzy_match::match_indices_case_insensitive(&credential.username, query_str);
                if label_match.is_some() || username_match.is_some() {
                    Some(QuickCredentialSearchItem { credential }.into())
                } else {
                    None
                }
            })
            .collect_vec())
    }
}

#[cfg(test)]
pub(crate) fn filter_credentials(
    credentials: &[warp_quick_credential::QuickCredential],
    query_text: &str,
) -> Vec<warp_quick_credential::QuickCredential> {
    if query_text.is_empty() {
        return credentials.to_vec();
    }
    credentials
        .iter()
        .filter(|c| {
            fuzzy_match::match_indices_case_insensitive(&c.label, query_text).is_some()
                || fuzzy_match::match_indices_case_insensitive(&c.username, query_text).is_some()
        })
        .cloned()
        .collect()
}

#[cfg(test)]
#[path = "quick_credential_data_source_tests.rs"]
mod tests;
