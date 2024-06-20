use std::collections::HashMap;

use color_eyre::eyre::{ContextCompat, WrapErr};
use console::{style, Style};
use futures::StreamExt;
use glob::glob;
use near_cli_rs::common::CallResultExt;
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

pub fn get_local_components(
) -> color_eyre::eyre::Result<HashMap<String, crate::socialdb_types::SocialDbComponent>> {
    let mut components = HashMap::new();

    for component_filepath in glob("./src/**/*.jsx")?.filter_map(Result::ok) {
        let component_name: crate::socialdb_types::ComponentName = component_filepath
            .strip_prefix("src")?
            .with_extension("")
            .components()
            .filter_map(|component| match component {
                std::path::Component::Normal(text) => Some(text.to_str().wrap_err_with(|| {
                    format!(
                        "Component name cannot be presented as UTF-8: {}",
                        component_filepath.display()
                    )
                }).ok()?),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(".");

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

pub fn get_remote_components(
    network_config: &near_cli_rs::config::NetworkConfig,
    component_name_list: Vec<&String>,
    near_social_account_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
    social_db_folder: &str,
) -> color_eyre::eyre::Result<
    HashMap<crate::socialdb_types::ComponentName, crate::socialdb_types::SocialDbComponent>,
> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let chunk_size = 15;
    let concurrency = 10;

    runtime
        .block_on(
            futures::stream::iter(component_name_list.chunks(chunk_size))
                .map(|components_name_batch| async {
                    get_components(
                        network_config,
                        near_social_account_id,
                        account_id,
                        components_name_batch,
                        social_db_folder,
                    )
                    .await
                })
                .buffer_unordered(concurrency)
                .collect::<Vec<Result<_, _>>>(),
        )
        .into_iter()
        .try_fold(HashMap::new(), |mut acc, x| {
            acc.extend(x?);
            Ok::<_, color_eyre::eyre::Error>(acc)
        })
}

async fn get_components(
    network_config: &near_cli_rs::config::NetworkConfig,
    near_social_account_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
    components_names_batch: &[&crate::socialdb_types::ComponentName],
    social_db_folder: &str,
) -> color_eyre::Result<
    HashMap<crate::socialdb_types::ComponentName, crate::socialdb_types::SocialDbComponent>,
> {
    let args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
        keys: components_names_batch
            .iter()
            .map(|name| format!("{account_id}/{social_db_folder}/{name}/**"))
            .collect(),
    })
    .wrap_err("Internal error: could not serialize SocialDB input args")?
    .into_bytes();

    match network_config
        .json_rpc_client()
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: near_social_account_id.clone(),
                method_name: "get".to_string(),
                args: near_primitives::types::FunctionArgs::from(args),
            },
        })
        .await
        .wrap_err("Failed to query batch of components from Social DB")?
        .kind
    {
        near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(call_result) => {
            Ok(call_result
                .parse_result_from_json::<crate::socialdb_types::SocialDb>()
                .wrap_err("ERROR: failed to parse Social DB response")?
                .accounts
                .remove(account_id)
                .map(|crate::socialdb_types::SocialDbComponentKey { key }| key)
                .unwrap_or_default()
                .remove(social_db_folder)
                .map(|crate::socialdb_types::SocialDbAccountMetadata { components }| components)
                .unwrap_or_default())
        }
        _ => unreachable!("ERROR: unexpected response type from JSON RPC client"),
    }
}

pub fn get_updated_components(
    local_components: HashMap<String, crate::socialdb_types::SocialDbComponent>,
    remote_components: &HashMap<
        crate::socialdb_types::ComponentName,
        crate::socialdb_types::SocialDbComponent,
    >,
) -> HashMap<String, crate::socialdb_types::SocialDbComponent> {
    local_components
        .into_iter()
        .filter(|(component_name, new_component)| {
            if let Some(old_component) = remote_components.get(component_name) {
                let has_code_changed = crate::common::diff_code(old_component.code(), new_component.code()).is_err();
                let has_metadata_changed = old_component.metadata() != new_component.metadata() && new_component.metadata().is_some();
                if !has_code_changed {
                    println!("Code for component <{component_name}> has not changed");
                }
                if has_metadata_changed {
                    println!(
                        "Metadata for component <{component_name}> changed:\n - old metadata: {:?}\n - new metadata: {:?}",
                        old_component.metadata(), new_component.metadata()
                    );
                } else {
                    println!("Metadata for component <{component_name}> has not changed");
                }
                has_code_changed || has_metadata_changed
            } else {
                println!("Found new component <{component_name}> to deploy");
                true
            }
        })
        .collect()
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

pub fn social_db_data_from_key(full_key: &str, data_to_set: &mut serde_json::Value) {
    if let Some((prefix, key)) = full_key.rsplit_once('/') {
        *data_to_set = serde_json::json!({ key: data_to_set });
        social_db_data_from_key(prefix, data_to_set)
    } else {
        *data_to_set = serde_json::json!({ full_key: data_to_set });
    }
}
