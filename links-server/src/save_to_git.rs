use git2::Repository;
use git2::Signature;
use std::path::Path;
use tracing::trace;

pub fn commit(repo_dir: &str, author: &str) -> Result<(), git2::Error> {
    trace!("committing to git repo: {}", repo_dir);
    let repo = Repository::open(&Path::new(repo_dir))?;

    trace!("repo opened");
    let mut index = repo.index()?;
    trace!("index opened");

    index.add_all(["*.md"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    trace!("added all files to index");

    index.write()?;
    trace!("index written");

    let tree_id = index.write_tree()?;
    trace!("tree written");
    let tree = repo.find_tree(tree_id)?;
    trace!("tree found");
    let parent_commit = repo.head()?.peel_to_commit()?;
    trace!("parent commit found");

    let author = Signature::now(author, "_")?;
    trace!("author found");
    repo.commit(
        Some("HEAD"),
        &author,
        &author,
        "saved via gui",
        &tree,
        &[&parent_commit],
    )?;
    Ok(())
}
