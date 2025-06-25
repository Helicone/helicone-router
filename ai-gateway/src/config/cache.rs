use serde::{Deserialize, Serialize};

pub(crate) const MAX_BUCKET_SIZE: u8 = 10;
pub(crate) const DEFAULT_BUCKETS: u8 = 1;

#[derive(
    Debug, Default, Clone, Deserialize, Serialize, Eq, PartialEq, Hash,
)]
#[serde(default, rename_all = "kebab-case")]
pub struct CacheConfig {
    /// Cache-control header: <https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Cache-Control>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directive: Option<String>,
    #[serde(default = "default_buckets")]
    pub buckets: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
}

#[cfg(feature = "testing")]
impl crate::tests::TestDefault for CacheConfig {
    fn test_default() -> Self {
        Self {
            directive: None,
            buckets: DEFAULT_BUCKETS,
            seed: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum CacheStore {
    InMemory {
        // apparently container-level `rename_all` for enums doesn't
        // apply to the fields of the enum, so we need to rename the field
        // manually
        #[serde(rename = "max-size", default = "default_max_size")]
        max_size: usize,
    },
}

impl CacheStore {
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        match (self, other) {
            (
                Self::InMemory { max_size },
                Self::InMemory {
                    max_size: other_max_size,
                },
            ) => Self::InMemory {
                max_size: *max_size.max(other_max_size),
            },
        }
    }
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::InMemory {
            max_size: default_max_size(),
        }
    }
}

fn default_max_size() -> usize {
    // 256MB
    1024 * 1024 * 256
}

fn default_buckets() -> u8 {
    1
}
