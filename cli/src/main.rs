mod commands;
mod utils;

use clap::{Parser, Subcommand};
use commands::{config, generate, list, update};
use utils::cache::CacheManager;

#[derive(Parser)]
#[command(
    name = "devc",
    author = "SamitoX4",
    version,
    about = "CLI for generating devcontainers",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, help = "Verbose output")]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Generate a devcontainer for your project")]
    Gen {
        #[arg(short = 't', long, help = "Template name (e.g., nodejs, react-native, java)")]
        template: Option<String>,

        #[arg(short = 'n', long, help = "Project name")]
        name: Option<String>,

        #[arg(long, help = "Git user name")]
        git_name: Option<String>,

        #[arg(long, help = "Git user email")]
        git_email: Option<String>,

        #[arg(long, help = "Security mode: developer, secure, root, custom")]
        security_mode: Option<String>,

        #[arg(long, help = "Remote user name (VS Code connection user)")]
        remote_user: Option<String>,

        #[arg(long, help = "Password for the remote user")]
        remote_password: Option<String>,

        #[arg(long, help = "Password for root / container user")]
        container_password: Option<String>,

        #[arg(long, help = "Sudo mode: nopasswd, password, none")]
        sudo_mode: Option<String>,

        #[arg(long, help = "Save generated credentials to a file (path or 'default' for ~/.devc/credentials/<project>.json)")]
        save_credentials: Option<String>,
    },

    #[command(about = "List available templates")]
    List {
        #[arg(short, long, help = "Show template details")]
        detailed: bool,
    },

    #[command(about = "Update templates from remote repository")]
    Update {
        #[arg(short, long, help = "Force update even if up to date")]
        force: bool,
    },

    #[command(about = "Configure Git user for devcontainers")]
    Config {
        #[arg(long, help = "Git user name")]
        git_name: Option<String>,

        #[arg(long, help = "Git user email")]
        git_email: Option<String>,

        #[arg(short, long, help = "Show current configuration")]
        show: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut cache = CacheManager::new()?;

    if cli.verbose {
        println!("Checking for updates...");
    }

    if let Err(e) = cache.check_updates(cli.verbose).await {
        if cli.verbose {
            eprintln!("Update check failed (will use cached): {}", e);
        }
    }

    match cli.command {
        Commands::Gen {
            template,
            name,
            git_name,
            git_email,
            security_mode,
            remote_user,
            remote_password,
            container_password,
            sudo_mode,
            save_credentials,
        } => {
            generate::run(
                template.as_deref(),
                name.as_deref(),
                git_name.as_deref(),
                git_email.as_deref(),
                security_mode.as_deref(),
                remote_user.as_deref(),
                remote_password.as_deref(),
                container_password.as_deref(),
                sudo_mode.as_deref(),
                save_credentials.as_deref(),
                &mut cache,
            )
            .await?;
        }
        Commands::List { detailed } => {
            list::run(&cache, detailed)?;
        }
        Commands::Update { force } => {
            update::run(&cache, force).await?;
        }
        Commands::Config {
            git_name,
            git_email,
            show,
        } => {
            config::run(&mut cache, git_name.as_deref(), git_email.as_deref(), show)?;
        }
    }

    Ok(())
}
