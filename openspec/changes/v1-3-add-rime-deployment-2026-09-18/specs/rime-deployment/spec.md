## ADDED Requirements

### Requirement: Detect platform deployer

应用 MUST 自动识别当前平台可用的官方 Rime 部署入口，并将 Linux 自动识别标记为实验性。

#### Scenario: Detect Squirrel on macOS

- **WHEN** 鼠须管安装于系统或用户 Input Methods 目录
- **THEN** 应用识别 `Squirrel --reload` 为重新部署命令

#### Scenario: Detect Weasel on Windows

- **WHEN** 小狼毫部署器存在于标准安装目录
- **THEN** 应用识别 `WeaselDeployer.exe /deploy` 并使用部署器目录作为工作目录

#### Scenario: Detect librime deployer on Linux

- **WHEN** `rime_deployer` 存在于 PATH 且用户词典目录已知
- **THEN** 应用生成 `--build` 命令并将能力标记为实验性

### Requirement: Configure custom deployer

应用 MUST 允许用户设置可执行文件、参数列表，并记忆或恢复自动识别。

#### Scenario: Remember a custom deployer

- **WHEN** 用户选择部署程序并保存参数
- **THEN** 应用下次启动继续使用该配置，且不通过 Shell 执行

### Requirement: Save and deploy

应用 MUST 在词典保存成功后执行部署，展示独立的部署中状态和结果。

#### Scenario: Save and deploy successfully

- **WHEN** 用户存在未保存修改并点击“保存并重新部署”且部署器成功退出
- **THEN** 应用先安全保存全部修改，再提示保存和部署均成功

#### Scenario: Deployment fails after save

- **WHEN** 词典保存成功但部署器失败、超时或不可用
- **THEN** 应用保留已保存文件并明确提示部署失败，不执行回滚

