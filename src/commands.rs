#![allow(dead_code)]

use std::{
    path::PathBuf,
    collections::HashSet,
};
use clap::{Parser, Subcommand, Args};

use crate::{
    Result,
    repo::{Repository, RepoError},
    object::{
        ObjectError,
        GitObject,
        ObjectHash,
        ObjectFormat,
        Commit,
        Tag,
        ObjectMetadata, Tree,
    },
    refs,
    index::{Index, UnstagedChange, StagedChange},
    branch,
    workdir::WorkDir,
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
   Branch(BranchArgs),
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
   Status(StatusArgs),
   Tag(TagArgs),
}

#[derive(clap::ValueEnum, Clone)]
pub enum ClapObjectFormat {
    Commit,
    Tree,
    Tag,
    Blob,
}

impl From<ClapObjectFormat> for ObjectFormat {
    fn from(value: ClapObjectFormat) -> Self {
        use ClapObjectFormat::*;

        match value {
            Commit => ObjectFormat::Commit,
            Tree => ObjectFormat::Tree,
            Tag => ObjectFormat::Tag,
            Blob => ObjectFormat::Blob,
        }
    }
}

/// Adds files to the staging index
#[derive(Args)]
pub struct AddArgs {
    /// The file or directory to stage
    pub path: PathBuf,
}

pub fn cmd_add(args: AddArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let mut index = Index::from_repo(repo.workdir())?;

    if !index.ext_data.is_empty() {
        eprintln!("Warning: index contains unsupported extensions.");
    }

    index.add(repo.workdir(), &args.path)?;
    index.write(repo.workdir())?;

    Ok(())
}

/// Create, list, and delete branches
#[derive(Args)]
pub struct BranchArgs {
    #[arg(short, long)]
    pub delete: bool,
    pub branch_name: Option<String>,
    #[arg(default_value = "HEAD")]
    pub start_point: String,
}

pub fn cmd_branch(args: BranchArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    if let Some(branch_name) = args.branch_name {
        if args.delete {
            branch::delete(&branch_name, repo.workdir())?;
        }
        else {
            let hash = GitObject::find(repo.workdir(), &args.start_point)?;
            branch::create(&branch_name, repo.workdir(), &hash)?;
        }
    }
    else {
        refs::list(repo.workdir())?.iter()
            .filter_map(|(name, _)| name.strip_prefix("refs/heads/"))
            .for_each(|name| println!("{name}"));
    }

    Ok(())
}

/// Displays contents of repository object
#[derive(Args)]
pub struct CatFileArgs {
    /// The type of object to display
    #[arg(id = "TYPE")]
    pub object_type: ClapObjectFormat,

    /// The object to display
    pub object: String,
}

pub fn cmd_cat_file(args: CatFileArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let hash = GitObject::find(repo.workdir(), &args.object)?;
    let object = GitObject::read(repo.workdir(), &hash)?;

    println!("{}", String::from_utf8_lossy(&object.serialize()));

    Ok(())
}

/// Checkout a commit inside of a directory.
#[derive(Args)]
pub struct CheckoutArgs {
    /// The commit or tree to checkout.
    pub commit: String,
    /// The EMPTY directory to checkout on.
    pub path: PathBuf,
}

pub fn cmd_checkout(args: CheckoutArgs) -> Result<()> {
    if !WorkDir::is_valid_path(&args.path)? {
        return Err(RepoError::InitPathExists(args.path).into());
    }

    let repo = Repository::find(".")?;
    let hash = GitObject::find(repo.workdir(), &args.commit)?;
    let tree = Tree::read_from_commit(repo.workdir(), &hash)?;

    std::fs::create_dir(&args.path)?;
    tree.checkout(repo.workdir(), args.path)
}

/// Commits staged changes to the current branch.
#[derive(Args)]
pub struct CommitArgs {
    /// A message to attach to the tag.
    #[arg(short, default_value = "")]
    pub message: String,
}

pub fn cmd_commit(args: CommitArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let index = Index::from_repo(repo.workdir())?;
    let meta = ObjectMetadata::new(&repo, args.message)?;

    let hash = Commit::create(&index, repo.workdir(), meta)?;
    println!("{hash}");

    Ok(())
}

/// Computes object hash and optionally creates a blob from a file.
#[derive(Args)]
pub struct HashObjectArgs {
    /// Actually write the object into the database
    #[arg(short, long)]
    pub write: bool,

    /// The type of the object
    #[arg(id = "type", short, long, default_value = "blob")]
    pub format: ClapObjectFormat,

    /// Path to read the object from
    pub path: PathBuf,
}

pub fn cmd_hash_object(args: HashObjectArgs) -> Result<()> {
    let object = GitObject::from_path(args.path, args.format.into())?;
    let hash = if args.write {
        let repo = Repository::find(".")?;
        object.write(repo.workdir())?
    }
    else {
        object.hash()
    };

    println!("{hash}");

    Ok(())
}

/// Creates a new git repository.
#[derive(Args)]
pub struct InitArgs {
    /// Where to create the repository.
    pub path: Option<PathBuf>,
}

pub fn cmd_init(args: InitArgs) -> Result<()> {
    let path = args.path.unwrap_or(PathBuf::from("."));
    Repository::init(&path)?;
    
    println!("Successfully initialized git repository at {}", path.to_string_lossy());

    Ok(())
}

/// Display history of a given commit.
#[derive(Args)]
pub struct LogArgs {
    /// The commit to start at.
    #[arg(default_value = "HEAD")]
    pub commit: String,
}

pub fn cmd_log(args: LogArgs) -> Result<()> {
    let repo = Repository::find(".")?;

    println!("digraph wyaglog{{");
    let hash = GitObject::find(repo.workdir(), &args.commit)?;
    log_graphviz(repo.workdir(), &hash, &mut HashSet::new())?;
    println!("}}");

    Ok(())
}

fn log_graphviz(wd: &WorkDir, hash: &ObjectHash, seen: &mut HashSet<ObjectHash>) -> Result<()> {
    if seen.contains(hash) {
        return Ok(());
    }
    seen.insert(*hash);

    match GitObject::read(wd, hash)? {
        GitObject::Commit(commit) => {
            for parent_hash in commit.parents() {
                println!("c_{hash} -> c_{parent_hash}");
                log_graphviz(wd, parent_hash, seen)?;
            }
        },
        object => return Err(branch::BranchError::BrokenCommitGraph(object.get_format()).into()),
    };

    Ok(())
}

/// List all the files in the staging index.
#[derive(Args)]
pub struct LsFilesArgs {
    // empty
}

pub fn cmd_ls_files(_args: LsFilesArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let index = Index::from_repo(repo.workdir())?;

    if !index.ext_data.is_empty() {
        eprintln!("Warning: index contains unsupported extensions.");
    }

    for (path, entry) in index.entries {
        println!("{} {}", entry.hash, path);
    }

    Ok(())
}

/// Pretty-print a tree object.
#[derive(Args)]
pub struct LsTreeArgs {
    /// The tree object to display.
    pub object: String,
}

pub fn cmd_ls_tree(args: LsTreeArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let hash = GitObject::find(repo.workdir(), &args.object)?;
    let tree = Tree::read(repo.workdir(), &hash)?;

    for (path, entry) in &tree.entries {
        let object = GitObject::read(repo.workdir(), &entry.hash)?;
        println!("{:0>6} {} {}\t{}", entry.mode, object.get_format(), entry.hash, path);
    }

    Ok(())
}


#[derive(Args)]
pub struct MergeArgs {
    
}

pub fn cmd_merge(_args: MergeArgs) -> Result<()> {
    Ok(())
}


#[derive(Args)]
pub struct RebaseArgs {
    
}

pub fn cmd_rebase(_args: RebaseArgs) -> Result<()> {
    Ok(())
}

/// Determines which object hash a name refers to (if any).
#[derive(Args)]
pub struct RevParseArgs {
    /// The name to parse.
    pub name: String,
}

pub fn cmd_rev_parse(args: RevParseArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let hashes = match GitObject::find(repo.workdir(), &args.name) {
        Ok(hash) => vec![hash],
        Err(err) => match err.downcast::<ObjectError>() {
            Ok(ObjectError::InvalidId(_)) => vec![],
            Ok(ObjectError::AmbiguousId { matches, .. }) => matches,
            Ok(err) => return Err(err.into()),
            Err(err) => return Err(err),
        },
    };

    match hashes.len() {
        0 => println!(),
        1 => println!("{}", hashes[0]),
        n => {
            println!("{} is ambiguous: {n} matches", args.name);
            for hash in hashes {
                println!("{hash}");
            }
        }
    };

    Ok(())
}

/// Removes files from the staging index and file system
#[derive(Args)]
pub struct RmArgs {
    /// The file or directory to remove. Must match index and branch tip.
    pub path: PathBuf,
}

pub fn cmd_rm(args: RmArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let mut index = Index::from_repo(repo.workdir())?;

    if !index.ext_data.is_empty() {
        eprintln!("Warning: index contains unsupported extensions.");
    }

    index.remove(repo.workdir(), &args.path)?;
    index.write(repo.workdir())?;

    Ok(())
}

/// List staged and unstaged changes 
#[derive(Args)]
pub struct StatusArgs {
    /// The file or directory to compare
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

pub fn cmd_status(args: StatusArgs) -> Result<()> {
    let (staged_changes, unstaged_changes) = {
        let repo = Repository::find(".")?;
        let wd = repo.workdir();
        let path = wd.canonicalize_path(args.path)?;
        let index = Index::from_repo(wd)?;
        let commit_hash = branch::get_current(wd)?.tip(wd)?;
        
        (
            index.list_staged_changes(wd, &commit_hash, &path)?,
            index.list_unstaged_changes(wd, &path, false)?
        )
    };

    if !staged_changes.is_empty() {
        println!("Changes staged for commit:");
        for change in staged_changes {
            match change {
                StagedChange::Created { path } =>  println!("created:   {path}"),
                StagedChange::Modified { path } => println!("modified:  {path}"),
                StagedChange::Deleted { path } =>  println!("deleted:   {path}"),
            };
        }
    }
    else {
        println!("No changes staged for commit");
    }

    if !unstaged_changes.is_empty() {
        println!("Changes not staged for commit:");
        for change in unstaged_changes {
            match change {
                UnstagedChange::Created { path, .. } => println!("created:   {path}"),
                UnstagedChange::Modified { path, ..} => println!("modified:  {path}"),
                UnstagedChange::Deleted { path }     => println!("deleted:   {path}"),
            };
        }
    }
    else {
        println!("No unstaged changes");
    }

    Ok(())
}

/// List references.
#[derive(Args)]
pub struct ShowRefArgs {
    // empty
}

pub fn cmd_show_ref(_args: ShowRefArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let refs = refs::list(repo.workdir())?;

    for (name, hash) in refs {
        println!("{hash} {name}");
    }

    Ok(())
}

/// List, create, or delete tags.
#[derive(Args)]
pub struct TagArgs {
    /// Create an annotated tag.
    #[arg(short, long)]
    pub annotate: bool,

    /// Delete the tag.
    #[arg(short, long)]
    pub delete: bool,

    /// The new tag's name.
    pub name: Option<String>,

    /// The object the new tag will point to.
    #[arg(default_value = "HEAD")]
    pub object: String,

    /// A message to attach to the tag.
    #[arg(short, default_value = "")]
    pub message: String,
}

pub fn cmd_tag(args: TagArgs) -> Result<()> {
    if let Some(name) = args.name {
        let repo = Repository::find(".")?;

        if args.delete {
            Tag::delete(repo.workdir(), &name)?;
        }
        else{
            // Create a tag
            let hash = GitObject::find(repo.workdir(), &args.object)?;
            let meta = ObjectMetadata::new(&repo, args.message)?;

            if args.annotate {
                Tag::create(repo.workdir(), &name, &hash, meta)?;
            }
            else {
                Tag::create_lightweight(repo.workdir(), &name, &hash)?;
            }
        }
    }
    else {
        // List existing tags
        let repo = Repository::find(".")?;
        let refs = refs::list(repo.workdir())?;
        let tag_names = refs.iter()
            .filter(|(name, _)| name.starts_with("refs/tags/"))
            .map(|(name, _)| &name["refs/tags/".len()..]);

        for tag_name in tag_names {
            println!("{tag_name}");
        }
    }

    Ok(())
}
