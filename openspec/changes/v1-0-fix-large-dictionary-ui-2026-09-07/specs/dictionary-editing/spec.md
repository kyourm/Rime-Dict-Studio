## MODIFIED Requirements

### Requirement: Edit dictionary entries

应用 MUST 支持搜索、新增和修改由词组、编码、可选权重构成的词条，并在大型词典中保持界面可操作。

#### Scenario: Load a large dictionary

- **WHEN** 用户打开包含九万条词条的有效词典
- **THEN** 应用在两秒内展示不超过 300 条首批结果，并允许继续加载或搜索全部词条

#### Scenario: Add a valid phrase

- **WHEN** 用户输入非空词组、合法五笔编码及合法权重并确认
- **THEN** 应用将词条加入内存模型，并可通过搜索找到

#### Scenario: Reject an exact duplicate

- **WHEN** 用户新增与现有词组和编码都相同的词条
- **THEN** 应用阻止操作并说明重复原因

### Requirement: Rime directory discovery

应用 MUST 自动识别当前平台常见的 Rime 用户目录，允许用户改选目录，并始终提供目录和文件选择操作。

#### Scenario: Keep selection actions visible

- **WHEN** 目录内包含超过侧栏可视高度的词典文件
- **THEN** 词典列表独立滚动，选择目录和选择文件按钮仍然可见

#### Scenario: Open a detected directory

- **WHEN** 应用启动且发现可用的标准 Rime 目录
- **THEN** 应用展示该目录中的 `.dict.yaml` 文件
