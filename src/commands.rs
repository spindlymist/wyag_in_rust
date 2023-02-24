#![allow(dead_code)]

use std::{
    path::PathBuf,
    collections::HashSet,
};
use clap::{Parser, Subcommand, Args};

use crate::{
    Error,
    Result,
    repo::Repository,
    object::{
        GitObject,
        ObjectHash,
        ObjectFormat,
        Commit,
        Tag,
        ObjectMetadata,
    },
    refs,
    index::Index,
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
   Tag(TagArgs),
}

#[derive(clap::ValueEnum, Clone)]
enum ClapObjectFormat {
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
    path: PathBuf,
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
    delete: bool,
    branch_name: Option<String>,
    #[arg(default_value = "HEAD")]
    start_point: String,
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
    object_type: ClapObjectFormat,

    /// The object to display
    object: String,
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
    commit: String,
    /// The EMPTY directory to checkout on.
    path: PathBuf,
}

pub fn cmd_checkout(args: CheckoutArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let hash = GitObject::find(repo.workdir(), &args.commit)?;
    let mut object = GitObject::read(repo.workdir(), &hash)?;
    
    if let GitObject::Commit(commit) = object {
        let tree_hash = match commit.map.get("tree") {
            Some(val) => ObjectHash::try_from(&val[..])?,
            None => return Err(Error::BadCommitFormat),
        };
        object = GitObject::read(repo.workdir(), &tree_hash)?;
    }
    
    if let GitObject::Tree(tree) = object {
        if args.path.is_file() {
            return Err(Error::InitPathIsFile);
        }
        else if args.path.is_dir()
             && args.path.read_dir()?.next().is_some()
        {
            return Err(Error::InitDirectoryNotEmpty);
        }
        else {
            std::fs::create_dir(&args.path)?;
        }

        tree.checkout(repo.workdir(), args.path)
    }
    else {
        Err(Error::ObjectNotTree)
    }
}

/// Commits staged changes to the current branch.
#[derive(Args)]
pub struct CommitArgs {
    /// A message to attach to the tag.
    #[arg(short, default_value = "")]
    message: String,
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
    write: bool,

    /// The type of the object
    #[arg(id = "type", short, long, default_value = "blob")]
    format: ClapObjectFormat,

    /// Path to read the object from
    path: PathBuf,
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
    path: Option<PathBuf>,
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
    commit: String,
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

    let commit = GitObject::read(wd, hash)?;

    if let GitObject::Commit(commit) = commit {
        for parent in commit.map.get_all("parent") {
            let parent_hash = ObjectHash::try_from(&parent[..])?;
            println!("c_{hash} -> c_{parent_hash}");
            log_graphviz(wd, &parent_hash, seen)?;
        }

        Ok(())
    }
    else {
        Err(Error::NonCommitInGraph)
    }
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

    for entry in index.entries.values() {
        println!("{} {}", entry.hash, entry.path.to_string_lossy());
    }

    Ok(())
}

/// Pretty-print a tree object.
#[derive(Args)]
pub struct LsTreeArgs {
    /// The tree object to display.
    object: String,
}

pub fn cmd_ls_tree(args: LsTreeArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let hash = GitObject::find(repo.workdir(), &args.object)?;
    let tree = match GitObject::read(repo.workdir(), &hash)? {
        GitObject::Tree(tree) => tree,
        _ => return Err(Error::ObjectNotTree),
    };

    for entry in &tree.entries {
        let object = GitObject::read(repo.workdir(), &entry.hash)?;
        println!("{:0>6} {} {}\t{}", entry.mode, object.get_format(), entry.hash, entry.name);
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
    name: String,
}

pub fn cmd_rev_parse(args: RevParseArgs) -> Result<()> {
    let repo = Repository::find(".")?;
    let hashes = match GitObject::find(repo.workdir(), &args.name) {
        Ok(hash) => vec![hash],
        Err(err) => match err {
            Error::BadObjectId => vec![],
            Error::AmbiguousObjectId(candidates) => candidates,
            _ => return Err(err),
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

#[derive(Args)]
pub struct RmArgs {
    
}

pub fn cmd_rm(_args: RmArgs) -> Result<()> {
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
    annotate: bool,

    /// Delete the tag.
    #[arg(short, long)]
    delete: bool,

    /// The new tag's name.
    name: Option<String>,

    /// The object the new tag will point to.
    #[arg(default_value = "HEAD")]
    object: String,

    /// A message to attach to the tag.
    #[arg(short, default_value = "")]
    message: String,
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
