// Story 11.7: 托盘错误类型

use thiserror::Error;

/// 托盘相关错误
#[derive(Error, Debug)]
pub enum TrayError {
    #[error("Failed to build tray icon: {0}")]
    BuildError(String),

    #[error("Failed to load icon: {0}")]
    IconLoadError(String),

    #[error("Failed to build menu: {0}")]
    MenuBuildError(String),

    #[error("Failed to update tray: {0}")]
    UpdateError(String),

    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),
}

impl From<TrayError> for crate::error::AppError {
    fn from(err: TrayError) -> Self {
        crate::error::AppError::TrayError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_error_display() {
        let err = TrayError::BuildError("test error".to_string());
        assert_eq!(err.to_string(), "Failed to build tray icon: test error");

        let err = TrayError::IconLoadError("icon not found".to_string());
        assert_eq!(err.to_string(), "Failed to load icon: icon not found");

        let err = TrayError::MenuBuildError("menu error".to_string());
        assert_eq!(err.to_string(), "Failed to build menu: menu error");

        let err = TrayError::UpdateError("update failed".to_string());
        assert_eq!(err.to_string(), "Failed to update tray: update failed");
    }
}
