## ADDED Requirements

### Requirement: Windows release console visibility

应用 MUST 在 Windows 正式构建中使用 GUI 子系统，并在调试构建中保留控制台子系统。

#### Scenario: Launch a Windows release build

- **WHEN** 用户启动 Windows 正式安装版
- **THEN** 系统只显示应用窗口，不额外显示命令行窗口

#### Scenario: Launch a Windows debug build

- **WHEN** 开发者启动 Windows 调试构建
- **THEN** 系统保留控制台窗口和诊断输出

