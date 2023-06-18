pub mod data_interface;
mod model;
mod data_interfaces {
    pub mod data_interface_sqlite;
}
mod command_macros;
mod clwm;
mod clwm_file;

use clap::{Parser, Subcommand};
use clwm::Clwm;
use clwm_file::ClwmFile;
use data_interface::DataInterfaceType;

#[derive(Parser)] // requires `derive` feature
#[command(name = "clwm")]
#[command(about = "Command Line World Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New {
        #[command(subcommand)]
        command: NewSubcommands,
    }
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
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();


    let clwm = Clwm::new("world.clwm".to_string()).await?;

    let mut data_interface = clwm.data_interface;
    data_interface.init().await?;

    match &cli.command {
        Commands::New { command } => {
            match command {
                NewSubcommands::Noun {
                    name,
                    r#type,
                    metadata,
                } => {
                    let name = arg_input!(name, "What is the name of this noun?:");
                    let noun_type = arg_input!(r#type, "What is the type of this noun?:");
                    let metadata = arg_input!(metadata, "What is the metadata of this noun?:");
                    
                    let noun = data_interface.new_noun(name, noun_type, metadata).await?;
                    println!("New {:?}", noun);
                }
            }
        }
    }
    Ok(())
}
