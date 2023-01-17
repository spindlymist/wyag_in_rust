#![allow(dead_code)]

use std::{
    path::PathBuf,
};
use clap::{Parser, Subcommand, Args};

use crate::{
    repo::GitRepository,
    error::Error,
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

pub fn cmd_add(args: AddArgs) -> Result<(), Error> {
    Ok(())
}

/// Displays contents of repository object
#[derive(Args)]
pub struct CatFileArgs {
    
}

pub fn cmd_cat_file(args: CatFileArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct CheckoutArgs {
    
}

pub fn cmd_checkout(args: CheckoutArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct CommitArgs {
    
}

pub fn cmd_commit(args: CommitArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct HashObjectArgs {
    
}

pub fn cmd_hash_object(args: HashObjectArgs) -> Result<(), Error> {
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

pub fn cmd_log(args: LogArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct LsFilesArgs {
    
}

pub fn cmd_ls_files(args: LsFilesArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct LsTreeArgs {
    
}

pub fn cmd_ls_tree(args: LsTreeArgs) -> Result<(), Error> {
    Ok(())
}


#[derive(Args)]
pub struct MergeArgs {
    
}

pub fn cmd_merge(args: MergeArgs) -> Result<(), Error> {
    Ok(())
}


#[derive(Args)]
pub struct RebaseArgs {
    
}

pub fn cmd_rebase(args: RebaseArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct RevParseArgs {
    
}

pub fn cmd_rev_parse(args: RevParseArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct RmArgs {
    
}

pub fn cmd_rm(args: RmArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct ShowRefArgs {
    
}

pub fn cmd_show_ref(args: ShowRefArgs) -> Result<(), Error> {
    Ok(())
}

#[derive(Args)]
pub struct TagArgs {
    
}

pub fn cmd_tag(args: TagArgs) -> Result<(), Error> {
    Ok(())
}
