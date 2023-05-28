use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod with_json;
mod with_json_file;
mod with_text;
mod with_text_file;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::SetContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to pass the data to set to the key?
pub enum DataType {
    #[strum_discriminants(strum(
        message = "with-json        - Valid JSON arguments (e.g. {\"token_id\": \"42\"})"
    ))]
    /// Valid JSON arguments (e.g. {"token_id": "42"})
    WithJson(self::with_json::JsonData),
    #[strum_discriminants(strum(message = "with-text        - Arbitrary text arguments"))]
    /// Arbitrary text arguments
    WithText(self::with_text::TextData),
    #[strum_discriminants(strum(
        message = "with-json-file   - Reading from a reusable text file"
    ))]
    /// Reading from a reusable text file
    WithJsonFile(self::with_json_file::JsonDataFile),
    #[strum_discriminants(strum(
        message = "with-text-file   - Reading from a reusable JSON file"
    ))]
    /// Reading from a reusable JSON file
    WithTextFile(self::with_text_file::TextDataFile),
}

#[derive(Clone)]
pub struct DataContext {
    pub config: near_cli_rs::config::Config,
    pub set_to_account_id: near_cli_rs::types::account_id::AccountId,
    pub key: String,
    pub value: serde_json::Value,
}
