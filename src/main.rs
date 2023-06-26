pub mod data_interface;
mod model;
mod data_interfaces {
    pub mod data_interface_sqlite;
}
mod clwm;
mod clwm_error;
mod clwm_file;
mod command_macros;

use chrono::TimeZone;
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
    Find {
        #[command(subcommand)]
        command: FindSubcommands,
    },
    Update {
        #[command(subcommand)]
        command: UpdateSubcommands,
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

#[derive(Subcommand)]
enum FindSubcommands {
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

#[derive(Subcommand)]
enum UpdateSubcommands {
    Noun {
        id: i64,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        r#type: Option<String>,
        #[arg(short, long)]
        metadata: Option<String>,
    },
    NounType {
        id: i64,
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
            }
        },
        Commands::Find { command } => match command {
            FindSubcommands::Noun {
                name,
                r#type,
                metadata,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                for noun in clwm.get_all_nouns().await?.iter() {
                    println!(
                        "{}. {} {} {}",
                        noun.noun_id.unwrap(),
                        noun.name,
                        noun.noun_type,
                        noun.last_changed.unwrap().to_rfc3339()
                    );
                }
            }
            FindSubcommands::NounType { r#type, metadata } => {
                let mut clwm = get_clwm(&cli).await?;
                for noun_type in clwm.get_all_noun_types().await?.iter() {
                    println!(
                        "{}. {} {}",
                        noun_type.noun_type_id.unwrap(),
                        noun_type.noun_type,
                        noun_type.last_changed.unwrap().to_rfc3339()
                    );
                }
            }
        },
        Commands::Update { command } => match command {
            UpdateSubcommands::Noun {
                id,
                name,
                r#type,
                metadata,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(mut noun) = clwm.get_noun_by_id(*id).await? {
                    if let Some(name_change) = name {
                        noun.name = name_change.to_string();
                    }
                    if let Some(noun_type_change) = r#type {
                        noun.noun_type = noun_type_change.to_string();
                    }
                    if let Some(metadata_change) = metadata {
                        noun.metadata = metadata_change.to_string();
                    }

                    let updated_noun = clwm.update_noun(noun).await?;

                    println!("Updated {:?}", updated_noun);
                } else {
                    println!("No noun exists with id {}", id);
                }
            }
            UpdateSubcommands::NounType {
                id,
                r#type,
                metadata,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(mut noun_type) = clwm.get_noun_type_by_id(*id).await? {
                    if let Some(noun_type_change) = r#type {
                        noun_type.noun_type = noun_type_change.to_string();
                    }
                    if let Some(metadata_change) = metadata {
                        noun_type.metadata = metadata_change.to_string();
                    }

                    let updated_noun_type = clwm.update_noun_type(noun_type).await?;

                    println!("Updated {:?}", updated_noun_type);
                } else {
                    println!("No noun exists with id {}", id);
                }
            }
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
