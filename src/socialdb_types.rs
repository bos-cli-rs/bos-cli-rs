use std::collections::HashMap;

pub type WidgetName = String;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SocialDbQuery {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDb {
    #[serde(flatten)]
    pub accounts: HashMap<near_primitives::types::AccountId, SocialDbAccountMetadata>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDbAccountMetadata {
    #[serde(rename = "widget")]
    pub widgets: HashMap<WidgetName, SocialDbWidget>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDbWidget {
    #[serde(rename = "")]
    pub code: String,
    pub metadata: Option<SocialDbWidgetMetadata>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbWidgetMetadata {
    pub description: Option<String>,
    pub image: Option<SocialDbWidgetMetadataImage>,
    pub name: Option<String>,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbWidgetMetadataImage {
    pub url: Option<String>,
    pub ipfs_cid: Option<String>,
}
