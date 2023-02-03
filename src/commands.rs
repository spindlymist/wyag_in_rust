#![allow(dead_code)]

use std::{
    path::PathBuf,
    collections::HashSet,
};
use clap::{Parser, Subcommand, Args};

use crate::{
    error::Error,
    repo::{GitRepository, repo_find, repo_open_file},
    object::{
        GitObject,
        ObjectHash,
        ObjectFormat,
        object_read,
        object_write,
        object_find,
        tree_checkout,
        tag_create,
        tag_create_lightweight,
        object_hash,
        commit_create,
    },
    refs::ref_list,
    index::{index_parse, index_add, index_write},
    branch::{branch_delete, branch_create},
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

impl Into<ObjectFormat> for ClapObjectFormat {
    fn into(self) -> ObjectFormat {
        use ClapObjectFormat::*;

        match self {
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

pub fn cmd_add(args: AddArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;
    let mut index = {
        let index_file = repo_open_file(&repo, "index", None)?;
        let mut buf_reader = std::io::BufReader::new(index_file);

        index_parse(&mut buf_reader)?
    };

    if index.ext_data.len() > 0 {
        eprintln!("Warning: index contains unsupported extensions.");
    }

    index_add(&mut index, &repo, &args.path)?;
    index_write(&index, &repo)?;

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

pub fn cmd_branch(args: BranchArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;
    if let Some(branch_name) = args.branch_name {
        if args.delete {
            branch_delete(&branch_name, &repo)?;
        }
        else {
            let hash = object_find(&repo, &args.start_point)?;
            branch_create(&branch_name, &repo, &hash)?;
        }
    }
    else {
        ref_list(&repo)?.iter()
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

pub fn cmd_cat_file(args: CatFileArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;
    let hash = object_find(&repo, &args.object)?;
    let object = object_read(&repo, &hash)?;

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

pub fn cmd_checkout(args: CheckoutArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;
    let hash = object_find(&repo, &args.commit)?;
    let mut object = object_read(&repo, &hash)?;
    
    if let GitObject::Commit(commit) = object {
        let tree_hash = match commit.map.get("tree") {
            Some(val) => ObjectHash::try_from(&val[..])?,
            None => return Err(Error::BadCommitFormat),
        };
        object = object_read(&repo, &tree_hash)?;
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

        tree_checkout(&repo, &tree, args.path)
    }
    else {
        Err(Error::ObjectNotTree)
    }
}

#[derive(Args)]
pub struct CommitArgs {
    
}

pub fn cmd_commit(_args: CommitArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;
    let index = {
        let index_file = repo_open_file(&repo, "index", None)?;
        let mut buf_reader = std::io::BufReader::new(index_file);

        index_parse(&mut buf_reader)?
    };

    let hash = commit_create(&index, &repo)?;
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

pub fn cmd_hash_object(args: HashObjectArgs) -> Result<(), Error> {
    let object = GitObject::from_path(args.path, args.format.into())?;
    let hash;

    if args.write {
        let repo = repo_find(".")?;
        hash = object_write(&repo, &object)?;
    }
    else {
        hash = object_hash(&object);
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

/// Display history of a given commit.
#[derive(Args)]
pub struct LogArgs {
    /// The commit to start at.
    #[arg(default_value = "HEAD")]
    commit: String,
}

pub fn cmd_log(args: LogArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;

    println!("digraph wyaglog{{");
    let hash = object_find(&repo, &args.commit)?;
    log_graphviz(&repo, &hash, &mut HashSet::new())?;
    println!("}}");

    Ok(())
}

fn log_graphviz<'a>(repo: &GitRepository, hash: &'a ObjectHash, seen: &mut HashSet<ObjectHash>) -> Result<(), Error> {
    if seen.contains(hash) {
        return Ok(());
    }
    seen.insert(*hash);

    let commit = object_read(&repo, &hash)?;

    if let GitObject::Commit(commit) = commit {
        for parent in commit.map.get_all("parent") {
            let parent_hash = ObjectHash::try_from(&parent[..])?;
            println!("c_{} -> c_{}", hash, parent_hash);
            log_graphviz(&repo, &parent_hash, seen)?;
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

pub fn cmd_ls_files(_args: LsFilesArgs) -> Result<(), Error> {
    let index = {
        let repo = repo_find(".")?;
        let index_file = repo_open_file(&repo, "index", None)?;
        let mut buf_reader = std::io::BufReader::new(index_file);

        index_parse(&mut buf_reader)?
    };

    if index.ext_data.len() > 0 {
        eprintln!("Warning: index contains unsupported extensions.");
    }

    for (_, entry) in &index.entries {
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

pub fn cmd_ls_tree(args: LsTreeArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;
    let hash = object_find(&repo, &args.object)?;
    let tree = match object_read(&repo, &hash)? {
        GitObject::Tree(tree) => tree,
        _ => return Err(Error::ObjectNotTree),
    };

    for entry in &tree.entries {
        let object = object_read(&repo, &entry.hash)?;
        println!("{:0>6} {} {}\t{}", entry.mode, object.get_format(), entry.hash, entry.name);
    }

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

/// Determines which object hash a name refers to (if any).
#[derive(Args)]
pub struct RevParseArgs {
    /// The name to parse.
    name: String,
}

pub fn cmd_rev_parse(args: RevParseArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;
    let hashes = match object_find(&repo, &args.name) {
        Ok(hash) => vec![hash],
        Err(err) => match err {
            Error::BadObjectId => vec![],
            Error::AmbiguousObjectId(candidates) => candidates,
            _ => return Err(err),
        },
    };

    match hashes.len() {
        0 => println!(""),
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

pub fn cmd_rm(_args: RmArgs) -> Result<(), Error> {
    Ok(())
}

/// List references.
#[derive(Args)]
pub struct ShowRefArgs {
    // empty
}

pub fn cmd_show_ref(_args: ShowRefArgs) -> Result<(), Error> {
    let repo = repo_find(".")?;
    let refs = ref_list(&repo)?;

    for (name, hash) in refs {
        println!("{hash} {name}");
    }

    Ok(())
}

/// List or create tags.
#[derive(Args)]
pub struct TagArgs {
    /// Create an annotated tag.
    #[arg(short, long)]
    annotate: bool,

    /// The new tag's name.
    name: Option<String>,

    /// The object the new tag will point to.
    #[arg(default_value = "HEAD")]
    object: String,
}

pub fn cmd_tag(args: TagArgs) -> Result<(), Error> {
    if let Some(name) = args.name {
        let repo = repo_find(".")?;
        let hash = object_find(&repo, &args.object)?;

        if args.annotate {
            tag_create(&repo, &name, &hash)?;
        }
        else {
            tag_create_lightweight(&repo, &name, &hash)?;
        }
    }
    else {
        let repo = repo_find(".")?;
        let refs = ref_list(&repo)?;
        let tag_names = refs.iter()
            .filter(|(name, _)| name.starts_with("refs/tags/"))
            .map(|(name, _)| &name["refs/tags/".len()..]);

        for tag_name in tag_names {
            println!("{tag_name}");
        }
    }

    Ok(())
}
