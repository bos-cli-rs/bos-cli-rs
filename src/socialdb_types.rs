use std::collections::HashMap;

pub type ComponentName = String;

pub type KeyName = String;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SocialDbQuery {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDb {
    #[serde(flatten)]
    pub accounts: HashMap<near_primitives::types::AccountId, SocialDbComponentKey>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDbComponentKey {
    #[serde(flatten)]
    pub key: HashMap<KeyName, SocialDbAccountMetadata>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDbAccountMetadata {
    #[serde(flatten)]
    pub components: HashMap<ComponentName, SocialDbComponent>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum SocialDbComponent {
    Code(String),
    CodeWithMetadata {
        #[serde(rename = "")]
        code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<SocialDbComponentMetadata>,
    },
}

impl SocialDbComponent {
    pub fn code(&self) -> &str {
        match self {
            Self::Code(code) => code,
            Self::CodeWithMetadata { code, .. } => code,
        }
    }

    pub fn metadata(&self) -> Option<&SocialDbComponentMetadata> {
        match self {
            Self::Code(_) => None,
            Self::CodeWithMetadata { metadata, .. } => metadata.as_ref(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbComponentMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<SocialDbComponentMetadataImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, Option<String>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbComponentMetadataImage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipfs_cid: Option<String>,
}
