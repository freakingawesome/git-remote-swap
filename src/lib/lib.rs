use std::{path, io};
use walkdir::{WalkDir};

pub fn is_likely_git_repo(dir: &path::PathBuf) -> bool {
    dir.join(".git").is_dir()
        || (dir.join("refs").is_dir() && dir.join("config").is_file())
}

pub fn walk_git_repos(root: &path::PathBuf) -> impl Iterator<Item=path::PathBuf> {
    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|dir| dir.ok())
        .map(|dir| dir.path().to_path_buf())
        .filter(|p| is_likely_git_repo(p))
}

#[allow(dead_code)]
mod tests {
    use std::collections::HashSet;
use std::collections::HashMap;
use git2::Repository;
    use super::*;

    use tempfile::Builder;
    use std::{fs, io, path};
    
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
        test_is_likely_git_repo(RepoType::WorkDir, true)
    }
    
    #[test]
    fn test_is_likely_git_repo_bare() {
        test_is_likely_git_repo(RepoType::Bare, true)
    }
    
    #[test]
    fn visit_sub_repos() {
        with_temp_dir(|temp_path| {
            // let mut repos: Vec<&str> = Vec::new();

            make_git_repo(RepoType::None, &temp_path.join("not_a_repo"))?;
            make_git_repo(RepoType::WorkDir, &temp_path.join("a_repo"))?;
            make_git_repo(RepoType::Bare, &temp_path.join("a_bare_repo"))?;
            make_git_repo(RepoType::WorkDir, &temp_path.join("a").join("deeply").join("nested").join("repo"))?;
            make_git_repo(RepoType::Bare, &temp_path.join("another").join("deeply").join("nested").join("bare_repo"))?;
            
            // for repo_path in walk_git_repos(temp_path).map(|p| String::from(p.as_path().to_str())) {
            let walked: HashSet<String> = walk_git_repos(&temp_path).map(|p| p.into_os_string().into_string()).filter_map(|p| p.ok()).collect();
            
            // assert_eq!(walked.contains(&format!("{}/{}", &temp_path.display(), "not_a_repo")), true);
            let temp_path_base: String = temp_path.into_os_string().into_string().unwrap();

            // println!("DBG: {}", temp_path_base);
            // for x in &walked {
            //     println!("DBG2: {}", &x);
            // }

            assert_eq!(walked.contains(format!("{}/a_repo", &temp_path_base).as_str()), true);
            assert_eq!(walked.contains(format!("{}/a_bare_repo", &temp_path_base).as_str()), true);
            assert_eq!(walked.contains(format!("{}/a/deeply/nested/repo", &temp_path_base).as_str()), true);
            assert_eq!(walked.contains(format!("{}/another/deeply/nested/bare_repo", &temp_path_base).as_str()), true);
            assert_eq!(walked.len(), 4, "There exist more folders interpretted as repos than expected");

            
            // walked.filter
            // walked.
            // walked.remove_item(temp_path.join("not_a_repo").into_os_string().into_string());
                /*
            visit_git_repos(temp_path, move |p| {
                let cloned_path: &str = p.to_str().unwrap().clone();
                repos.push(&cloned_path);
            })?;
            */
            //visit_git_repos(temp_path, |p: &path::PathBuf| {repos.push(Path::new(p.to_str()));});

            Ok(())
        }).unwrap();
    }
    
    fn with_temp_dir<T>(run_test: T) -> io::Result<()>
        where T: FnOnce(path::PathBuf) -> io::Result<()>
    {
        let tmp_dir = Builder::new().prefix("git-remote-swap-testrun-").tempdir()?;
        run_test(tmp_dir.path().to_path_buf())?;
        tmp_dir.close()
    }
    
    fn make_git_repo(which: RepoType, at: &path::PathBuf) -> io::Result<()> {
        fs::create_dir_all(at)?;
        
        match which {
            RepoType::None => {();},
            RepoType::WorkDir => {Repository::init(at).unwrap();},
            RepoType::Bare => {Repository::init_bare(at).unwrap();},
        };

        Ok(())
    }

    fn test_is_likely_git_repo(repo_type: RepoType, expected: bool) {
        with_temp_dir(|temp_path| {
            let dir = temp_path.join("a_git_repo");
            make_git_repo(repo_type, &dir)?;
            assert_eq!(is_likely_git_repo(&dir), expected);
            Ok(())
        }).unwrap();
    }
}