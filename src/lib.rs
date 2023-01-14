use std::{
    path::{Path, PathBuf},
    error,
    fmt,
    fs,
    io,
};
use ini::Ini;

mod commands;
use commands::*;
pub use commands::Cli;

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
    pub fn init<P>(dir: P) -> Result<GitRepository, Error>
    where
        P: AsRef<Path>
    {
        let working_dir = PathBuf::from(dir.as_ref());
        if working_dir.is_file() {
            return Err(Error::InitPathIsFile);
        }
        else if working_dir.is_dir() {
            match working_dir.read_dir() {
                Ok(mut files) => {
                    if files.next().is_some() {
                        return Err(Error::InitDirectoryNotEmpty);
                    }
                },
                Err(err) => return Err(err.into()),
            }
        }

        let git_dir = working_dir.join(".git");
        fs::create_dir_all(&git_dir)?;

        let mut config = Ini::new();
        config.with_section(Some("core"))
            .set("repositoryformatversion", "0")
            .set("filemode", "false")
            .set("bare", "false");

        let repo = GitRepository {
            working_dir,
            git_dir,
            config,
        };

        repo_dir(&repo, "branches")?;
        repo_dir(&repo, "objects")?;
        repo_dir(&repo, "refs/tags")?;
        repo_dir(&repo, "refs/heads")?;

        {
            let description_file = repo_file(&repo, "description")?;
            fs::write(description_file, "Unnamed repository; edit this file 'description' to name the repository.\n")?;
        }

        {
            let head_file = repo_file(&repo, "HEAD")?;
            fs::write(head_file, "ref: refs/heads/master\n")?;
        }

        Ok(repo)
    }

    pub fn from_dir<P>(dir: P) -> Result<GitRepository, Error>
    where
        P: AsRef<Path>
    {
        let working_dir = PathBuf::from(dir.as_ref());
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
    InitPathIsFile,
    InitDirectoryNotEmpty,
    FailedToCreateDirectory(io::Error),
    IoError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IoError(value)
    }
}

fn repo_path<P>(repo: &GitRepository, path: P) -> PathBuf
where
    P: AsRef<Path>
{
    repo.git_dir.join(path)
}

fn repo_file<P>(repo: &GitRepository, path: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path>
{
    if let Some(parent_path) = path.as_ref().parent() {
        repo_dir(repo, parent_path)?;
    }

    Ok(repo_path(repo, path))
}

fn repo_dir<P>(repo: &GitRepository, path: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path>
{
    let path = repo_path(&repo, path.as_ref());

    if !path.is_dir() {
        fs::create_dir_all(&path)?;
    }
    
    Ok(path)
}
