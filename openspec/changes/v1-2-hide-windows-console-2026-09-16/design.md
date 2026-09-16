# Design

在 Rust 二进制入口使用条件 crate 属性：仅当目标为 Windows 且不是 `debug_assertions` 构建时指定 `windows_subsystem = "windows"`。该配置在编译期生效，不影响 macOS、Linux 或 Windows 调试构建。

