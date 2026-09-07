## ADDED Requirements

### Requirement: Dictionary file selection

应用 MUST 仅展示当前词典和选择文件操作，不列出同目录中的其他词典。

#### Scenario: Choose a dictionary

- **WHEN** 用户点击选择文件并选择有效 `.dict.yaml`
- **THEN** 应用打开该词典并将其记为下次启动的默认文件

#### Scenario: Remembered file is unavailable

- **WHEN** 应用启动且上次选择的文件不存在
- **THEN** 应用显示选择词典文件的空状态

### Requirement: Appearance preferences

应用 MUST 提供跟随系统、浅色和深色显示模式以及有限主题色，并持久化用户选择。

#### Scenario: First launch appearance

- **WHEN** 用户尚未设置外观偏好
- **THEN** 应用默认跟随操作系统明暗模式并使用系统主题色

#### Scenario: Restore appearance

- **WHEN** 用户选择显示模式或主题色后重新启动应用
- **THEN** 应用恢复之前的外观选择

### Requirement: Compact solid-color interface

应用 MUST 使用紧凑、无渐变的纯色界面，并提供跨平台应用图标。

#### Scenario: Open the desktop tool

- **WHEN** 用户打开应用
- **THEN** 应用以约 720×560 的紧凑窗口显示当前文件、搜索、新增、词条表格和保存操作

## REMOVED Requirements

### Requirement: Rime directory discovery

**Reason**: 单文件编辑工作流不需要展示整个目录，文件列表增加了视觉负担和误操作面。

**Migration**: 用户通过“选择文件”直接定位词典；应用继续记忆最后选择的有效文件。
