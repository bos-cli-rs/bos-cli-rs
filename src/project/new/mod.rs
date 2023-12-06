use color_eyre::eyre::{ContextCompat, WrapErr};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = NewContext)]
pub struct New {
    /// Enter a new project name (path to the project):
    pub project_dir: near_cli_rs::types::path_buf::PathBuf,
}

#[derive(Debug, Clone)]
pub struct NewContext;

impl NewContext {
    pub fn from_previous_context(
        _previous_context: near_cli_rs::GlobalContext,
        scope: &<New as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let project_dir: &std::path::Path = scope.project_dir.as_ref();

        for new_project_file in NEW_PROJECT_FILES {
            let new_file_path = project_dir.join(new_project_file.file_path);
            std::fs::create_dir_all(new_file_path.parent().wrap_err_with(|| {
                format!("Impossible to get parent for `{}`", new_file_path.display())
            })?)?;
            std::fs::write(&new_file_path, new_project_file.content).wrap_err_with(|| {
                format!("Failed to write to file: {}", new_file_path.display())
            })?;
        }

        std::process::Command::new("git")
            .arg("init")
            .current_dir(project_dir)
            .output()
            .wrap_err("Failed to execute process: `git init`")?;

        println!("New project is created at '{}'\n", project_dir.display());
        println!("Now you can build and deploy your project:");
        println!("1. Install dependencies: `npm install`");
        println!("2. Edit components");
        println!("3. Deploy: `npm run deploy`");

        Ok(Self)
    }
}

struct NewProjectFile {
    file_path: &'static str,
    content: &'static str,
}

const NEW_PROJECT_FILES: &[NewProjectFile] = &[
    NewProjectFile {
        file_path: ".github/workflows/test.yml",
        content: include_str!("new-project-template/.github/workflows/test.yml"),
    },
    NewProjectFile {
        file_path: ".husky/pre-commit",
        content: include_str!("new-project-template/.husky/pre-commit"),
    },
    NewProjectFile {
        file_path: "src/components/pages/homepage.tsx",
        content: include_str!("new-project-template/src/components/pages/homepage.tsx"),
    },
    NewProjectFile {
        file_path: "src/components/subfolder/my-nested-component.tsx",
        content: include_str!(
            "new-project-template/src/components/subfolder/my-nested-component.tsx"
        ),
    },
    NewProjectFile {
        file_path: "src/includes/common.tsx",
        content: include_str!("new-project-template/src/includes/common.tsx"),
    },
    NewProjectFile {
        file_path: ".editorconfig",
        content: include_str!("new-project-template/.editorconfig"),
    },
    NewProjectFile {
        file_path: ".gitattributes",
        content: include_str!("new-project-template/.gitattributes"),
    },
    NewProjectFile {
        file_path: ".gitignore",
        content: include_str!("new-project-template/.gitignore"),
    },
    NewProjectFile {
        file_path: "build.js",
        content: include_str!("new-project-template/build.js"),
    },
    NewProjectFile {
        file_path: "eslint.config.js",
        content: include_str!("new-project-template/eslint.config.js"),
    },
    NewProjectFile {
        file_path: "global.d.ts",
        content: include_str!("new-project-template/global.d.ts"),
    },
    NewProjectFile {
        file_path: "LICENSE-MIT",
        content: include_str!("new-project-template/LICENSE-MIT"),
    },
    NewProjectFile {
        file_path: "package-lock.json",
        content: include_str!("new-project-template/package-lock.json"),
    },
    NewProjectFile {
        file_path: "package.json",
        content: include_str!("new-project-template/package.json"),
    },
    NewProjectFile {
        file_path: "README.md",
        content: include_str!("new-project-template/README.md"),
    },
    NewProjectFile {
        file_path: "tsconfig.json",
        content: include_str!("new-project-template/tsconfig.json"),
    },
];
