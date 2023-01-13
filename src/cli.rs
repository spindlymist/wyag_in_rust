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

#[derive(Args)]
pub struct CatFileArgs {
    
}

#[derive(Args)]
pub struct CheckoutArgs {
    
}

#[derive(Args)]
pub struct CommitArgs {
    
}

#[derive(Args)]
pub struct HashObjectArgs {
    
}

#[derive(Args)]
pub struct InitArgs {
    
}

#[derive(Args)]
pub struct LogArgs {
    
}

#[derive(Args)]
pub struct LsFilesArgs {
    
}

#[derive(Args)]
pub struct LsTreeArgs {
    
}

#[derive(Args)]
pub struct MergeArgs {
    
}

#[derive(Args)]
pub struct RebaseArgs {
    
}

#[derive(Args)]
pub struct RevParseArgs {
    
}

#[derive(Args)]
pub struct RmArgs {
    
}

#[derive(Args)]
pub struct ShowRefArgs {
    
}

#[derive(Args)]
pub struct TagArgs {
    
}
