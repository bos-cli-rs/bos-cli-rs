use std::collections::HashMap;

use color_eyre::eyre::{ContextCompat, WrapErr};
use console::{style, Style};
use glob::glob;
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt, RpcQueryResponseExt};
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

pub fn is_account_exist(
    networks: &linked_hash_map::LinkedHashMap<String, near_cli_rs::config::NetworkConfig>,
    account_id: near_primitives::types::AccountId,
) -> bool {
    for network in networks {
        if near_cli_rs::common::get_account_state(
            network.1.clone(),
            account_id.clone(),
            near_primitives::types::Finality::Final.into(),
        )
        .is_ok()
        {
            return true;
        }
    }
    false
}

pub fn get_local_widgets() -> color_eyre::eyre::Result<
    std::collections::HashMap<String, crate::socialdb_types::SocialDbWidget>,
> {
    let mut widgets = HashMap::new();

    for widget_filepath in glob("./src/**/*.jsx")?.filter_map(Result::ok) {
        let widget_name: crate::socialdb_types::WidgetName = widget_filepath
            .strip_prefix("src")?
            .with_extension("")
            .to_str()
            .wrap_err_with(|| {
                format!(
                    "Widget name cannot be presented as UTF-8: {}",
                    widget_filepath.display()
                )
            })?
            .replace('/', ".");

        let code = std::fs::read_to_string(&widget_filepath).wrap_err_with(|| {
            format!(
                "Failed to read widget source code from {}",
                widget_filepath.display()
            )
        })?;

        let metadata_filepath = widget_filepath.with_extension("metadata.json");
        let metadata = if let Ok(metadata_json) = std::fs::read_to_string(&metadata_filepath) {
            Some(serde_json::from_str(&metadata_json).wrap_err_with(|| {
                format!(
                    "Failed to parse widget metadata from {}",
                    metadata_filepath.display()
                )
            })?)
        } else {
            None
        };

        widgets.insert(
            widget_name,
            crate::socialdb_types::SocialDbWidget { code, metadata },
        );
    }
    Ok(widgets)
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
