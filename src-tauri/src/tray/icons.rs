// Story 11.7: 托盘图标管理

use tauri::image::Image;

use super::TrayError;

/// 托盘图标状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayIconState {
    /// 正常状态（Gateway 未运行）
    Normal,
    /// 活跃状态（Gateway 运行中）
    Active,
    /// 错误状态
    Error,
}

// 内嵌图标资源
// 使用专用的托盘图标（无边距，在小尺寸下更清晰）
const ICON_BYTES: &[u8] = include_bytes!("../../icons/tray-icon.ico");

/// 加载托盘图标
///
/// 注意：由于图像生成不可用，暂时所有状态使用同一图标
/// 后续可通过准备不同颜色的图标文件来区分状态
pub fn load_icon(_state: TrayIconState) -> Result<Image<'static>, TrayError> {
    // 目前所有状态使用同一图标
    // 通过 tooltip 和菜单状态来区分不同状态
    // 使用 from_bytes 加载图标（自动检测格式）
    Image::from_bytes(ICON_BYTES)
        .map_err(|e: tauri::Error| TrayError::IconLoadError(e.to_string()))
}

/// 获取图标状态的描述
pub fn get_state_description(state: TrayIconState) -> &'static str {
    match state {
        TrayIconState::Normal => "正常",
        TrayIconState::Active => "运行中",
        TrayIconState::Error => "错误",
    }
}

/// 获取图标状态的 emoji
pub fn get_state_emoji(state: TrayIconState) -> &'static str {
    match state {
        TrayIconState::Normal => "⚪",
        TrayIconState::Active => "🟢",
        TrayIconState::Error => "🔴",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_icon_state_equality() {
        assert_eq!(TrayIconState::Normal, TrayIconState::Normal);
        assert_ne!(TrayIconState::Normal, TrayIconState::Active);
        assert_ne!(TrayIconState::Active, TrayIconState::Error);
    }

    #[test]
    fn test_load_icon() {
        // 测试图标加载
        let result = load_icon(TrayIconState::Normal);
        assert!(result.is_ok(), "Should load icon successfully");

        let result = load_icon(TrayIconState::Active);
        assert!(result.is_ok(), "Should load active icon successfully");

        let result = load_icon(TrayIconState::Error);
        assert!(result.is_ok(), "Should load error icon successfully");
    }

    #[test]
    fn test_get_state_description() {
        assert_eq!(get_state_description(TrayIconState::Normal), "正常");
        assert_eq!(get_state_description(TrayIconState::Active), "运行中");
        assert_eq!(get_state_description(TrayIconState::Error), "错误");
    }

    #[test]
    fn test_get_state_emoji() {
        assert_eq!(get_state_emoji(TrayIconState::Normal), "⚪");
        assert_eq!(get_state_emoji(TrayIconState::Active), "🟢");
        assert_eq!(get_state_emoji(TrayIconState::Error), "🔴");
    }
}
