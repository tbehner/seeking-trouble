use regex::Regex;
use git2::{Repository,Oid, DiffDelta, DiffHunk};
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

    pub fn contains_pattern(&self, oid: Oid, patterns: &[Regex]) -> bool {
        let commit = self.repo.find_commit(oid).unwrap();
        let commit_message = commit.message().unwrap();
        patterns.iter().find(|p| p.find(&commit_message).is_some()).is_some()
    }

    pub fn commits_matching(&self, patterns: &[Regex]) -> Result<Vec<Oid>,CodeRepositoryError> {
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

    pub fn get_changes(&self, commit_id: Oid) -> String {
        let commit = self.repo.find_commit(commit_id).unwrap();
        let commit_tree = commit.tree().unwrap();

        let diff = self.repo.diff_tree_to_tree(None, Some(&commit_tree), None).unwrap();
        let mut sum: Vec<String> = vec![];

        let mut concat_hunks = |delta: DiffDelta, _hunk: DiffHunk| -> bool {
            let old_file_id = delta.old_file().id();
            if  old_file_id.is_zero() {
                sum.push("".into())
            } else {
                let old_file =  self.repo.find_blob(old_file_id).unwrap();
                let file_content = old_file.content();
                sum.push(String::from_utf8_lossy(&file_content).to_string());
            }

            true
        };


        diff.foreach(&mut |_,_| {true}, None, Some(&mut concat_hunks), None).unwrap();
        sum.join("")
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

    #[test]
    fn extract_empty_string_from_initial_commit_adding_empty_file() {
        let some_repo = CodeRepository::new("../shitty_test_project").unwrap();
        let commit = git2::Oid::from_str("b2d9ff0faf5d9f201849485f96962e9facaa1428").unwrap();
        let changes: String = some_repo.get_changes(commit);
        assert!(changes.is_empty());
    }
    

    #[test]
    fn extract_empty_string_from_initial_commit_adding_nonempty_file() {
        let some_repo = CodeRepository::new(".").unwrap();
        let commit = git2::Oid::from_str("a26f0fcc8faea89939859ebba4e51265ba415db0").unwrap();
        let changes: String = some_repo.get_changes(commit);
        dbg!(&changes);
        assert!(changes.is_empty());
    }
}
