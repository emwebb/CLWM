mod data_interface;
mod model;
mod data_interfaces {
    pub mod data_interface_sqlite;
}
mod clwm;
mod clwm_error;
mod clwm_file;
mod command_macros;

use std::{
    fs::File,
    io::Read,
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use clwm::Clwm;
use data_interface::DataInterfaceType;
use model::{DataObject, DataTypeDefinition};

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
    Get {
        #[command(subcommand)]
        command: GetSubcommands,
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
    AttributeType {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        data_type: Option<String>,
        #[arg(short = 'u', long)]
        multiple_allowed: Option<bool>,
        #[arg(short, long)]
        metadata: Option<String>,
    },
    Attribute {
        #[arg(short = 'o', long)]
        parent_noun_id: Option<i64>,
        #[arg(short = 't', long)]
        parent_attribute_id: Option<i64>,
        #[arg(short, long)]
        attribute_type_id: Option<i64>,
        #[arg(short, long)]
        data: Option<PathBuf>,
        #[arg(short = 'v', long)]
        data_type_version: Option<i64>,
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
    },
    NounType {
        #[arg(short, long)]
        r#type: Option<String>,
    },
    DataType {
        #[arg(short, long)]
        name: Option<String>,
    },
    AttributeType {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        data_type: Option<String>,
    },
    Attribute {
        #[arg(short = 'o', long)]
        parent_noun_id: Option<i64>,
        #[arg(short = 't', long)]
        parent_attribute_id: Option<i64>,
        #[arg(short, long)]
        attribute_type_id: Option<i64>,
        #[arg(short, long)]
        data: Option<PathBuf>,
        #[arg(short = 'v', long)]
        data_type_version: Option<i64>,
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
    AttributeType {
        id: i64,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short = 'u', long)]
        multiple_allowed: Option<bool>,
        #[arg(short, long)]
        metadata: Option<String>,
    },
    Attribute {
        id: i64,
        #[arg(short, long)]
        data: Option<PathBuf>,
        #[arg(short = 'v', long)]
        data_type_version: Option<i64>,
        #[arg(short, long)]
        metadata: Option<String>,
    },
}

#[derive(Subcommand)]
enum GetSubcommands {
    Noun { id: i64 },
    NounType { id: i64 },
    DataType { name: String },
    AttributeType { id: i64 },
    Attribute { id: i64 },
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
                    None => open_editor("toml".to_string())?,
                };
                let defintion = toml::from_str::<DataTypeDefinition>(&defintion_string)?;
                let data_type = clwm.new_data_type(name, defintion).await?;
                println!("New {:?}", data_type);
            }
            NewSubcommands::AttributeType {
                name,
                data_type,
                multiple_allowed,
                metadata,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                let name = arg_input!(name, "What is the name of this attribute type?:");
                let data_type =
                    arg_input!(data_type, "What is the data type of this attribute type?:");
                let multiple_allowed = arg_input!(
                    multiple_allowed,
                    "Can this attribute type have multiple values?:"
                )
                .parse::<bool>()?;
                let metadata =
                    arg_input!(metadata, "What is the metadata of this attribute type?:");
                let attribute_type = clwm
                    .new_attribute_type(name, multiple_allowed, data_type, metadata)
                    .await?;
                println!("New {:?}", attribute_type);
            }
            NewSubcommands::Attribute {
                parent_noun_id,
                parent_attribute_id,
                attribute_type_id,
                data,
                data_type_version,
                metadata,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                let attribute_type_id =
                    arg_input!(attribute_type_id, "What is the id of the attribute type?:")
                        .parse::<i64>()?;
                let (parent_noun_id, parent_attribute_id) = if parent_noun_id.is_none()
                    && parent_attribute_id.is_none()
                {
                    let parent_type = arg_input!(
                        None::<String>,
                        "What is the parent of this attribute? (Attribute/Noun):"
                    );
                    if parent_type.to_lowercase() == "attribute" {
                        (
                            None,
                            Some(
                                arg_input!(
                                    parent_attribute_id,
                                    "What is the id of the parent attribute?:"
                                )
                                .parse::<i64>()?,
                            ),
                        )
                    } else if parent_type.to_lowercase() == "noun" {
                        (
                            Some(
                                arg_input!(parent_noun_id, "What is the id of the parent noun?:")
                                    .parse::<i64>()?,
                            ),
                            None,
                        )
                    } else {
                        println!("Invalid parent type");
                        return Ok(());
                    }
                } else {
                    (*parent_noun_id, *parent_attribute_id)
                };
                let data_string = match data {
                    Some(data) => read_file(data.to_path_buf())?,
                    None => open_editor("toml".to_string())?,
                };

                let data = toml::from_str::<DataObject>(&data_string)?;
                let data_type_version =
                    arg_input!(data_type_version, "What is the version of the data type?:")
                        .parse::<i64>()?;
                let metadata = arg_input!(metadata, "What is the metadata of this attribute?:");
                let attribute = clwm
                    .new_attribute(
                        attribute_type_id,
                        parent_noun_id,
                        parent_attribute_id,
                        data,
                        data_type_version,
                        metadata,
                    )
                    .await?;
                println!("New {:?}", attribute);
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
            }
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
            FindSubcommands::AttributeType { name, data_type } => {
                let mut clwm = get_clwm(&cli).await?;
                for attribute_type in clwm.get_all_attribute_types().await?.iter() {
                    println!(
                        "{}. {} {} {} {}",
                        attribute_type.attribute_type_id.unwrap(),
                        attribute_type.attribute_name,
                        attribute_type.data_type,
                        attribute_type.multiple_allowed,
                        attribute_type.last_changed.unwrap().to_rfc3339()
                    );
                }
            }
            FindSubcommands::Attribute {
                parent_noun_id,
                parent_attribute_id,
                attribute_type_id,
                data,
                data_type_version,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                for attribute in clwm.get_all_attributes().await?.iter() {
                    println!(
                        "{}. {} {} {} {} {} {}",
                        attribute.attribute_id.unwrap(),
                        attribute.attribute_type_id,
                        attribute.parent_noun_id.unwrap_or(0),
                        attribute.parent_attribute_id.unwrap_or(0),
                        attribute.data_type_version,
                        attribute.last_changed.unwrap().to_rfc3339(),
                        attribute.metadata
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
            UpdateSubcommands::DataType { name, defintion } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(mut data_type) =
                    clwm.get_latest_data_type_by_name(name.to_string()).await?
                {
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
            UpdateSubcommands::AttributeType {
                id,
                name,
                multiple_allowed,
                metadata,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(mut attribute_type) = clwm.get_attribute_type_by_id(*id).await? {
                    if let Some(name_change) = name {
                        attribute_type.attribute_name = name_change.to_string();
                    }
                    if let Some(multiple_allowed_change) = multiple_allowed {
                        attribute_type.multiple_allowed = *multiple_allowed_change;
                    }
                    if let Some(metadata_change) = metadata {
                        attribute_type.metadata = metadata_change.to_string();
                    }

                    let updated_attribute_type = clwm.update_attribute_type(attribute_type).await?;

                    println!("Updated {:?}", updated_attribute_type);
                } else {
                    println!("No attribute type exists with id {}", id);
                }
            }
            UpdateSubcommands::Attribute {
                id,
                data,
                data_type_version,
                metadata,
            } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(mut attribute) = clwm.get_attribute_by_id(*id).await? {
                    if let Some(data_change) = data {
                        let data_string = read_file(data_change.to_path_buf())?;
                        let data = toml::from_str::<DataObject>(&data_string)?;
                        attribute.data = data;
                    }
                    if let Some(data_type_version_change) = data_type_version {
                        attribute.data_type_version = *data_type_version_change;
                    }
                    if let Some(metadata_change) = metadata {
                        attribute.metadata = metadata_change.to_string();
                    }

                    let updated_attribute = clwm.update_attribute(attribute).await?;

                    println!("Updated {:?}", updated_attribute);
                } else {
                    println!("No attribute exists with id {}", id);
                }
            }
        },
        Commands::Get { command } => match command {
            GetSubcommands::Noun { id } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(mut noun) = clwm.get_noun_by_id(*id).await? {
                    clwm.populate_noun(&mut noun).await?;
                    println!("{}", toml::to_string(&noun)?);
                } else {
                    println!("No noun exists with id {}", id);
                }
            }
            GetSubcommands::NounType { id } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(noun_type) = clwm.get_noun_type_by_id(*id).await? {
                    println!("{}", toml::to_string(&noun_type)?);
                } else {
                    println!("No noun type exists with id {}", id);
                }
            }
            GetSubcommands::DataType { name } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(data_type) = clwm
                    .get_all_data_type_by_name(name.to_string())
                    .await?
                    .last()
                {
                    println!("{}", toml::to_string(&data_type)?);
                } else {
                    println!("No data type exists with name {}", name);
                }
            }
            GetSubcommands::AttributeType { id } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(attribute_type) = clwm.get_attribute_type_by_id(*id).await? {
                    println!("{}", toml::to_string(&attribute_type)?);
                } else {
                    println!("No attribute type exists with id {}", id);
                }
            }
            GetSubcommands::Attribute { id } => {
                let mut clwm = get_clwm(&cli).await?;
                if let Some(mut attribute) = clwm.get_attribute_by_id(*id).await? {
                    clwm.populate_attribute(&mut attribute).await?;
                    println!("{}", toml::to_string(&attribute)?);
                } else {
                    println!("No attribute exists with id {}", id);
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
        .read_to_string(&mut file_content)?;
    Ok(file_content)
}
