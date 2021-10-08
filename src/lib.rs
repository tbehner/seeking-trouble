use regex::Regex;
use git2::Repository;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeRepositoryError {
    #[error("data store disconnected")]
    Open(#[from] git2::Error),
}

pub struct CodeRepository {
    repo: Repository,
}

impl CodeRepository{
    pub fn new(path: &str) -> Result<CodeRepository,CodeRepositoryError> {
        Ok(CodeRepository{repo: Repository::open(path)?})
    }

    pub fn contains_pattern(&self, oid: git2::Oid, patterns: &[Regex]) -> bool {
        let commit = self.repo.find_commit(oid).unwrap();
        let commit_message = commit.message().unwrap();
        patterns.iter().find(|p| p.find(&commit_message).is_some()).is_some()
    }

    pub fn commits_matching(&self, patterns: &[Regex]) -> Result<Vec<git2::Oid>,CodeRepositoryError> {
        let mut walk = self.repo.revwalk()?;
        match walk.push_head() {
            Ok(_) => {
                Ok(walk
                    .filter(|or| or.is_ok()).map(|or| or.unwrap())
                    .filter(|oid| self.contains_pattern(*oid, patterns))
                    .collect())
            },
            Err(_) => {
                Ok(vec![])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn open_repository() {
        let repository = CodeRepository::new("../shitty_project");
        assert!(repository.is_ok());
    }

    #[test]
    fn find_no_commits_on_empty_repository() {
        let empty_repo = CodeRepository::new("../shitty_empty_project");
        let patterns: Vec<Regex> = vec![];
        assert_eq!(empty_repo.unwrap().commits_matching(&patterns).unwrap().len(), 0);
    }

    fn number_of_commits_in_this_repo() -> usize {
        let stdout = process::Command::new("git")
            .args(&["rev-list", "--all", "--count"])
            .output()
            .expect("working git command").stdout;
        String::from_utf8_lossy(&stdout).trim().parse().unwrap()
    }

    #[test]
    fn find_all_commits_on_this_repo_with_matchall_pattern() {
        let some_repo = CodeRepository::new(".").unwrap();
        let patterns = vec![Regex::new(".*").unwrap()];
        assert_eq!(some_repo.commits_matching(&patterns).unwrap().len(), number_of_commits_in_this_repo());
    }

    #[test]
    fn commits_are_filtered_with_patterns() {
        let some_repo = CodeRepository::new(".").unwrap();
        let patterns = vec![Regex::new("Initial").unwrap()];
        assert!(some_repo.commits_matching(&patterns).unwrap().len() < number_of_commits_in_this_repo());
    }
}
