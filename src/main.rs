pub mod data_interface;
mod model;
mod data_interfaces {
    pub mod data_interface_sqlite;
}
mod clwm;
mod clwm_error;
mod clwm_file;
mod command_macros;

use clap::{Parser, Subcommand};
use clwm::Clwm;
use data_interface::DataInterfaceType;

#[derive(Parser)] // requires `derive` feature
#[command(name = "clwm")]
#[command(about = "Command Line World Manager", long_about = None)]
struct Cli {
    #[arg(short, long)]
    file: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Create {
        filename: String,
        data_interface: DataInterfaceType,
        url: String,
    },
    New {
        #[command(subcommand)]
        command: NewSubcommands,
    },
}

#[derive(Subcommand)]
enum NewSubcommands {
    Noun {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        r#type: Option<String>,
        #[arg(short, long)]
        metadata: Option<String>,
    },
    NounType {
        #[arg(short, long)]
        r#type: Option<String>,
        #[arg(short, long)]
        metadata: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create {
            filename,
            data_interface,
            url,
        } => {
            Clwm::create(*data_interface, url.to_string(), filename.to_string()).await?;
        }
        Commands::New { command } => match command {
            NewSubcommands::Noun {
                name,
                r#type,
                metadata,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                let name = arg_input!(name, "What is the name of this noun?:");
                let noun_type = arg_input!(r#type, "What is the type of this noun?:");
                let metadata = arg_input!(metadata, "What is the metadata of this noun?:");
                let noun = clwm.new_noun(name, noun_type, metadata).await?;
                println!("New {:?}", noun);
            }
            NewSubcommands::NounType { r#type, metadata } => {
                let mut clwm = get_clwm(&cli).await?;
                let noun_type = arg_input!(r#type, "What is the name of this noun type?:");
                let metadata = arg_input!(metadata, "What is the metadata of this noun type?");
                let noun_type = clwm.new_noun_type(noun_type, metadata).await?;
                println!("New {:?}", noun_type);
            },
        },
    }
    Ok(())
}

async fn get_clwm(cli: &Cli) -> anyhow::Result<Clwm> {
    let file_name = if cli.file.is_some() {
        cli.file.as_ref().unwrap().to_string()
    } else {
        "world.clwm".to_string()
    };
    Clwm::new(file_name).await
}
