pub mod data_interface;
mod model;
mod data_interfaces {
    pub mod data_interface_sqlite;
}
mod clwm;
mod clwm_error;
mod clwm_file;
mod command_macros;

use std::{
    env::{temp_dir, var},
    fs::File,
    io::Read,
    path::PathBuf,
    process::Command,
};

use clap::{Parser, Subcommand};
use clwm::Clwm;
use data_interface::DataInterfaceType;
use model::DataTypeDefinition;

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
    DataType {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        defintion: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum FindSubcommands {
    Noun {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        r#type: Option<String>,
    },
    NounType {
        #[arg(short, long)]
        r#type: Option<String>,
    },
    DataType {
        #[arg(short, long)]
        name: Option<String>,
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
    DataType {
        name: String,
        #[arg(short, long)]
        defintion: Option<PathBuf>,
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
            NewSubcommands::DataType { name, defintion } => {
                let mut clwm = get_clwm(&cli).await?;
                let name = arg_input!(name, "What is the name of this data type?:");
                let defintion_string = match defintion {
                    Some(defintion) => read_file(defintion.to_path_buf())?,
                    None => open_editor("toml".to_string())?
                };
                let defintion = toml::from_str::<DataTypeDefinition>(&defintion_string)?;
                let data_type = clwm.new_data_type(name, defintion).await?;
                println!("New {:?}", data_type);
            }
        },
        Commands::Find { command } => match command {
            FindSubcommands::Noun { name, r#type } => {
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
            FindSubcommands::NounType { r#type } => {
                let mut clwm = get_clwm(&cli).await?;
                for noun_type in clwm.get_all_noun_types().await?.iter() {
                    println!(
                        "{}. {} {}",
                        noun_type.noun_type_id.unwrap(),
                        noun_type.noun_type,
                        noun_type.last_changed.unwrap().to_rfc3339()
                    );
                }
            },
            FindSubcommands::DataType { name } => {
                let mut clwm = get_clwm(&cli).await?;
                for data_type in clwm.get_all_data_types().await?.iter() {
                    println!(
                        "{}. {} {}",
                        data_type.name,
                        data_type.version.unwrap(),
                        data_type.change_date.unwrap().to_rfc3339()
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
            },
            UpdateSubcommands::DataType {
                name,
                defintion,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(mut data_type) = clwm.get_latest_data_type_by_name(name.to_string()).await? {
                    if let Some(new_definition) = defintion {
                        let definition_string = read_file(new_definition.to_path_buf())?;
                        let definition = toml::from_str::<DataTypeDefinition>(&definition_string)?;
                        data_type.definition = definition;
                    }
                    let updated_data_type = clwm.update_data_type(data_type).await?;

                    println!("Updated {:?}", updated_data_type);
                } else {
                    println!("No data type exists with name {}", name);
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

fn open_editor(extension: String) -> anyhow::Result<String> {
   Ok(edit::edit("")?)
}

fn read_file(file_path: PathBuf) -> anyhow::Result<String> {
    let mut file_content = String::new();
    File::open(file_path)
        .expect("Could not open file")
        .read_to_string(&mut file_content);
    Ok(file_content)
}
