use crate::search::mixer::SearchMixer;

pub type OneKeySearchMixer = SearchMixer<OneKeySearchItemAction>;

#[derive(Clone, Debug)]
pub enum OneKeySearchItemAction {
    SelectCredential(warp_onekey::OneKeyCredential),
}
