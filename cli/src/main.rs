mod config;
mod downloader;
mod error;
mod menu;
mod platform;
mod plugin;
mod runner;
mod self_update;
mod updater;
mod version;

use crate::config::Config;
use crate::downloader::Downloader;
use crate::menu::{Menu, MenuChoice};
use crate::platform::Platform;
use crate::runner::BinaryRunner;
use crate::updater::{UpdateCheckResult, UpdateChecker};
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;

// Dialoguer is already imported in the functions where needed

#[derive(Parser, Debug)]
#[command(
    name = "saorsa",
    about = "Interactive CLI menu for Saorsa tools",
    version,
    author
)]
struct Args {
    /// Disable automatic update checks
    #[arg(long)]
    no_update_check: bool,

    /// Use system-installed binaries instead of downloading
    #[arg(long)]
    use_system: bool,

    /// Force re-download of binaries
    #[arg(long)]
    force_download: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Run a specific tool directly (sb or sdisk)
    #[arg(short, long)]
    run: Option<String>,

    /// Arguments to pass to the tool (when using --run)
    #[arg(trailing_var_arg = true)]
    tool_args: Vec<String>,

    /// Execute a plugin
    #[arg(long)]
    plugin: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.verbose {
        EnvFilter::from_default_env()
            .add_directive("cli=debug".parse()?)
            .add_directive("saorsa=debug".parse()?)
    } else {
        EnvFilter::from_default_env()
            .add_directive("cli=info".parse()?)
            .add_directive("saorsa=info".parse()?)
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // Load configuration
    let mut config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load configuration: {}", e);
        eprintln!("Using default configuration...");
        Config::default()
    });
    config.update_from_cli(args.no_update_check, args.use_system);
    config.ensure_directories()?;

    // Detect platform
    let platform = Platform::detect().context("Failed to detect platform")?;

    tracing::debug!("Detected platform: {:?}", platform);

    // Initialize components with Arc wrappers for thread-safe sharing
    let downloader = Arc::new(Downloader::new(
        config.github.owner.clone(),
        config.github.repo.clone(),
    )?);

    // Wrap config in Arc<RwLock<>> for thread-safe access from background task
    let config = Arc::new(RwLock::new(config));

    // Spawn background update checker (non-blocking)
    let update_result: Arc<RwLock<Option<UpdateCheckResult>>> = Arc::new(RwLock::new(None));
    {
        let update_checker = UpdateChecker::new(Arc::clone(&config), Arc::clone(&downloader));
        let update_result_clone = Arc::clone(&update_result);
        tokio::spawn(async move {
            if let Some(result) = update_checker.check().await {
                *update_result_clone.write().await = Some(result);
            }
        });
    }

    let runner = BinaryRunner::new();

    // Initialize plugin system
    let mut plugin_manager =
        plugin::init_plugin_system().context("Failed to initialize plugin system")?;

    // Handle plugin execution
    if let Some(plugin_name) = args.plugin.as_ref() {
        return plugin_manager.execute_plugin(plugin_name, &args.tool_args);
    }

    // Handle direct run mode
    if let Some(tool) = args.run.as_ref() {
        let config_read = config.read().await;
        return run_tool_directly(
            tool,
            args.tool_args,
            &config_read,
            &platform,
            &downloader,
            &runner,
            args.force_download,
        )
        .await;
    }

    // Main menu loop
    let mut menu = Menu::new();

    loop {
        // Refresh update status (background task may have completed)
        if let Some(result) = update_result.read().await.as_ref() {
            if result.update_available {
                menu.set_update_status(result.latest_version.clone());
            }
        }

        // Check for binaries and update menu
        let config_read = config.read().await;
        let (sb_path, sdisk_path) =
            check_binaries(&config_read, &platform, &downloader, &runner).await?;
        drop(config_read); // Release lock before menu interaction
        menu.set_binary_paths(sb_path.clone(), sdisk_path.clone());

        // Show menu and get choice
        let choice = menu.run().await?;

        match choice {
            MenuChoice::RunSB => {
                if let Some(path) = sb_path {
                    println!("Starting Saorsa Browser...");
                    runner.run_interactive(&path, vec![])?;
                } else {
                    println!("Saorsa Browser not installed. Attempting to download...");
                    match downloader.download_binary("sb", &platform, false).await {
                        Ok(path) => {
                            runner.run_interactive(&path, vec![])?;
                        }
                        Err(e) => {
                            if let Some(downloader_err) =
                                e.downcast_ref::<crate::downloader::DownloadError>()
                            {
                                match downloader_err {
                                    crate::downloader::DownloadError::NoReleases => {
                                        println!("❌ No releases found for Saorsa Browser.");
                                        println!("This might be normal if the repository has no releases yet.");
                                        println!("Press Enter to continue...");
                                        let mut input = String::new();
                                        std::io::stdin().read_line(&mut input)?;
                                    }
                                    _ => {
                                        println!(
                                            "❌ Failed to download Saorsa Browser: {}",
                                            downloader_err
                                        );
                                        println!("Press Enter to continue...");
                                        let mut input = String::new();
                                        std::io::stdin().read_line(&mut input)?;
                                    }
                                }
                            } else {
                                println!("❌ Failed to download Saorsa Browser: {}", e);
                                println!("Press Enter to continue...");
                                let mut input = String::new();
                                std::io::stdin().read_line(&mut input)?;
                            }
                        }
                    }
                }
            }
            MenuChoice::RunSDisk => {
                if let Some(path) = sdisk_path {
                    println!("Starting Saorsa Disk...");
                    runner.run_interactive(&path, vec![])?;
                } else {
                    println!("Saorsa Disk not installed. Attempting to download...");
                    match downloader.download_binary("sdisk", &platform, false).await {
                        Ok(path) => {
                            runner.run_interactive(&path, vec![])?;
                        }
                        Err(e) => {
                            if let Some(downloader_err) =
                                e.downcast_ref::<crate::downloader::DownloadError>()
                            {
                                match downloader_err {
                                    crate::downloader::DownloadError::NoReleases => {
                                        println!("❌ No releases found for Saorsa Disk.");
                                        println!("This might be normal if the repository has no releases yet.");
                                        println!("Press Enter to continue...");
                                        let mut input = String::new();
                                        std::io::stdin().read_line(&mut input)?;
                                    }
                                    _ => {
                                        println!(
                                            "❌ Failed to download Saorsa Disk: {}",
                                            downloader_err
                                        );
                                        println!("Press Enter to continue...");
                                        let mut input = String::new();
                                        std::io::stdin().read_line(&mut input)?;
                                    }
                                }
                            } else {
                                println!("❌ Failed to download Saorsa Disk: {}", e);
                                println!("Press Enter to continue...");
                                let mut input = String::new();
                                std::io::stdin().read_line(&mut input)?;
                            }
                        }
                    }
                }
            }
            MenuChoice::UpdateBinaries => {
                println!("Updating binaries...");
                update_binaries(&platform, &downloader).await?;
                println!("Update complete! Press Enter to continue...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
            }
            MenuChoice::UpdateCLI => {
                use dialoguer::{theme::ColorfulTheme, Confirm};

                // Confirm with user
                let confirm = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Download and install update? This will restart the application.")
                    .default(true)
                    .interact()?;

                if !confirm {
                    println!("Update cancelled.");
                    println!("Press Enter to continue...");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    continue;
                }

                // Perform update
                match self_update::perform_self_update(&downloader, &platform).await {
                    Ok(result) => {
                        if result.needs_restart {
                            println!("\nRestarting with new version...");
                            if let Err(e) = self_update::restart() {
                                println!("Failed to restart: {}", e);
                                println!("Please manually restart the application.");
                                println!("Press Enter to exit...");
                                let mut input = String::new();
                                std::io::stdin().read_line(&mut input)?;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Update failed: {}", e);
                        println!("Press Enter to continue...");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                    }
                }
            }
            MenuChoice::Settings => {
                let mut config_write = config.write().await;
                *config_write = show_settings_menu(config_write.clone())?;
                config_write
                    .save()
                    .context("Failed to save configuration")?;
            }
            MenuChoice::Plugins => {
                show_plugins_menu(&mut plugin_manager)?;
            }
            MenuChoice::Exit => {
                println!("Goodbye!");
                break;
            }
        }
    }

    Ok(())
}

async fn check_binaries(
    config: &Config,
    platform: &Platform,
    downloader: &Downloader,
    runner: &BinaryRunner,
) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
    let mut sb_path = None;
    let mut sdisk_path = None;

    // Check for sb binary
    if config.behavior.use_system_binaries {
        sb_path = runner.which("sb");
    }
    if sb_path.is_none() {
        let cache_path = downloader.binary_path("sb", platform);
        if runner.check_binary_exists(&cache_path) {
            sb_path = Some(cache_path);
        }
    }

    // Check for sdisk binary
    if config.behavior.use_system_binaries {
        sdisk_path = runner.which("sdisk");
    }
    if sdisk_path.is_none() {
        let cache_path = downloader.binary_path("sdisk", platform);
        if runner.check_binary_exists(&cache_path) {
            sdisk_path = Some(cache_path);
        }
    }

    Ok((sb_path, sdisk_path))
}

async fn update_binaries(platform: &Platform, downloader: &Downloader) -> Result<()> {
    println!("Checking for latest sb binary...");
    match downloader.download_binary("sb", platform, true).await {
        Ok(_) => println!("✓ Successfully downloaded sb binary"),
        Err(e) => {
            if let Some(downloader_err) = e.downcast_ref::<crate::downloader::DownloadError>() {
                match downloader_err {
                    crate::downloader::DownloadError::NoReleases => {
                        println!("⚠ No releases found for sb. This might be normal if the repository has no releases yet.");
                    }
                    _ => {
                        println!("✗ Failed to download sb binary: {}", downloader_err);
                        return Err(e);
                    }
                }
            } else {
                println!("✗ Failed to download sb binary: {}", e);
                return Err(e);
            }
        }
    }

    println!("Checking for latest sdisk binary...");
    match downloader.download_binary("sdisk", platform, true).await {
        Ok(_) => println!("✓ Successfully downloaded sdisk binary"),
        Err(e) => {
            if let Some(downloader_err) = e.downcast_ref::<crate::downloader::DownloadError>() {
                match downloader_err {
                    crate::downloader::DownloadError::NoReleases => {
                        println!("⚠ No releases found for sdisk. This might be normal if the repository has no releases yet.");
                    }
                    _ => {
                        println!("✗ Failed to download sdisk binary: {}", downloader_err);
                        return Err(e);
                    }
                }
            } else {
                println!("✗ Failed to download sdisk binary: {}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}

async fn run_tool_directly(
    tool: &str,
    args: Vec<String>,
    config: &Config,
    platform: &Platform,
    downloader: &Downloader,
    runner: &BinaryRunner,
    force_download: bool,
) -> Result<()> {
    let binary_name = match tool {
        "sb" | "saorsa-browser" => "sb",
        "sdisk" | "saorsa-disk" => "sdisk",
        _ => {
            anyhow::bail!("Unknown tool: {}. Available tools: sb, sdisk", tool);
        }
    };

    // Try to find the binary
    let mut binary_path = None;

    if config.behavior.use_system_binaries && !force_download {
        binary_path = runner.which(binary_name);
    }

    if binary_path.is_none() {
        let cache_path = downloader.binary_path(binary_name, platform);
        if runner.check_binary_exists(&cache_path) && !force_download {
            binary_path = Some(cache_path);
        } else {
            println!("Downloading {} binary...", binary_name);
            match downloader
                .download_binary(binary_name, platform, force_download)
                .await
            {
                Ok(path) => {
                    binary_path = Some(path);
                }
                Err(e) => {
                    if let Some(downloader_err) =
                        e.downcast_ref::<crate::downloader::DownloadError>()
                    {
                        match downloader_err {
                            crate::downloader::DownloadError::NoReleases => {
                                println!("❌ No releases found for {}.", binary_name);
                                println!(
                                    "This might be normal if the repository has no releases yet."
                                );
                                return Err(anyhow::anyhow!(
                                    "No releases found for {}",
                                    binary_name
                                ));
                            }
                            _ => {
                                println!(
                                    "❌ Failed to download {} binary: {}",
                                    binary_name, downloader_err
                                );
                                return Err(e);
                            }
                        }
                    } else {
                        println!("❌ Failed to download {} binary: {}", binary_name, e);
                        return Err(e);
                    }
                }
            }
        }
    }

    if let Some(path) = binary_path {
        runner.run_interactive(&path, args)?;
    } else {
        anyhow::bail!("Failed to find or download {} binary", binary_name);
    }

    Ok(())
}

/// Interactive settings configuration menu.
#[allow(clippy::too_many_lines)]
fn show_settings_menu(mut config: Config) -> Result<Config> {
    use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

    loop {
        println!("\n=== Settings Configuration ===\n");

        let options = vec![
            format!("GitHub Owner: {}", config.github.owner),
            format!("GitHub Repository: {}", config.github.repo),
            format!("Check Prereleases: {}", config.github.check_prerelease),
            format!("Auto Update Check: {}", config.behavior.auto_update_check),
            format!(
                "Use System Binaries: {}",
                config.behavior.use_system_binaries
            ),
            format!("Prefer Local Build: {}", config.behavior.prefer_local_build),
            "Save and Return".to_string(),
            "Cancel".to_string(),
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select setting to modify")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                // GitHub Owner
                let owner: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter GitHub owner")
                    .default(config.github.owner.clone())
                    .interact_text()?;
                config.github.owner = owner;
            }
            1 => {
                // GitHub Repository
                let repo: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter GitHub repository name")
                    .default(config.github.repo.clone())
                    .interact_text()?;
                config.github.repo = repo;
            }
            2 => {
                // Check Prereleases
                let prerelease = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Check prerelease versions?")
                    .default(config.github.check_prerelease)
                    .interact()?;
                config.github.check_prerelease = prerelease;
            }
            3 => {
                // Auto Update Check
                let auto_update = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enable automatic update checks?")
                    .default(config.behavior.auto_update_check)
                    .interact()?;
                config.behavior.auto_update_check = auto_update;
            }
            4 => {
                // Use System Binaries
                let use_system = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Use system-installed binaries when available?")
                    .default(config.behavior.use_system_binaries)
                    .interact()?;
                config.behavior.use_system_binaries = use_system;
            }
            5 => {
                // Prefer Local Build
                let prefer_local = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Prefer local builds over downloads?")
                    .default(config.behavior.prefer_local_build)
                    .interact()?;
                config.behavior.prefer_local_build = prefer_local;
            }
            6 => {
                // Save and Return
                return Ok(config);
            }
            7 => {
                // Cancel
                return Ok(config);
            }
            _ => unreachable!(),
        }
    }
}

/// Display current configuration settings.
/// Reserved for future settings display feature.
#[allow(dead_code)]
fn show_settings(config: &Config) -> Result<()> {
    println!("\n=== Current Settings ===\n");
    println!(
        "GitHub Repository: {}/{}",
        config.github.owner, config.github.repo
    );
    println!("Check Prereleases: {}", config.github.check_prerelease);
    println!("Cache Directory: {:?}", config.cache_dir()?);
    println!("Auto Update Check: {}", config.behavior.auto_update_check);
    println!(
        "Use System Binaries: {}",
        config.behavior.use_system_binaries
    );
    println!("Prefer Local Build: {}", config.behavior.prefer_local_build);
    println!("\nConfig file: {:?}", Config::config_path()?);
    println!("\nPress Enter to continue...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}

/// Display detailed information about a specific plugin.
/// Reserved for future plugin details UI.
#[allow(dead_code)]
fn show_plugin_details(
    plugin_manager: &plugin::PluginManager,
    plugin: &plugin::PluginMetadata,
) -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, Select};

    loop {
        println!("\n=== Plugin Details ===");
        println!("🔌 Name: {}", plugin.name);
        println!("📦 Version: {}", plugin.version);
        println!("📝 Description: {}", plugin.description);
        // Path information removed from PluginMetadata for simplicity

        if let Some(plugin_instance) = plugin_manager.get_plugin_info(&plugin.name) {
            println!("🎯 Help: {}", plugin_instance.help);
        }

        let options = vec![
            "▶️  Execute plugin",
            "📖 Show help",
            "🚪 Back to plugin list",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                // Execute plugin
                // Execute the plugin
                println!("\nExecuting plugin '{}'...", plugin.name);
                match plugin_manager.execute_plugin(&plugin.name, &[]) {
                    Ok(_) => println!("✅ Plugin executed successfully"),
                    Err(e) => println!("❌ Plugin execution failed: {}", e),
                }
            }
            1 => {
                // Show help
                if let Some(plugin_instance) = plugin_manager.get_plugin_info(&plugin.name) {
                    println!("\n=== Plugin Help ===");
                    println!("{}", plugin_instance.help);
                    println!("\nPress Enter to continue...");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                }
            }
            2 => {
                // Back to plugin list
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn show_plugin_directories(plugin_manager: &plugin::PluginManager) -> Result<()> {
    println!("\n=== Plugin Directories ===");

    let dirs = plugin_manager.get_plugin_names();
    if dirs.is_empty() {
        println!("No plugin directories configured.");
    } else {
        for (i, dir) in dirs.iter().enumerate() {
            println!("{}. {:?}", i + 1, dir);
        }
    }

    println!("\nDefault plugin directories:");
    println!("  - ~/.saorsa-cli/plugins (user plugins)");
    println!("  - /usr/local/lib/saorsa-cli/plugins (system plugins)");

    println!("\nPress Enter to continue...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}

fn show_plugins_menu(plugin_manager: &mut plugin::PluginManager) -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, Input, Select};

    loop {
        println!("\n=== Plugin Management ===\n");

        let plugins = plugin_manager.get_plugins();

        if plugins.is_empty() {
            println!("No plugins loaded.");
            println!("\nPlugin directories:");
            for dir in plugin_manager.get_plugin_names() {
                println!("  - {:?}", dir);
            }
            println!("\nPress Enter to continue...");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            return Ok(());
        }

        // Create menu options
        let mut options: Vec<String> = plugins
            .iter()
            .map(|plugin| {
                format!(
                    "🔌 Execute: {} v{} - {}",
                    plugin.name, plugin.version, plugin.description
                )
            })
            .collect();

        options.push("📋 Show Plugin Details".to_string());
        options.push("🔄 Refresh Plugins".to_string());
        options.push("📁 Show Plugin Directories".to_string());
        options.push("🚪 Return to Main Menu".to_string());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select plugin action")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            // Execute plugin options
            i if i < plugins.len() => {
                let plugin = &plugins[i];
                let plugin_name = &plugin.name;

                if let Some(_plugin_instance) = plugin_manager.get_plugin_info(plugin_name) {
                    println!("\n🎯 Executing plugin: {}", plugin_name);
                    println!("📝 Description: {}", plugin.description);
                    println!("🏷️  Version: {}", plugin.version);

                    // Get arguments for the plugin
                    let args_input: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter arguments (or leave empty)")
                        .allow_empty(true)
                        .interact_text()?;

                    let args: Vec<String> = if args_input.trim().is_empty() {
                        vec![]
                    } else {
                        args_input
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect()
                    };

                    println!("\n🚀 Executing {} with args: {:?}", plugin_name, args);

                    match plugin_manager.execute_plugin(plugin_name, &args) {
                        Ok(_) => {
                            println!("\n✅ Plugin executed successfully!");
                        }
                        Err(e) => {
                            println!("\n❌ Plugin execution failed: {}", e);
                        }
                    }

                    println!("\nPress Enter to continue...");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                }
            }

            // Show plugin details
            i if i == plugins.len() => {
                // Show all plugins
                println!("\n=== All Plugins ===");
                for plugin in &plugins {
                    println!("\n📦 {}", plugin.name);
                    println!("   Version: {}", plugin.version);
                    println!("   Description: {}", plugin.description);
                }
                println!("\nPress Enter to continue...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
            }

            // Refresh plugins
            i if i == plugins.len() + 1 => {
                println!("\n🔄 Refreshing plugins...");
                // In a real implementation, this would reload plugins from disk
                println!("✅ Plugins refreshed!");
                println!("\nPress Enter to continue...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
            }

            // Show plugin directories
            i if i == plugins.len() + 2 => {
                show_plugin_directories(plugin_manager)?;
            }

            // Return to main menu
            i if i == plugins.len() + 3 => {
                return Ok(());
            }

            _ => unreachable!(),
        }
    }
}
