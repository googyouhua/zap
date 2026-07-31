use warpui::{Entity, SingletonEntity};

#[derive(Default)]
pub struct OneKeyCredentialsChangedNotifier {}

impl OneKeyCredentialsChangedNotifier {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone, Debug)]
pub enum OneKeyCredentialsChangedEvent {
    CredentialsChanged,
}

impl Entity for OneKeyCredentialsChangedNotifier {
    type Event = OneKeyCredentialsChangedEvent;
}

impl SingletonEntity for OneKeyCredentialsChangedNotifier {}
