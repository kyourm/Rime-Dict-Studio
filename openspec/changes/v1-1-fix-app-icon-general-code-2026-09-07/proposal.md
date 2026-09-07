# Bundle the app icon and accept general Rime codes

## Why

0.2.0 安装包虽然生成了图标源文件，但 Tauri bundle 没有显式声明图标，导致 macOS 应用包缺少 `CFBundleIconFile` 和 Resources 图标。词条表单及领域校验仍错误地把编码限定为五笔字母。

## What Changes

- 在 Tauri bundle 中显式声明跨平台图标资源。
- 增加应用包图标声明和 ICNS 有效性的构建后检查。
- 将词条“编码”改为 Rime 通用编码，只限制为空或含 Tab/换行。
- 保留用户输入编码的大小写和符号，不自动转小写。

## Capabilities

- `dictionary-editing`：正确打包应用图标并支持通用 Rime 编码。

### Clarifications

- 用户指定继续使用上一版已设计的图标，不需要重新设计。
- 用户要求新增词条仅称为“编码”，不限制为五笔编码。
