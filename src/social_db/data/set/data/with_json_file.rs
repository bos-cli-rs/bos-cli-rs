use color_eyre::eyre::Context;
use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::SetContext)]
#[interactive_clap(output_context = JsonDataFileContext)]
pub struct JsonDataFile {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the path to the data file:
    path: near_cli_rs::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct JsonDataFileContext(super::DataContext);

impl JsonDataFileContext {
    pub fn from_previous_context(
        previous_context: super::super::SetContext,
        scope: &<JsonDataFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let file = std::fs::File::open(&scope.path.0)
            .wrap_err_with(|| format!("Access to data file <{:?}> not found!", scope.path))?;
        let reader = std::io::BufReader::new(file);
        let value: serde_json::Value =
            serde_json::from_reader(reader).wrap_err("File data is not in JSON format!")?;
        Ok(Self(super::DataContext {
            global_context: previous_context.global_context,
            set_to_account_id: previous_context.set_to_account_id,
            key: previous_context.key,
            value,
        }))
    }
}

impl From<JsonDataFileContext> for super::DataContext {
    fn from(item: JsonDataFileContext) -> Self {
        item.0
    }
}

impl JsonDataFile {
    fn input_path(
        _context: &super::super::SetContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::path_buf::PathBuf>> {
        loop {
            let path: near_cli_rs::types::path_buf::PathBuf =
                CustomType::new("Enter the path to the arguments file:").prompt()?;
            let file_result = std::fs::File::open(&path.0);
            if let Ok(file) = file_result {
                let reader = std::io::BufReader::new(file);
                if serde_json::from_reader::<std::io::BufReader<std::fs::File>, serde_json::Value>(
                    reader,
                )
                .is_err()
                {
                    println!("File data is not in JSON format!");
                } else {
                    return Ok(Some(path));
                }
            } else {
                println!("Access to data file <{:?}> not found!", path)
            }
        }
    }
}
