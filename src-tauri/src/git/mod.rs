//! Git Time Machine 模块
//!
//! 提供根据时间戳查看历史代码状态的功能。
//! 严格只读操作，不修改工作目录，不执行 checkout。

pub mod error;
pub mod time_machine;

pub use error::GitError;
pub use time_machine::{CommitInfo, GitTimeMachine, Snapshot};
