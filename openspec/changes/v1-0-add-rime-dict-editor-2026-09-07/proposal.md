# Rime Dict Studio

## Why

直接编辑 Rime `.dict.yaml` 容易误用空格、产生重复词条或破坏头部配置，需要一个跨平台、可记忆路径且安全保存的桌面工具。

## What Changes

- 自动识别 Windows、macOS、Linux 常见 Rime 用户目录。
- 允许手动选择并记忆 Rime 目录与目标 `.dict.yaml` 文件。
- 搜索、新增、修改五笔词条的编码与权重。
- 保存前校验并创建一个滚动覆盖的备份文件。
- 明确不自动触发 Rime 重新部署。

## Capabilities

- `dictionary-editing`：发现、选择、记忆、编辑并安全保存 Rime 词典。

### Clarifications

- Q：平台范围？A：Windows、macOS、Linux 全平台。
- Q：技术形态？A：接受 Tauri + Vue 3 + TypeScript 桌面 GUI。
- Q：文件范围？A：由用户指定要编辑的 `.dict.yaml`，并记忆该选择。
- Q：编码生成？A：仅手动输入。
- Q：词条格式？A：`词组<Tab>编码<Tab>权重`，没有其他格式。
- Q：自动识别？A：识别各平台标准 Rime 目录。
- Q：备份策略？A：保存前备份，但只保留一份，新备份覆盖旧备份。
- Q：自动部署？A：不需要，用户手动部署。
- Q：样本？A：读取 `/Users/jiangliming/Library/Rime` 下现有词典确认格式。
- Q：验收流程？A：定位目录、选词典、搜索、新增或修改、校验、备份并保存。
- 用户于 2026-09-07 明确确认“无歧义，可以开始”。
