use git2::{Repository, Signature};
use mm_git::{GitError, GitRepository};
use mm_git_git2::{Git2Repository, create_git_service};
use tempfile::TempDir;

fn init_repo(dir: &TempDir) -> Repository {
    let mut opts = git2::RepositoryInitOptions::new();
    opts.initial_head("main");
    let repo = Repository::init_opts(dir.path(), &opts).expect("init repo");
    let sig = Signature::now("Test", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();
    drop(tree);
    repo
}

#[tokio::test]
async fn test_get_status_success() {
    let dir = TempDir::new().unwrap();
    let repo = init_repo(&dir);
    let expected_branch = repo.head().unwrap().shorthand().unwrap().to_string();
    let service = create_git_service();
    let status = service.get_status(dir.path()).await.unwrap();
    assert_eq!(status.branch, expected_branch);
    assert!(!status.is_dirty);
    assert_eq!(status.ahead_by, 0);
    assert_eq!(status.behind_by, 0);
    assert!(status.changed_files.is_empty());
}

#[tokio::test]
async fn test_get_status_invalid_path() {
    let repo = Git2Repository::new();
    let path = std::path::Path::new("/nonexistent/path");
    let result = repo.get_status(path).await;
    assert!(matches!(result, Err(GitError::RepositoryError { .. })));
}
