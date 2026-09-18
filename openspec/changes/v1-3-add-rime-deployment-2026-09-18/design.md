# Design

## Deep module

Rust 端 `RimeDeployer` 以 `deployment_state` 和 `deploy_rime` 两个命令形成小接口，隐藏平台识别、路径搜索、参数、工作目录、超时和退出码。前端不包含平台分支。

## Adapters

- Windows Adapter：扫描标准 Program Files 下的 Rime/weasel 版本目录，执行 `/deploy`。
- macOS Adapter：扫描系统与用户 Input Methods，执行 `Squirrel --reload`。
- Linux Adapter：从 PATH 查找 `rime_deployer`，按词典目录和共享数据目录生成 `--build` 参数，标记实验性。
- Custom Adapter：执行用户显式选择的程序与参数，不启动 Shell。

## Failure semantics

保存和部署是顺序发生的两个结果。只有保存成功才进入部署；部署失败不回滚词典。子进程设定超时并在超时后终止。

