use crate::utils::constants::TAGS_SIZE_LIMIT;
use crate::utils::core::tags::Tag;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub data: Option<Vec<u8>>,
    pub target: Option<String>,
    pub tags: Option<Vec<Tag>>,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            data: None,
            target: None,
            tags: None,
        }
    }

    pub fn data(mut self, data: Option<Vec<u8>>) -> Self {
        self.data = data;
        self
    }

    pub fn target(mut self, target: Option<String>) -> Self {
        self.target = target;
        self
    }

    pub fn tags(mut self, tags: Option<Vec<Tag>>) -> Self {
        self.tags = tags;
        self
    }

    pub fn build(self) -> eyre::Result<Self> {
        let data = self
            .clone()
            .data
            .ok_or_else(|| eyre::eyre!("data field is required"))?;

        let tags = self.clone().tags.unwrap_or_default();
        // assert data existence
        assert_ne!(data.len(), 0);
        // assert Tags vector max size
        assert!(serde_json::to_string(&tags)?.len() <= TAGS_SIZE_LIMIT);

        Ok(Self {
            data: self.data,
            target: self.target,
            tags: self.tags,
        })
    }
}
