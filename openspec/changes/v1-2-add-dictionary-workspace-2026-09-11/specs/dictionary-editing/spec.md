## MODIFIED Requirements

### Requirement: Edit dictionary entries

应用 MUST 支持在当前词典及其有效导入词典组成的词库组中搜索和修改词条，并将新增词条写入当前选择文件。

#### Scenario: Search across imported dictionaries

- **WHEN** 用户打开包含有效 `import_tables` 的词典并搜索词组或编码
- **THEN** 应用返回整个词库组内的匹配词条并显示来源文件

#### Scenario: Show exact code collisions

- **WHEN** 用户在新增词条表单中输入编码
- **THEN** 应用实时展示词库组内具有完全相同编码的词条、权重和来源

#### Scenario: Edit an imported entry

- **WHEN** 用户修改导入词典中的词条权重
- **THEN** 应用在对应来源文件的内存文档中记录修改

## ADDED Requirements

### Requirement: Dictionary group loading

应用 MUST 递归加载当前词典头部中未注释的 `import_tables`，忽略缺失文件并避免循环导入。

#### Scenario: Open a dictionary group

- **WHEN** 用户选择一个含有递归导入关系的 `.dict.yaml` 文件
- **THEN** 应用仅加载每个可用词典一次，并显示词库组的文件数和总词条数

#### Scenario: Open an imported child dictionary

- **WHEN** 用户选择一个被同目录根词典直接或间接导入的子词典
- **THEN** 应用自动发现根词典并加载其完整词库组，但仍将所选子词典作为新增词条的写入目标

### Requirement: Multi-file safe save

应用 MUST 保存词库组中所有已修改文件，并对每个文件创建单份滚动备份后原子替换。

#### Scenario: Save changes in several dictionaries

- **WHEN** 用户修改多个来源文件并执行保存
- **THEN** 应用安全保存全部修改文件，且每个目标只保留 `<词典>.backup`
