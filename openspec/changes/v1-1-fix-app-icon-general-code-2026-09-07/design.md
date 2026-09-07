# Design

## Icon diagnosis

构建后检查发现 0.2.0 的 Info.plist 不含 `CFBundleIconFile`，`.app/Contents/Resources` 目录也不存在。根因是 `tauri.conf.json` 的 bundle 未显式设置 icon，而不是 Finder 缓存。

通过 bundle icon 列表声明 PNG、ICNS 和 ICO。`scripts/verify-bundle-icon.sh` 在应用包边界校验 plist 声明、文件存在性和 ICNS 类型。

## General code

词典正文解析器将编码列视为任意非 Tab 文本。新增或编辑时只拒绝空编码及 Tab/CR/LF，因为这些字符会破坏 TSV 行结构。其他字符和大小写均原样保留。
