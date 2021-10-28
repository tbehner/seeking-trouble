use regex::Regex;
use git2::{Repository,Oid, DiffDelta, DiffHunk, DiffLine};
use thiserror::Error;
use std::ops::Range;
use std::collections::HashMap;
use crate::change_set::ChangeSet;


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

        let mut concat_lines = |_delta: DiffDelta, _maybe_hunk: Option<DiffHunk>, line: DiffLine| -> bool {

            if line.origin_value() == git2::DiffLineType::Deletion {
                sum.push(String::from_utf8_lossy(line.content()).to_string());
            }

            true
        };


        diff.foreach(&mut |_,_| {true}, None, None, Some(&mut concat_lines)).unwrap();
        sum.join("")
    }

    pub fn get_change_sets(&self, commit_id: Oid) -> Vec<ChangeSet> {
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

        let mut sum: HashMap<String, ChangeSet> = HashMap::new();

        let mut concat_lines = |_delta: DiffDelta, _maybe_hunk: Option<DiffHunk>, line: DiffLine| -> bool {

            if line.origin_value() == git2::DiffLineType::Deletion {

                //sum.push(String::from_utf8_lossy(line.content()).to_string());
                // check if the filename has a ChangeSet
                //      if not create one, with the respective content
                // add the linenumber to the ChangeSet
            }

            true
        };


        diff.foreach(&mut |_,_| {true}, None, None, Some(&mut concat_lines)).unwrap();
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;
    use tempdir::TempDir;
    use anyhow::Result;
    use std::path::Path;
    use git2::Repository;
    use std::fs::File;
    use std::io::prelude::*;
    use indoc::indoc;

    fn with_empty_repo(test: fn(&Path) -> ()) -> Result<()>{
        let repo_dir = TempDir::new("empty_repository")?;
        let _repo = Repository::init(repo_dir.path())?;

        test(repo_dir.path());

        Ok(())
    }

    fn commit_file(repo_dir: &Path, filename: &str, content: &str, msg: &str) -> Result<()>{
        let mut new_file = File::create(repo_dir.join(filename))?;
        new_file.write(content.as_bytes())?;

        process::Command::new("git")  
            .args(&["add", filename])
            .current_dir(repo_dir)
            .output()?;
        process::Command::new("git")  
            .args(&["commit", "-a", "-m", msg])
            .current_dir(repo_dir)
            .output()?;
        Ok(())
    }

    fn create_temporary_repository() -> Result<TempDir> {
        let repo_dir = TempDir::new("buggy_repository")?;
        Repository::init(repo_dir.path())?;
        Ok(repo_dir)
    }

    fn with_repo_containing_bugs(test: fn(&Path) -> ()) -> Result<()> {
        let repo_dir = create_temporary_repository()?;
        commit_file(repo_dir.path(), "foo", "", "Create foo")?;

        test(repo_dir.path());

        Ok(())
    }

    fn with_repo_containing_function_pointer_bug(test: fn(&Path) -> ()) -> Result<()> {
        let repo_dir = create_temporary_repository()?;
        let buggy_code = indoc! {r#"
            #include <stdio.h>

            void foo(int i) {
              printf("%i\n", i);
            }

            typedef void (*fpt)(unsigned int i);

            int main() {
              fpt fp;

              fp = foo;

              foo(10);
              fp(20);
            }
        "#};

        let fixed_bug = indoc!{r#"
            #include <stdio.h>

            void foo(int i) {
              printf("%i\n", i);
            }

            typedef void (*fpt)(int i);

            int main() {
              fpt fp;

              fp = foo;

              foo(10);
              fp(20);
            }
        "#};

        commit_file(repo_dir.path(), "main.c", buggy_code, "this should work!")?;
        commit_file(repo_dir.path(), "main.c", fixed_bug, "fixed bug")?;

        test(repo_dir.path());

        Ok(())

    }

    #[test]
    fn open_repository() -> Result<()> {
        with_empty_repo(|repo_path: &Path| {
            let repository = CodeRepository::new(repo_path.to_str().unwrap());
            assert!(repository.is_ok());
        })
    }

    #[test]
    fn find_no_commits_on_empty_repository() -> Result<()> {
        with_empty_repo(|repo_path: &Path| {
            let empty_repo = CodeRepository::new(repo_path.to_str().unwrap());
            let patterns: Vec<Regex> = vec![];
            assert_eq!(empty_repo.unwrap().commits_matching(&patterns).unwrap().len(), 0);
        })
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

    fn initial_commit(repo_path: &Path) -> String {
        let stdout = process::Command::new("git")
            .args(&["rev-list", "--max-parents=0", "HEAD"])
            .current_dir(repo_path)
            .output()
            .expect("querying first commit id")
            .stdout;
        String::from_utf8_lossy(&stdout).trim().to_string()
    }

    #[test]
    fn extract_empty_string_from_initial_commit_adding_empty_file() -> Result<()> {
        with_repo_containing_bugs(|path: &Path| {
            let some_repo = CodeRepository::new(path.to_str().unwrap()).unwrap();
            let commit_id = initial_commit(path);
            dbg!(&commit_id);
            let commit = git2::Oid::from_str(&commit_id).unwrap();
            let changes: String = some_repo.get_changes(commit);
            assert!(changes.is_empty());
        })
    }
    
    #[test]
    fn extract_empty_string_from_initial_commit_adding_nonempty_file() {
        let some_repo = CodeRepository::new(".").unwrap();
        let commit = git2::Oid::from_str("a26f0fcc8faea89939859ebba4e51265ba415db0").unwrap();
        let changes: String = some_repo.get_changes(commit);
        dbg!(&changes);
        assert!(changes.is_empty());
    }

    fn get_last_commit(repo_path: &str) -> String {
        let stdout = process::Command::new("git")
            .args(&["rev-list", "--max-count=1", "master"])
            .current_dir(repo_path)
            .output()
            .expect("querying the last commit on master")
            .stdout;
        String::from_utf8_lossy(&stdout).trim().to_string()
    }

    #[test]
    fn extract_string_from_commit() -> Result<()>{
        with_repo_containing_function_pointer_bug(|project_path| {
            let project_path_str = project_path.to_str().unwrap();
            let some_repo = CodeRepository::new(project_path_str).unwrap();
            let commit = git2::Oid::from_str(&get_last_commit(project_path_str)).unwrap();
            let changes: String = some_repo.get_changes(commit);
            dbg!(&changes);
            assert!(changes.contains("typedef"));
            assert!(!changes.contains("fp(20)"));
        })?;
        Ok(())
    }

    #[test]
    fn extract_line_from_commit() -> Result<()> {
        with_repo_containing_function_pointer_bug(|project_path| {
            let prj_str = project_path.to_str().unwrap();
            let some_repo = CodeRepository::new(prj_str).unwrap();
            let commit = git2::Oid::from_str(&get_last_commit(prj_str)).unwrap();
            let changes: String = some_repo.get_changes(commit);
            dbg!(&changes);
            assert!(changes.contains("typedef"));
            assert!(!changes.contains("main"));
        })?;
        Ok(())
    }

    #[test]
    #[ignore]
    fn extract_line_from_commit_to_changeset() -> Result<()> {
        with_repo_containing_function_pointer_bug(|project_path| {
            let prj_str = project_path.to_str().unwrap();
            let some_repo = CodeRepository::new(prj_str).unwrap();
            let commit = git2::Oid::from_str(&get_last_commit(prj_str)).unwrap();
            let changes: Vec<ChangeSet> = some_repo.get_change_sets(commit);
            let expected_line: usize = 6;
            assert!(changes.iter().find(|cs| cs.code.contains("typedef") && cs.ranges().iter().find(|r| r.contains(&expected_line)).is_some()).is_some())
        })?;
        Ok(())
    }


}
