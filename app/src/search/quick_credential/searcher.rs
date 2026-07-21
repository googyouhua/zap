use crate::search::mixer::SearchMixer;

pub type QuickCredentialSearchMixer = SearchMixer<QuickCredentialSearchItemAction>;

#[derive(Clone, Debug)]
pub enum QuickCredentialSearchItemAction {
    SelectCredential(warp_quick_credential::QuickCredential),
}
