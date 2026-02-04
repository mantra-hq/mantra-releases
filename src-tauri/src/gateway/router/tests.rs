use super::*;
use rusqlite::params;

fn create_test_db() -> Arc<Database> {
    Arc::new(Database::new_in_memory().unwrap())
}

fn create_test_project(db: &Database, id: &str, name: &str, cwd: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    db.connection()
        .execute(
            "INSERT INTO projects (id, name, cwd, created_at, last_activity) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![id, name, cwd, now],
        )
        .unwrap();
}

fn add_project_path(db: &Database, project_id: &str, path: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    db.connection()
        .execute(
            "INSERT INTO project_paths (id, project_id, path, is_primary, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
            params![id, project_id, path, now],
        )
        .unwrap();
}

#[tokio::test]
async fn test_find_project_exact_match() {
    let db = create_test_db();
    create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
    add_project_path(&db, "proj1", "/home/user/projects/mantra");

    let router = ContextRouter::new(db);
    let result = router
        .find_project_by_path(Path::new("/home/user/projects/mantra"))
        .await;

    assert!(result.is_some());
    let ctx = result.unwrap();
    assert_eq!(ctx.project_id, "proj1");
    assert_eq!(ctx.project_name, "Project 1");
}

#[tokio::test]
async fn test_find_project_prefix_match() {
    let db = create_test_db();
    create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
    add_project_path(&db, "proj1", "/home/user/projects/mantra");

    let router = ContextRouter::new(db);
    // 查询子目录
    let result = router
        .find_project_by_path(Path::new("/home/user/projects/mantra/apps/client"))
        .await;

    assert!(result.is_some());
    let ctx = result.unwrap();
    assert_eq!(ctx.project_id, "proj1");
    assert_eq!(ctx.matched_path, PathBuf::from("/home/user/projects/mantra"));
}

#[tokio::test]
async fn test_find_project_longest_prefix_match() {
    let db = create_test_db();

    // 创建两个项目，一个是另一个的父目录
    create_test_project(&db, "proj1", "Projects Root", "/home/user/projects");
    add_project_path(&db, "proj1", "/home/user/projects");

    create_test_project(&db, "proj2", "Mantra", "/home/user/projects/mantra");
    add_project_path(&db, "proj2", "/home/user/projects/mantra");

    let router = ContextRouter::new(db);

    // 查询 mantra 子目录，应该匹配到 proj2（更长的前缀）
    let result = router
        .find_project_by_path(Path::new("/home/user/projects/mantra/apps/client"))
        .await;

    assert!(result.is_some());
    let ctx = result.unwrap();
    assert_eq!(ctx.project_id, "proj2");
    assert_eq!(ctx.project_name, "Mantra");
    // /home/user/projects/mantra = 26 characters
    assert_eq!(ctx.match_length, 26);
}

#[tokio::test]
async fn test_find_project_no_match() {
    let db = create_test_db();
    create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
    add_project_path(&db, "proj1", "/home/user/projects/mantra");

    let router = ContextRouter::new(db);
    // 查询不相关的路径
    let result = router
        .find_project_by_path(Path::new("/home/other/path"))
        .await;

    assert!(result.is_none());
}

#[tokio::test]
async fn test_find_project_partial_name_no_match() {
    let db = create_test_db();
    create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
    add_project_path(&db, "proj1", "/home/user/projects/mantra");

    let router = ContextRouter::new(db);
    // 查询部分名称匹配但不是前缀的路径
    let result = router
        .find_project_by_path(Path::new("/home/user/projects/mantra-other"))
        .await;

    // 不应该匹配，因为 mantra-other 不是 mantra 的子目录
    assert!(result.is_none());
}

#[tokio::test]
async fn test_cache_hit() {
    let db = create_test_db();
    create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
    add_project_path(&db, "proj1", "/home/user/projects/mantra");

    let router = ContextRouter::new(db);

    // 第一次查询
    let result1 = router
        .find_project_by_path(Path::new("/home/user/projects/mantra"))
        .await;
    assert!(result1.is_some());

    // 第二次查询应该命中缓存
    let result2 = router
        .find_project_by_path(Path::new("/home/user/projects/mantra"))
        .await;
    assert!(result2.is_some());
    assert_eq!(result1.unwrap().project_id, result2.unwrap().project_id);
}

#[tokio::test]
async fn test_clear_cache() {
    let db = create_test_db();
    create_test_project(&db, "proj1", "Project 1", "/home/user/projects/mantra");
    add_project_path(&db, "proj1", "/home/user/projects/mantra");

    let router = ContextRouter::new(db);

    // 查询以填充缓存
    let _ = router
        .find_project_by_path(Path::new("/home/user/projects/mantra"))
        .await;

    // 清除缓存
    router.clear_cache().await;

    // 缓存应该为空
    let cache = router.cache.lock().await;
    assert!(cache.is_empty());
}

#[test]
fn test_normalize_path_trailing_slash() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let normalized = router.normalize_path(Path::new("/home/user/projects/"));
    assert_eq!(normalized, PathBuf::from("/home/user/projects"));
}

#[test]
fn test_normalize_path_no_trailing_slash() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let normalized = router.normalize_path(Path::new("/home/user/projects"));
    assert_eq!(normalized, PathBuf::from("/home/user/projects"));
}

#[test]
fn test_parse_work_dir_from_root_uri() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let params = serde_json::json!({
        "rootUri": "file:///home/user/projects/mantra"
    });

    let result = router.parse_work_dir_from_params(&params);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
}

#[test]
fn test_parse_work_dir_from_workspace_folders() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let params = serde_json::json!({
        "workspaceFolders": [
            {
                "uri": "file:///home/user/projects/mantra",
                "name": "mantra"
            }
        ]
    });

    let result = router.parse_work_dir_from_params(&params);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
}

#[test]
fn test_parse_work_dir_workspace_folders_priority() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    // workspaceFolders 应该优先于 rootUri
    let params = serde_json::json!({
        "rootUri": "file:///other/path",
        "workspaceFolders": [
            {
                "uri": "file:///home/user/projects/mantra",
                "name": "mantra"
            }
        ]
    });

    let result = router.parse_work_dir_from_params(&params);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
}

#[test]
fn test_parse_work_dir_from_root_path() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let params = serde_json::json!({
        "rootPath": "/home/user/projects/mantra"
    });

    let result = router.parse_work_dir_from_params(&params);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects/mantra"));
}

#[test]
fn test_parse_work_dir_no_params() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let params = serde_json::json!({});

    let result = router.parse_work_dir_from_params(&params);
    assert!(result.is_none());
}

#[test]
fn test_uri_to_path_unix() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let result = router.uri_to_path("file:///home/user/projects");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/projects"));
}

#[test]
fn test_uri_to_path_with_spaces() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let result = router.uri_to_path("file:///home/user/my%20projects");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("/home/user/my projects"));
}

#[test]
fn test_uri_to_path_invalid() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let result = router.uri_to_path("http://example.com");
    assert!(result.is_none());
}

#[cfg(target_os = "windows")]
#[test]
fn test_uri_to_path_windows() {
    let db = create_test_db();
    let router = ContextRouter::new(db);

    let result = router.uri_to_path("file:///C:/Users/user/projects");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), PathBuf::from("C:/Users/user/projects"));
}
