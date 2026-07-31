use itertools::Itertools;
use warpui::AppContext;

use crate::search::data_source::{Query, QueryResult};
use crate::search::mixer::{DataSourceRunErrorWrapper, SyncDataSource};

use super::search_item::OneKeySearchItem;
use super::searcher::OneKeySearchItemAction;

pub struct OneKeyDataSource {
    credentials: Vec<warp_onekey::OneKeyCredential>,
}

impl OneKeyDataSource {
    pub fn new() -> Result<Self, String> {
        let credentials = warp_onekey::find_all().map_err(|e| e.to_string())?;
        Ok(Self { credentials })
    }
}

impl SyncDataSource for OneKeyDataSource {
    type Action = OneKeySearchItemAction;

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
                    OneKeySearchItem { credential }.into()
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
                    Some(OneKeySearchItem { credential }.into())
                } else {
                    None
                }
            })
            .collect_vec())
    }
}

#[cfg(test)]
pub(crate) fn filter_credentials(
    credentials: &[warp_onekey::OneKeyCredential],
    query_text: &str,
) -> Vec<warp_onekey::OneKeyCredential> {
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
#[path = "data_source_tests.rs"]
mod tests;
