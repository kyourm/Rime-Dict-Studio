## ADDED Requirements

### Requirement: Bundled application icon

应用 MUST 在各平台安装包中声明并包含已设计的词典编辑器图标。

#### Scenario: Build a macOS application

- **WHEN** 构建 macOS `.app`
- **THEN** Info.plist 包含 `CFBundleIconFile`，且对应 ICNS 文件存在并有效

## MODIFIED Requirements

### Requirement: Edit dictionary entries

应用 MUST 支持搜索、新增和修改由词组、通用 Rime 编码、可选权重构成的词条。

#### Scenario: Add a valid phrase

- **WHEN** 用户输入非空词组、不含 Tab/换行的非空编码及合法权重并确认
- **THEN** 应用原样保留编码的大小写与符号，将词条加入内存模型，并可通过搜索找到

#### Scenario: Reject an exact duplicate

- **WHEN** 用户新增与现有词组和编码都相同的词条
- **THEN** 应用阻止操作并说明重复原因

#### Scenario: Load a large dictionary

- **WHEN** 用户打开包含九万条词条的有效词典
- **THEN** 应用在两秒内展示不超过 300 条首批结果，并允许继续加载或搜索全部词条
