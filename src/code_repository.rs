use regex::Regex;
use git2::{Repository,Oid, DiffDelta, DiffHunk, DiffLine};
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
        let mut parents = commit.parents();
        let diff = if parents.len() == 0 {
            self.repo.diff_tree_to_tree(None, Some(&commit_tree), None).unwrap()
        } else {
            let parent = parents.next().unwrap();
            let parent_tree = parent.tree().unwrap();
            self.repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None).unwrap()
        };
        let mut sum: Vec<String> = vec![];

        // let mut concat_hunks = |delta: DiffDelta, hunk: DiffHunk| -> bool {
        //     let old_file_id = delta.old_file().id();
        //     if  old_file_id.is_zero() {
        //         sum.push("".into())
        //     } else {
        //         let old_file = self.repo.find_blob(old_file_id).unwrap();
        //         let old_file_content =  String::from_utf8_lossy(old_file.content());
        //         let file_content: Vec<&str> = old_file_content.lines().collect();
        //         let start: usize = hunk.old_start() as usize;
        //         let end: usize = start + (hunk.old_lines() as usize);
        //         let changes = &file_content[start..end];
        //         sum.push(changes.join("\n"));
        //     }

        //     true
        // };

        let mut concat_lines = |_delta: DiffDelta, _maybe_hunk: Option<DiffHunk>, line: DiffLine| -> bool {

            if line.origin_value() == git2::DiffLineType::Deletion {
                sum.push(String::from_utf8_lossy(line.content()).to_string());
            }

            true
        };


        diff.foreach(&mut |_,_| {true}, None, None, Some(&mut concat_lines)).unwrap();
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

    #[test]
    fn extract_string_from_commit() {
        let some_repo = CodeRepository::new("../shitty_project").unwrap();
        let commit = git2::Oid::from_str("1a038d4ee7a19fe0eb5a83a5cf3c14109d3669bb").unwrap();
        let changes: String = some_repo.get_changes(commit);
        dbg!(&changes);
        assert!(changes.contains("typedef"));
        assert!(!changes.contains("fp(20)"));
    }

    #[test]
    fn extract_line_from_commit() {
        let some_repo = CodeRepository::new("../shitty_project").unwrap();
        let commit = git2::Oid::from_str("1a038d4ee7a19fe0eb5a83a5cf3c14109d3669bb").unwrap();
        let changes: String = some_repo.get_changes(commit);
        dbg!(&changes);
        assert!(changes.contains("typedef"));
        assert!(!changes.contains("main"));
    }

}
