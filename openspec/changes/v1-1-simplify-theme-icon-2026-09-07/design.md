# Design

## Single-file workspace

移除侧栏、目录按钮和所有文件列表状态。主界面只显示一个当前文件条、搜索与新增工具条、词条表格及保存区。已有大词典分批渲染策略不变。

## Appearance

CSS 使用语义变量表达背景、表面、正文、弱文本、边框和强调色，不使用渐变。未设置 `data-theme` 时通过 `prefers-color-scheme` 跟随系统；显式浅色或深色模式覆盖系统。偏好以受校验 JSON 保存到 WebView localStorage，损坏时安全回退到系统模式。

## Icon

图标采用无文字的“词典页 + 四个编码键位”符号。1024×1024 源图进入项目，Tauri CLI 生成 macOS `.icns`、Windows `.ico` 和各平台 PNG。
