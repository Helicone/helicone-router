use chrono::{DateTime, Utc};

use super::types::{Ack, Config, Key, MessageTypeRX, Status, Update};
const MAX_HISTORY_SIZE: usize = 100;

#[derive(Debug, Default)]
pub struct ControlPlaneState {
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub config: Config,

    // used mainly for debugging and testing, can remove later
    pub history: Vec<MessageTypeRX>,
}

impl ControlPlaneState {
    pub fn new() -> Self {
        Self {
            last_heartbeat: None,
            config: Config::default(),
            history: Vec::new(),
        }
    }
    pub fn update(&mut self, m: MessageTypeRX) {
        self.history.push(m.clone());
        if self.history.len() > MAX_HISTORY_SIZE {
            self.history.remove(0);
        }

        match m {
            MessageTypeRX::Config { data: m } => (),
            MessageTypeRX::Update(Update::Keys { data }) => {
                self.config.keys = data;
            }
            MessageTypeRX::Update(Update::AuthConfig { data }) => {
                self.config.auth = data;
            }
            MessageTypeRX::Update(Update::Config { data }) => {
                self.config = data;
            }

            /// we don't care about these for updating state, probably want to
            /// only have this function accept update in the future
            MessageTypeRX::Ack(_) => (),
            MessageTypeRX::Message { .. } => (),
        }
    }
}
