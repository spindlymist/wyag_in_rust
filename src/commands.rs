#![allow(dead_code)]

use clap::{Parser, Subcommand, Args};

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

pub fn cmd_add(args: AddArgs) {
    
}

#[derive(Args)]
pub struct CatFileArgs {
    
}

pub fn cmd_cat_file(args: CatFileArgs) {
    
}

#[derive(Args)]
pub struct CheckoutArgs {
    
}

pub fn cmd_checkout(args: CheckoutArgs) {
    
}

#[derive(Args)]
pub struct CommitArgs {
    
}

pub fn cmd_commit(args: CommitArgs) {
    
}

#[derive(Args)]
pub struct HashObjectArgs {
    
}

pub fn cmd_hash_object(args: HashObjectArgs) {
    
}

#[derive(Args)]
pub struct InitArgs {
    
}

pub fn cmd_init(args: InitArgs) {
    
}

#[derive(Args)]
pub struct LogArgs {
    
}

pub fn cmd_log(args: LogArgs) {
    
}

#[derive(Args)]
pub struct LsFilesArgs {
    
}

pub fn cmd_ls_files(args: LsFilesArgs) {
    
}

#[derive(Args)]
pub struct LsTreeArgs {
    
}

pub fn cmd_ls_tree(args: LsTreeArgs) {
    
}


#[derive(Args)]
pub struct MergeArgs {
    
}

pub fn cmd_merge(args: MergeArgs) {
    
}


#[derive(Args)]
pub struct RebaseArgs {
    
}

pub fn cmd_rebase(args: RebaseArgs) {
    
}

#[derive(Args)]
pub struct RevParseArgs {
    
}

pub fn cmd_rev_parse(args: RevParseArgs) {
    
}

#[derive(Args)]
pub struct RmArgs {
    
}

pub fn cmd_rm(args: RmArgs) {
    
}

#[derive(Args)]
pub struct ShowRefArgs {
    
}

pub fn cmd_show_ref(args: ShowRefArgs) {
    
}

#[derive(Args)]
pub struct TagArgs {
    
}

pub fn cmd_tag(args: TagArgs) {
    
}
