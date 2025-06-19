use serde::{Deserialize, Serialize};
use url::Url;

use crate::types::secret::Secret;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct HeliconeConfig {
    pub api_key: Secret<String>,
    pub base_url: Url,
    pub websocket_url: Url,
    pub enable_control_plane: bool,
}

impl Default for HeliconeConfig {
    fn default() -> Self {
        Self {
            api_key: Secret::from(
                std::env::var("HELICONE_API_KEY")
                    .unwrap_or("sk-helicone-...".to_string()),
            ),
            base_url: "http://localhost:8585".parse().unwrap(),
            websocket_url: "ws://localhost:8585/ws/v1/router/control-plane"
                .parse()
                .unwrap(),
            enable_control_plane: true,
        }
    }
}

#[cfg(feature = "testing")]
impl crate::tests::TestDefault for HeliconeConfig {
    fn test_default() -> Self {
        Self::default()
    }
}
