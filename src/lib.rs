pub type Result<T> = anyhow::Result<T>;

pub mod commands;
pub use commands::Cli;

pub mod branch;
pub mod index;
pub mod kvlm;
pub mod object;
pub mod refs;
pub mod repo;
pub mod workdir;

pub fn run(cli: Cli) {
    use commands::*;

    let result = match cli.command {
        Commands::Add(args) => cmd_add(args),
        Commands::Branch(args) => cmd_branch(args),
        Commands::CatFile(args) => cmd_cat_file(args),
        Commands::Checkout(args) => cmd_checkout(args),
        Commands::Commit(args) => cmd_commit(args),
        Commands::HashObject(args) => cmd_hash_object(args),
        Commands::Init(args) => cmd_init(args),
        Commands::Log(args) => cmd_log(args),
        Commands::LsFiles(args) => cmd_ls_files(args),
        Commands::LsTree(args) => cmd_ls_tree(args),
        Commands::Merge(args) => cmd_merge(args),
        Commands::Restore(args) => cmd_restore(args),
        Commands::RevParse(args) => cmd_rev_parse(args),
        Commands::Rm(args) => cmd_rm(args),
        Commands::ShowRef(args) => cmd_show_ref(args),
        Commands::Status(args) => cmd_status(args),
        Commands::Switch(args) => cmd_switch(args),
        Commands::Tag(args) => cmd_tag(args),
    };

    if let Err(err) = result {
        eprintln!("{err}");
    }
}
