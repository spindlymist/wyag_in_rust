pub mod cli;
pub use cli::Cli;

use std::{
    path::{Path, PathBuf},
    error,
    fmt,
};
use cli::*;
use ini::Ini;

pub fn run(cli: Cli) {
    match cli.command {
        Commands::Add(args) => cmd_add(args),
        Commands::CatFile(args) => cmd_cat_file(args),
        Commands::Checkout(args) => cmd_checkout(args),
        Commands::Commit(args) => cmd_commit(args),
        Commands::HashObject(args) => cmd_hash_object(args),
        Commands::Init(args) => cmd_init(args),
        Commands::Log(args) => cmd_log(args),
        Commands::LsFiles(args) => cmd_ls_files(args),
        Commands::LsTree(args) => cmd_ls_tree(args),
        Commands::Merge(args) => cmd_merge(args),
        Commands::Rebase(args) => cmd_rebase(args),
        Commands::RevParse(args) => cmd_rev_parse(args),
        Commands::Rm(args) => cmd_rm(args),
        Commands::ShowRef(args) => cmd_show_ref(args),
        Commands::Tag(args) => cmd_tag(args),
    };
}

pub struct GitRepository {
    working_dir: PathBuf,
    git_dir: PathBuf,
    config: Ini,
}

impl GitRepository {
    pub fn from_dir(dir: &Path) -> Result<GitRepository, Error> {
        let working_dir = PathBuf::from(dir);
        if !working_dir.is_dir() {
            return Err(Error::WorkingDirectoryInvalid);
        }

        let git_dir = working_dir.join(".git");
        if !git_dir.is_dir() {
            return Err(Error::DirectoryNotInitialized);
        }

        let config_file = git_dir.join("config");
        let config = match Ini::load_from_file(config_file) {
            Ok(cfg) => cfg,
            Err(err) => return Err(Error::FailedToLoadConfig(err.to_string())),
        };

        match config.get_from(Some("core"), "repositoryformatversion") {
            Some("0") => (),
            Some(version) => return Err(Error::UnsupportedRepoFmtVersion(String::from(version))),
            None => return Err(Error::RepoFmtVersionMissing),
        };

        Ok(GitRepository {
            working_dir,
            git_dir,
            config,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    WorkingDirectoryInvalid,
    DirectoryNotInitialized,
    FailedToLoadConfig(String),
    RepoFmtVersionMissing,
    UnsupportedRepoFmtVersion(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

fn cmd_add(args: AddArgs) {
    
}

fn cmd_cat_file(args: CatFileArgs) {
    
}

fn cmd_checkout(args: CheckoutArgs) {
    
}

fn cmd_commit(args: CommitArgs) {
    
}

fn cmd_hash_object(args: HashObjectArgs) {
    
}

fn cmd_init(args: InitArgs) {
    
}

fn cmd_log(args: LogArgs) {
    
}

fn cmd_ls_files(args: LsFilesArgs) {
    
}

fn cmd_ls_tree(args: LsTreeArgs) {
    
}

fn cmd_merge(args: MergeArgs) {
    
}

fn cmd_rebase(args: RebaseArgs) {
    
}

fn cmd_rev_parse(args: RevParseArgs) {
    
}

fn cmd_rm(args: RmArgs) {
    
}

fn cmd_show_ref(args: ShowRefArgs) {
    
}

fn cmd_tag(args: TagArgs) {
    
}
