use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub data: Option<Vec<u8>>,
    pub target: Option<String>,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            data: None,
            target: None,
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

    pub fn build(self) -> eyre::Result<Self> {
        let data = self
            .clone()
            .data
            .ok_or_else(|| eyre::eyre!("data field is required"))?;
        assert_ne!(data.len(), 0);
        Ok(Self {
            data: self.data,
            target: self.target,
        })
    }
}
