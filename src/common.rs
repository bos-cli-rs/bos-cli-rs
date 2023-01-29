use std::collections::HashMap;

use color_eyre::eyre::{ContextCompat, WrapErr};
use console::{style, Style};
use glob::glob;
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
    context: &near_cli_rs::GlobalContext,
    account_id: near_primitives::types::AccountId,
) -> bool {
    for network in context.0.networks.iter() {
        if tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(near_cli_rs::common::get_account_state(
                network.1.clone(),
                account_id.clone(),
                near_primitives::types::Finality::Final.into(),
            ))
            .is_ok()
        {
            return true;
        }
    }
    false
}

pub fn get_widgets() -> color_eyre::eyre::Result<
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
