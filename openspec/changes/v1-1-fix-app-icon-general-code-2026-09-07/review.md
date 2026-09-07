# Verification and review

## Standards

初审指出编码被裁剪以及图标检查未接入发布流程。取消编码 trim，并增加 `bundle:macos` 串联打包与图标验证后，复审无 P1/P2。

## Spec

初审发现首尾空格未原样保留。增加先失败后通过的回归用例并修复后，复审无 P1/P2。

## Verification

- 词典、主题和大文件界面测试全部通过。
- TypeScript、Vite、Clippy、Rust 测试及 OpenSpec 严格校验通过。
- macOS 应用包的 Info.plist 声明 `icon.icns`，对应资源存在且格式有效。
