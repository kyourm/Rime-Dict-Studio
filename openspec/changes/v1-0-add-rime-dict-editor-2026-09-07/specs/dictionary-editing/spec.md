## Purpose

为跨平台 Rime 用户提供可发现、可记忆且安全的词典编辑能力，避免手工维护 `.dict.yaml` 时破坏格式或丢失既有配置。

## ADDED Requirements

### Requirement: Rime directory discovery

应用 MUST 自动识别当前平台常见的 Rime 用户目录，并允许用户改选目录。

#### Scenario: Open a detected directory

- **WHEN** 应用启动且发现可用的标准 Rime 目录
- **THEN** 应用展示该目录中的 `.dict.yaml` 文件

### Requirement: Remember dictionary selection

应用 MUST 记住用户最后选择的 Rime 目录和词典文件。

#### Scenario: Restore the previous dictionary

- **WHEN** 用户重新启动应用
- **THEN** 应用自动打开上次成功选择的词典（若文件仍存在）

### Requirement: Edit dictionary entries

应用 MUST 支持搜索、新增和修改由词组、编码、可选权重构成的词条。

#### Scenario: Add a valid phrase

- **WHEN** 用户输入非空词组、合法五笔编码及合法权重并确认
- **THEN** 应用将词条加入内存模型，并可通过搜索找到

#### Scenario: Reject an exact duplicate

- **WHEN** 用户新增与现有词组和编码都相同的词条
- **THEN** 应用阻止操作并说明重复原因

### Requirement: Safe save

应用 MUST 保留 YAML 头部和注释，并在原子写入前创建单份滚动备份。

#### Scenario: Save a modified dictionary

- **WHEN** 用户保存通过校验的修改
- **THEN** 应用覆盖 `<词典>.backup` 后原子替换原词典，并提示用户手动部署 Rime
