use std::str::FromStr;

use color_eyre::eyre::Context;
use inquire::Select;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

#[derive(Debug, EnumDiscriminants, Clone, clap::ValueEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How do you want to pass the function call arguments?
pub enum FunctionArgsType {
    #[strum_discriminants(strum(
        message = "with-json        - Valid JSON arguments (e.g. {\"token_id\": \"42\"})"
    ))]
    /// Valid JSON arguments (e.g. {"token_id": "42"})
    WithJson,
    #[strum_discriminants(strum(message = "text-args        - Arbitrary text arguments"))]
    /// Arbitrary text arguments
    WithText,
    #[strum_discriminants(strum(
        message = "with-json-file   - Reading from a reusable text file"
    ))]
    /// Reading from a reusable text file
    WithJsonFile,
    #[strum_discriminants(strum(
        message = "with-text-file   - Reading from a reusable JSON file"
    ))]
    /// Reading from a reusable JSON file
    WithTextFile,
}

impl interactive_clap::ToCli for FunctionArgsType {
    type CliVariant = FunctionArgsType;
}

impl std::str::FromStr for FunctionArgsType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "with-json" => Ok(Self::WithJson),
            "with-text" => Ok(Self::WithText),
            "with-json-file" => Ok(Self::WithJsonFile),
            "with-text-file" => Ok(Self::WithTextFile),
            _ => Err("FunctionArgsType: incorrect value entered".to_string()),
        }
    }
}

impl std::fmt::Display for FunctionArgsType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::WithJson => write!(f, "with-json"),
            Self::WithText => write!(f, "with-text"),
            Self::WithJsonFile => write!(f, "with-json-file"),
            Self::WithTextFile => write!(f, "with-text-file"),
        }
    }
}

impl std::fmt::Display for FunctionArgsTypeDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::WithJson => write!(f, "with-json"),
            Self::WithText => write!(f, "with-text"),
            Self::WithJsonFile => write!(f, "with-json-file"),
            Self::WithTextFile => write!(f, "with-text-file"),
        }
    }
}

pub fn input_function_args_type() -> color_eyre::eyre::Result<Option<FunctionArgsType>> {
    let variants = FunctionArgsTypeDiscriminants::iter().collect::<Vec<_>>();
    let selected = Select::new("How would you like to proceed?", variants).prompt()?;
    match selected {
        FunctionArgsTypeDiscriminants::WithJson => Ok(Some(FunctionArgsType::WithJson)),
        FunctionArgsTypeDiscriminants::WithText => Ok(Some(FunctionArgsType::WithText)),
        FunctionArgsTypeDiscriminants::WithJsonFile => Ok(Some(FunctionArgsType::WithJsonFile)),
        FunctionArgsTypeDiscriminants::WithTextFile => Ok(Some(FunctionArgsType::WithTextFile)),
    }
}

pub fn function_args(
    args: String,
    function_args_type: FunctionArgsType,
) -> color_eyre::eyre::Result<serde_json::Value> {
    match function_args_type {
        super::call_function_args_type::FunctionArgsType::WithJson => {
            Ok(serde_json::Value::from_str(&args).wrap_err("Data not in JSON format!")?)
        }
        super::call_function_args_type::FunctionArgsType::WithText => {
            Ok(serde_json::Value::String(args))
        }
        super::call_function_args_type::FunctionArgsType::WithJsonFile => {
            let data_path = std::path::PathBuf::from(args);
            let data = std::fs::read(&data_path)
                .wrap_err_with(|| format!("Access to data file <{:?}> not found!", &data_path))?;
            Ok(serde_json::from_slice(&data).wrap_err("Data not in JSON format!")?)
        }
        super::call_function_args_type::FunctionArgsType::WithTextFile => {
            let data_path = std::path::PathBuf::from(args);
            let data = std::fs::read_to_string(&data_path)
                .wrap_err_with(|| format!("Access to data file <{:?}> not found!", &data_path))?;
            Ok(serde_json::Value::String(data))
        }
    }
}
