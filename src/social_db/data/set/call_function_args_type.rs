use std::str::FromStr;

use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SetContext)]
#[interactive_clap(output_context = FunctionArgsTypeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How do you want to pass the function call arguments?
pub enum FunctionArgsType {
    #[strum_discriminants(strum(
        message = "with-json        - Valid JSON arguments (e.g. {\"token_id\": \"42\"})"
    ))]
    /// Valid JSON arguments (e.g. {"token_id": "42"})
    WithJson(super::function_args::FunctionArgs),
    #[strum_discriminants(strum(message = "text-args        - Arbitrary text arguments"))]
    /// Arbitrary text arguments
    WithText(super::function_args::FunctionArgs),
    #[strum_discriminants(strum(
        message = "with-json-file   - Reading from a reusable text file"
    ))]
    /// Reading from a reusable text file
    WithJsonFile(super::function_args::FunctionArgs),
    #[strum_discriminants(strum(
        message = "with-text-file   - Reading from a reusable JSON file"
    ))]
    /// Reading from a reusable JSON file
    WithTextFile(super::function_args::FunctionArgs),
}

#[derive(Clone)]
pub struct FunctionArgsTypeContext {
    pub config: near_cli_rs::config::Config,
    pub set_to_account_id: near_cli_rs::types::account_id::AccountId,
    pub key: String,
    pub function_args_type: FunctionArgsTypeDiscriminants,
}

impl FunctionArgsTypeContext {
    pub fn from_previous_context(
        previous_context: super::SetContext,
        scope: &<FunctionArgsType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            set_to_account_id: previous_context.set_to_account_id,
            key: previous_context.key,
            function_args_type: *scope,
        })
    }
}

pub fn get_value_from_function_args(
    args: String,
    function_args_type: FunctionArgsTypeDiscriminants,
) -> color_eyre::eyre::Result<serde_json::Value> {
    match function_args_type {
        FunctionArgsTypeDiscriminants::WithJson => {
            Ok(serde_json::Value::from_str(&args).wrap_err("Data not in JSON format!")?)
        }
        FunctionArgsTypeDiscriminants::WithText => Ok(serde_json::Value::String(args)),
        FunctionArgsTypeDiscriminants::WithJsonFile => {
            let data_path = std::path::PathBuf::from(args);
            let data = std::fs::read(&data_path)
                .wrap_err_with(|| format!("Access to data file <{:?}> not found!", &data_path))?;
            Ok(serde_json::from_slice(&data).wrap_err("Data not in JSON format!")?)
        }
        FunctionArgsTypeDiscriminants::WithTextFile => {
            let data_path = std::path::PathBuf::from(args);
            let data = std::fs::read_to_string(&data_path)
                .wrap_err_with(|| format!("Access to data file <{:?}> not found!", &data_path))?;
            Ok(serde_json::Value::String(data))
        }
    }
}
