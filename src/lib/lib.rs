use git2::Error;
use git2::Repository;
use std::collections::HashMap;
use std::path;
use walkdir::WalkDir;

pub struct RemoteSwapConfig {
    dry_run: bool,
    root: path::PathBuf,
    remote_mapping: HashMap<String, String>,
}

impl RemoteSwapConfig {
    pub fn new(
        dry_run: bool,
        root: path::PathBuf,
        remote_mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            dry_run: dry_run,
            root: root,
            remote_mapping: remote_mapping,
        }
    }
}

pub fn is_likely_git_repo(dir: &path::PathBuf) -> bool {
    dir.ends_with(".git") || (dir.join("refs").is_dir() && dir.join("config").is_file())
}

pub fn visit_git_repos(root: &path::PathBuf) -> impl Iterator<Item = Repository> {
    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|dir| dir.ok())
        .map(|dir| dir.path().to_path_buf())
        .filter(|p| is_likely_git_repo(p))
        .map(|p| Repository::open(p))
        .filter_map(|repo| repo.ok())
}

pub fn run(config: RemoteSwapConfig) -> Result<(), Error> {
    println!("=================================");
    if config.dry_run {
        println!("DRY RUN: No changes will be made.");
    }
    println!("Searching for git repos under: {:?}", config.root);
    println!(
        "Config file has {} remote replacements",
        config.remote_mapping.len()
    );
    println!("=================================");

    for repo in visit_git_repos(&config.root) {
        println!("Found Repo: {:?}", repo.path());

        let remotes = repo.remotes()?;

        let swappable = remotes
            .iter()
            .filter_map(|name| name)
            .filter_map(|name| {
                repo.find_remote(name)
                    .ok()
                    .and_then(|r| r.url().map(|url| (name, url.to_string())))
            })
            .filter_map(|(name, url)| {
                config
                    .remote_mapping
                    .get(&url)
                    .map(|new_url| (name, url, new_url))
            });

        let mut any = false;
        for (name, url, new_url) in swappable {
            any = true;
            println!("  - remote: {}", name);
            println!("    - found url:  {}", url);
            println!("    - changed to: {}", new_url);

            if !config.dry_run {
                repo.remote_set_url(name, new_url)?;
            }
        }
        if !any {
            println!("  - No remotes needed swapping")
        }
    }
    Ok(())
}

#[allow(dead_code, unused_imports)]
mod tests {
    use super::*;
    use git2::Repository;
    use std::collections::HashSet;
    use std::{fs, io, path};
    use tempfile::Builder;

    enum RepoType {
        None,
        WorkDir,
        Bare,
    }

    #[test]
    fn test_is_likely_git_repo_none() {
        test_is_likely_git_repo(RepoType::None, false)
    }

    #[test]
    fn test_is_likely_git_repo_workdir() {
        with_temp_dir(|temp_path| {
            let dir = temp_path.join("a_git_repo");
            make_git_repo(RepoType::WorkDir, &dir)?;
            assert_eq!(is_likely_git_repo(&dir.join(".git")), true);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_is_likely_git_repo_bare() {
        test_is_likely_git_repo(RepoType::Bare, true)
    }

    #[test]
    fn test_visit_sub_repos() {
        with_temp_dir(|temp_path| {
            make_git_repo(RepoType::None, &temp_path.join("not_a_repo"))?;
            make_git_repo(RepoType::WorkDir, &temp_path.join("a_repo"))?;
            make_git_repo(RepoType::Bare, &temp_path.join("a_bare_repo"))?;
            make_git_repo(
                RepoType::WorkDir,
                &temp_path
                    .join("a")
                    .join("deeply")
                    .join("nested")
                    .join("repo"),
            )?;
            make_git_repo(
                RepoType::Bare,
                &temp_path
                    .join("another")
                    .join("deeply")
                    .join("nested")
                    .join("bare_repo"),
            )?;

            let visited: HashSet<path::PathBuf> = visit_git_repos(&temp_path)
                .map(|r| r.path().to_path_buf())
                .collect();
            let assert_path_visited = |p: &path::PathBuf| {
                assert_eq!(
                    visited.contains(p),
                    true,
                    "Expected {:?} to be in list of visited: {:?}",
                    p,
                    visited
                )
            };

            assert_path_visited(&temp_path.join("a_repo").join(".git"));
            assert_path_visited(&temp_path.join("a_bare_repo"));
            assert_path_visited(
                &temp_path
                    .join("a")
                    .join("deeply")
                    .join("nested")
                    .join("repo")
                    .join(".git"),
            );
            assert_path_visited(
                &temp_path
                    .join("another")
                    .join("deeply")
                    .join("nested")
                    .join("bare_repo"),
            );

            assert_eq!(
                visited.len(),
                4,
                "There exist more folders interpreted as repos than expected"
            );

            Ok(())
        })
        .unwrap();
    }

    fn with_temp_dir<T>(run_test: T) -> io::Result<()>
    where
        T: FnOnce(path::PathBuf) -> io::Result<()>,
    {
        let tmp_dir = Builder::new()
            .prefix("git-remote-swap-testrun-")
            .tempdir()?;
        run_test(tmp_dir.path().to_path_buf())?;
        tmp_dir.close()
    }
    fn make_git_repo(which: RepoType, at: &path::PathBuf) -> io::Result<()> {
        fs::create_dir_all(at)?;
        match which {
            RepoType::None => {
                ();
            }
            RepoType::WorkDir => {
                Repository::init(at).unwrap();
            }
            RepoType::Bare => {
                Repository::init_bare(at).unwrap();
            }
        };

        Ok(())
    }

    fn test_is_likely_git_repo(repo_type: RepoType, expected: bool) {
        with_temp_dir(|temp_path| {
            let dir = temp_path.join("a_git_repo");
            make_git_repo(repo_type, &dir)?;
            assert_eq!(is_likely_git_repo(&dir), expected);
            Ok(())
        })
        .unwrap();
    }
}
