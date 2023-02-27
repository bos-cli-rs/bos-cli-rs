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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SocialDbWidgetMetadata>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbWidgetMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<SocialDbWidgetMetadataImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, Option<String>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbWidgetMetadataImage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipfs_cid: Option<String>,
}
