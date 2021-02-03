use git2::Repository;
use std::path;
use walkdir::WalkDir;

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

#[allow(dead_code)]
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
            let assert_path_visited = |p: &path::PathBuf| assert_eq!(visited.contains(p), true);

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
