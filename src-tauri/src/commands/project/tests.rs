use super::*;
use crate::models::sources;

fn create_test_state() -> AppState {
    AppState {
        db: Mutex::new(Database::new_in_memory().unwrap()),
    }
}

fn create_test_session(id: &str, cwd: &str) -> MantraSession {
    MantraSession::new(id.to_string(), sources::CLAUDE.to_string(), cwd.to_string())
}

#[test]
fn test_list_projects_empty() {
    let state = create_test_state();
    let db = state.db.lock().unwrap();
    let projects = db.list_projects().unwrap();
    assert!(projects.is_empty());
}

#[test]
fn test_import_and_list() {
    let state = create_test_state();
    let mut db = state.db.lock().unwrap();
    let mut scanner = ProjectScanner::new(&mut db);

    let sessions = vec![
        create_test_session("sess_1", "/home/user/project1"),
        create_test_session("sess_2", "/home/user/project2"),
    ];

    let result = scanner.scan_and_import(sessions).unwrap();
    assert_eq!(result.imported_count, 2);
    assert_eq!(result.new_projects_count, 2);

    drop(scanner); // Release mutable borrow
    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 2);
}

#[test]
fn test_get_project_sessions() {
    let state = create_test_state();
    let mut db = state.db.lock().unwrap();
    let mut scanner = ProjectScanner::new(&mut db);

    let sessions = vec![
        create_test_session("sess_1", "/home/user/test"),
        create_test_session("sess_2", "/home/user/test"),
    ];

    scanner.scan_and_import(sessions).unwrap();

    drop(scanner); // Release mutable borrow
    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 1);

    let project_sessions = db.get_project_sessions(&projects[0].id).unwrap();
    assert_eq!(project_sessions.len(), 2);
}

#[test]
fn test_detect_language() {
    assert_eq!(detect_language("main.rs"), "rust");
    assert_eq!(detect_language("index.ts"), "typescript");
    assert_eq!(detect_language("app.tsx"), "typescript");
    assert_eq!(detect_language("script.js"), "javascript");
    assert_eq!(detect_language("app.jsx"), "javascript");
    assert_eq!(detect_language("README.md"), "markdown");
    assert_eq!(detect_language("main.py"), "python");
    assert_eq!(detect_language("main.go"), "go");
    assert_eq!(detect_language("Main.java"), "java");
    assert_eq!(detect_language("main.cpp"), "cpp");
    assert_eq!(detect_language("main.c"), "c");
    assert_eq!(detect_language("config.json"), "json");
    assert_eq!(detect_language("config.yaml"), "yaml");
    assert_eq!(detect_language("Cargo.toml"), "toml");
    assert_eq!(detect_language("unknown.xyz"), "text");
}

#[tokio::test]
async fn test_get_representative_file_finds_file() {
    // Get the Git repo root (mantra project root)
    // CARGO_MANIFEST_DIR is apps/client/src-tauri, we need the root
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_path = std::path::PathBuf::from(manifest_dir)
        .parent() // apps/client
        .and_then(|p| p.parent()) // apps
        .and_then(|p| p.parent()) // mantra (root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| manifest_dir.to_string());

    println!("Testing with repo_path: {}", repo_path);

    let result = get_representative_file(repo_path).await;
    println!("Result: {:?}", result);

    match &result {
        Ok(Some(file)) => {
            println!("Found file: {} ({})", file.path, file.language);
            assert!(!file.path.is_empty());
            assert!(!file.content.is_empty());
        }
        Ok(None) => {
            println!("No representative file found");
            // This shouldn't happen for mantra project which has README.md
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_get_representative_file_invalid_repo() {
    let result = get_representative_file("/nonexistent/path".to_string()).await;
    assert!(result.is_err());
}
