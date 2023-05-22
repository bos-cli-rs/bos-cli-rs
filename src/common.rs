use std::collections::HashMap;

use color_eyre::eyre::{ContextCompat, WrapErr};
use console::{style, Style};
use glob::glob;
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt, RpcQueryResponseExt};
use serde::de::{Deserialize, Deserializer};
use similar::{ChangeTag, TextDiff};

struct Line(Option<usize>);

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

pub struct DiffCodeError;

pub fn diff_code(old_code: &str, new_code: &str) -> Result<(), DiffCodeError> {
    let old_code = old_code.trim();
    let new_code = new_code.trim();
    if old_code == new_code {
        return Ok(());
    }

    let diff = TextDiff::from_lines(old_code, new_code);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("{:-^1$}", "-", 80);
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                print!(
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                );
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        print!("{}", s.apply_to(value).underlined().on_black());
                    } else {
                        print!("{}", s.apply_to(value));
                    }
                }
                if change.missing_newline() {
                    println!();
                }
            }
        }
    }
    Err(DiffCodeError)
}

pub fn get_local_components() -> color_eyre::eyre::Result<
    std::collections::HashMap<String, crate::socialdb_types::SocialDbComponent>,
> {
    let mut components = HashMap::new();

    for component_filepath in glob("./src/**/*.jsx")?.filter_map(Result::ok) {
        let component_name: crate::socialdb_types::ComponentName = component_filepath
            .strip_prefix("src")?
            .with_extension("")
            .to_str()
            .wrap_err_with(|| {
                format!(
                    "Component name cannot be presented as UTF-8: {}",
                    component_filepath.display()
                )
            })?
            .replace('/', ".");

        let code = std::fs::read_to_string(&component_filepath).wrap_err_with(|| {
            format!(
                "Failed to read component source code from {}",
                component_filepath.display()
            )
        })?;

        let metadata_filepath = component_filepath.with_extension("metadata.json");
        let metadata = if let Ok(metadata_json) = std::fs::read_to_string(&metadata_filepath) {
            Some(serde_json::from_str(&metadata_json).wrap_err_with(|| {
                format!(
                    "Failed to parse component metadata from {}",
                    metadata_filepath.display()
                )
            })?)
        } else {
            None
        };

        components.insert(
            component_name,
            crate::socialdb_types::SocialDbComponent::CodeWithMetadata { code, metadata },
        );
    }
    Ok(components)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PermissionKey {
    #[serde(rename = "predecessor_id")]
    PredecessorId(near_primitives::types::AccountId),
    #[serde(rename = "public_key")]
    PublicKey(near_crypto::PublicKey),
}

impl From<near_primitives::types::AccountId> for PermissionKey {
    fn from(predecessor_id: near_primitives::types::AccountId) -> Self {
        Self::PredecessorId(predecessor_id)
    }
}

impl From<near_crypto::PublicKey> for PermissionKey {
    fn from(public_key: near_crypto::PublicKey) -> Self {
        Self::PublicKey(public_key)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct IsWritePermissionGrantedInputArgs {
    key: String,
    #[serde(flatten)]
    permission_key: PermissionKey,
}

pub fn is_write_permission_granted<P: Into<PermissionKey>>(
    network_config: &near_cli_rs::config::NetworkConfig,
    near_social_account_id: &near_primitives::types::AccountId,
    permission_key: P,
    key: String,
) -> color_eyre::eyre::Result<bool> {
    let function_args = serde_json::to_string(&IsWritePermissionGrantedInputArgs {
        key,
        permission_key: permission_key.into(),
    })
    .wrap_err("Internal error: could not serialize `is_write_permission_granted` input args")?;
    let call_result = network_config
        .json_rpc_client()
        .blocking_call_view_function(
            near_social_account_id,
            "is_write_permission_granted",
            function_args.into_bytes(),
            near_primitives::types::Finality::Final.into(),
        )
        .wrap_err_with(|| "Failed to fetch query for view method: 'is_write_permission_granted'")?;

    let serde_call_result: serde_json::Value = call_result.parse_result_from_json()?;
    let result = serde_call_result.as_bool().expect("Unexpected response");
    Ok(result)
}

pub fn is_signer_access_key_function_call_access_can_call_set_on_social_db_account(
    near_social_account_id: &near_primitives::types::AccountId,
    access_key_permission: &near_primitives::views::AccessKeyPermissionView,
) -> color_eyre::eyre::Result<bool> {
    if let near_primitives::views::AccessKeyPermissionView::FunctionCall {
        allowance: _,
        receiver_id,
        method_names,
    } = access_key_permission
    {
        Ok(receiver_id == &near_social_account_id.to_string()
            && method_names.contains(&"set".to_string()))
    } else {
        Ok(false)
    }
}

pub fn get_access_key_permission(
    network_config: &near_cli_rs::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    public_key: &near_crypto::PublicKey,
) -> color_eyre::eyre::Result<near_primitives::views::AccessKeyPermissionView> {
    let permission = network_config
        .json_rpc_client()
        .blocking_call_view_access_key(
            account_id,
            public_key,
            near_primitives::types::Finality::Final.into(),
        )
        .wrap_err_with(|| format!("Failed to fetch query 'view access key' for <{public_key}>",))?
        .access_key_view()?
        .permission;
    Ok(permission)
}

pub fn required_deposit(
    network_config: &near_cli_rs::config::NetworkConfig,
    near_social_account_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
    data: &serde_json::Value,
    prev_data: Option<&serde_json::Value>,
) -> color_eyre::eyre::Result<near_cli_rs::common::NearBalance> {
    const STORAGE_COST_PER_BYTE: i128 = 10i128.pow(19);
    const MIN_STORAGE_BALANCE: u128 = STORAGE_COST_PER_BYTE as u128 * 2000;
    const INITIAL_ACCOUNT_STORAGE_BALANCE: i128 = STORAGE_COST_PER_BYTE * 500;
    const EXTRA_STORAGE_BALANCE: i128 = STORAGE_COST_PER_BYTE * 5000;

    let call_result_storage_balance = network_config
        .json_rpc_client()
        .blocking_call_view_function(
            near_social_account_id,
            "storage_balance_of",
            serde_json::json!({
                "account_id": account_id,
            })
            .to_string()
            .into_bytes(),
            near_primitives::types::Finality::Final.into(),
        );

    let storage_balance_result: color_eyre::eyre::Result<StorageBalance> =
        call_result_storage_balance
            .wrap_err_with(|| "Failed to fetch query for view method: 'storage_balance_of'")?
            .parse_result_from_json()
            .wrap_err_with(|| {
                "Failed to parse return value of view function call for StorageBalance."
            });

    let (available_storage, initial_account_storage_balance, min_storage_balance) =
        if let Ok(storage_balance) = storage_balance_result {
            (storage_balance.available, 0, 0)
        } else {
            (0, INITIAL_ACCOUNT_STORAGE_BALANCE, MIN_STORAGE_BALANCE)
        };

    let estimated_storage_balance = u128::try_from(
        STORAGE_COST_PER_BYTE * estimate_data_size(data, prev_data) as i128
            + initial_account_storage_balance
            + EXTRA_STORAGE_BALANCE,
    )
    .unwrap_or(0)
    .saturating_sub(available_storage);
    Ok(near_cli_rs::common::NearBalance::from_yoctonear(
        std::cmp::max(estimated_storage_balance, min_storage_balance),
    ))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct StorageBalance {
    #[serde(deserialize_with = "parse_u128_string")]
    pub available: u128,
    #[serde(deserialize_with = "parse_u128_string")]
    pub total: u128,
}

fn parse_u128_string<'de, D>(deserializer: D) -> color_eyre::eyre::Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse::<u128>()
        .map_err(serde::de::Error::custom)
}

/// https://github.com/NearSocial/VM/blob/24055641b53e7eeadf6efdb9c073f85f02463798/src/lib/data/utils.js#L182-L198
fn estimate_data_size(data: &serde_json::Value, prev_data: Option<&serde_json::Value>) -> isize {
    const ESTIMATED_KEY_VALUE_SIZE: isize = 40 * 3 + 8 + 12;
    const ESTIMATED_NODE_SIZE: isize = 40 * 2 + 8 + 10;

    match data {
        serde_json::Value::Object(data) => {
            let inner_data_size = data
                .iter()
                .map(|(key, value)| {
                    let prev_value = if let Some(serde_json::Value::Object(prev_data)) = prev_data {
                        prev_data.get(key)
                    } else {
                        None
                    };
                    if prev_value.is_some() {
                        estimate_data_size(value, prev_value)
                    } else {
                        key.len() as isize * 2
                            + estimate_data_size(value, None)
                            + ESTIMATED_KEY_VALUE_SIZE
                    }
                })
                .sum();
            if prev_data.map(serde_json::Value::is_object).unwrap_or(false) {
                inner_data_size
            } else {
                ESTIMATED_NODE_SIZE + inner_data_size
            }
        }
        serde_json::Value::String(data) => {
            data.len().max(8) as isize
                - prev_data
                    .and_then(serde_json::Value::as_str)
                    .map(str::len)
                    .unwrap_or(0) as isize
        }
        _ => {
            unreachable!("estimate_data_size expects only Object or String values");
        }
    }
}

/// Helper function that marks SocialDB values to be deleted by setting `null` to the values
pub fn mark_leaf_values_as_null(data: &mut serde_json::Value) {
    match data {
        serde_json::Value::Object(object_data) => {
            for value in object_data.values_mut() {
                mark_leaf_values_as_null(value);
            }
        }
        data => {
            *data = serde_json::Value::Null;
        }
    }
}
