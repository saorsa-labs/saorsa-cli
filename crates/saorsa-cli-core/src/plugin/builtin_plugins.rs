use crate::{CoreError, CoreResult, Plugin, PluginContext, PluginDescriptor, PluginMetadata};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const BUILTIN_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn builtin_plugins() -> Vec<(PluginDescriptor, Box<dyn Plugin>)> {
    vec![fd_plugin(), ripgrep_plugin()]
}

fn fd_plugin() -> (PluginDescriptor, Box<dyn Plugin>) {
    let help = "Wrapper around fd (fd-find). Provides fast file search. Supply a pattern \
and optional path. For example: `fd src main`."
        .to_string();
    build_plugin(
        "fd",
        "First-party wrapper around fd (fd-find) for lightning-fast file discovery.",
        help,
        "fd",
        Vec::new(),
    )
}

fn ripgrep_plugin() -> (PluginDescriptor, Box<dyn Plugin>) {
    let help = "Wrapper around ripgrep (rg). Provide a pattern and optional path/flags. \
By default this plugin will show ripgrep help when run without arguments."
        .to_string();
    build_plugin(
        "rg",
        "Ripgrep (rg) wrapper for searching file contents via the plugin menu.",
        help,
        "rg",
        vec!["--help".into()],
    )
}

fn build_plugin(
    name: &str,
    description: &str,
    help: String,
    command: &str,
    default_args: Vec<String>,
) -> (PluginDescriptor, Box<dyn Plugin>) {
    let metadata = PluginMetadata {
        name: name.to_string(),
        version: BUILTIN_VERSION.to_string(),
        description: description.to_string(),
        author: "Saorsa Labs".to_string(),
        help: Some(help.clone()),
        manifest_path: PathBuf::from(format!("builtin://{name}/manifest")),
        library_path: PathBuf::from(format!("builtin://{name}/library")),
    };

    let descriptor = PluginDescriptor {
        metadata: metadata.clone(),
    };

    let plugin = Box::new(ExternalCommandPlugin {
        metadata,
        help_text: help,
        command: command.to_string(),
        default_args,
    });

    (descriptor, plugin)
}

struct ExternalCommandPlugin {
    metadata: PluginMetadata,
    help_text: String,
    command: String,
    default_args: Vec<String>,
}

impl Plugin for ExternalCommandPlugin {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn description(&self) -> &str {
        &self.metadata.description
    }

    fn version(&self) -> &str {
        &self.metadata.version
    }

    fn author(&self) -> &str {
        &self.metadata.author
    }

    fn help(&self) -> &str {
        &self.help_text
    }

    fn execute(&self, args: &[String], _ctx: PluginContext<'_>) -> CoreResult<()> {
        let mut command = Command::new(&self.command);
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if args.is_empty() {
            if !self.default_args.is_empty() {
                command.args(&self.default_args);
            }
        } else {
            command.args(args);
        }

        let status = command.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(CoreError::EventError(format!(
                "{} exited with status {}",
                self.metadata.name, status
            )))
        }
    }
}
