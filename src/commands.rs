#![allow(dead_code)]

use std::{
    path::PathBuf,
};
use clap::{Parser, Subcommand, Args};

use crate::{
    error::Error,
    repo::{GitRepository, repo_find},
    object::{GitObject, ObjectHash, object_read, object_write, object_find},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
   Add(AddArgs),
   CatFile(CatFileArgs),
   Checkout(CheckoutArgs),
   Commit(CommitArgs),
   HashObject(HashObjectArgs),
   Init(InitArgs),
   Log(LogArgs),
   LsFiles(LsFilesArgs),
   LsTree(LsTreeArgs),
   Merge(MergeArgs),
   Rebase(RebaseArgs),
   RevParse(RevParseArgs),
   Rm(RmArgs),
   ShowRef(ShowRefArgs),
   Tag(TagArgs),
}

#[derive(Args)]
pub struct AddArgs {

}

pub fn cmd_add(_args: AddArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(clap::ValueEnum, Clone)]
enum ObjectFormat {
    Commit,
    Tree,
    Tag,
    Blob,
}

/// Displays contents of repository object
#[derive(Args)]
pub struct CatFileArgs {
    /// The type of object to display
    #[arg(id = "TYPE")]
    object_type: ObjectFormat,

    /// The object to display
    object: String,
}

pub fn cmd_cat_file(args: CatFileArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;
    let hash = object_find(&repo, args.object)?;
    let object = object_read(&repo, &hash)?;

    println!("{}", String::from_utf8_lossy(&object.serialize()));

    Ok(())
}

#[derive(Args)]
pub struct CheckoutArgs {
    
}

pub fn cmd_checkout(_args: CheckoutArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct CommitArgs {
    
}

pub fn cmd_commit(_args: CommitArgs) -> Result<(), Error> {
    Ok(())
}

/// Computes object hash and optionally creates a blob from a file.
#[derive(Args)]
pub struct HashObjectArgs {
    /// Actually write the object into the database
    #[arg(short, long)]
    write: bool,

    /// The type of the object
    #[arg(id = "type", short, long)]
    format: Option<ObjectFormat>,

    /// Path to read the object from
    path: PathBuf,
}

pub fn cmd_hash_object(args: HashObjectArgs) -> Result<(), Error> {
    // TODO move some of this logic to object module?
    let format = match args.format.unwrap_or(ObjectFormat::Blob) {
        ObjectFormat::Commit => "commit",
        ObjectFormat::Tree => "tree",
        ObjectFormat::Tag => "tag",
        ObjectFormat::Blob {..} => "blob",
    };
    let data = std::fs::read_to_string(args.path)?.into_bytes();
    let object = GitObject::deserialize(format, data)?;
    let hash;

    if args.write {
        let repo = repo_find(".")?;
        hash = object_write(&repo, &object)?;
    }
    else {
        hash = ObjectHash::new(object.serialize());
    }

    println!("{}", hash);

    Ok(())
}

/// Creates a new git repository.
#[derive(Args)]
pub struct InitArgs {
    /// Where to create the repository.
    path: Option<PathBuf>,
}

pub fn cmd_init(args: InitArgs) -> Result<(), Error> {
    let path = args.path.unwrap_or(PathBuf::from("."));
    GitRepository::init(&path)?;
    
    println!("Successfully initialized git repository at {}", path.to_string_lossy());

    Ok(())
}

#[derive(Args)]
pub struct LogArgs {
    
}

pub fn cmd_log(_args: LogArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct LsFilesArgs {
    
}

pub fn cmd_ls_files(_args: LsFilesArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct LsTreeArgs {
    
}

pub fn cmd_ls_tree(_args: LsTreeArgs) -> Result<(), Error> {
    Ok(())
}


#[derive(Args)]
pub struct MergeArgs {
    
}

pub fn cmd_merge(_args: MergeArgs) -> Result<(), Error> {
    Ok(())
}


#[derive(Args)]
pub struct RebaseArgs {
    
}

pub fn cmd_rebase(_args: RebaseArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct RevParseArgs {
    
}

pub fn cmd_rev_parse(_args: RevParseArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct RmArgs {
    
}

pub fn cmd_rm(_args: RmArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct ShowRefArgs {
    
}

pub fn cmd_show_ref(_args: ShowRefArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct TagArgs {
    
}

pub fn cmd_tag(_args: TagArgs) -> Result<(), Error> {
    Ok(())
}
