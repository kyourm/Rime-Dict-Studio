# Fix large dictionary loading and sidebar actions

## Why

加载 `wubi86_jidian.dict.yaml` 等近九万行词典时，界面会因一次创建全部词条 DOM 而长时间无响应。选择目录后，词典列表还会把底部操作按钮挤出可视区。

## What Changes

- 限制单次渲染的词条数量，并允许按需继续加载。
- 词典总数、搜索范围和内存文档保持完整，不截断数据。
- 让侧栏词典列表独立滚动，目录和文件选择按钮固定可见。
- 增加九万条数据规模的 UI 性能回归测试。

## Capabilities

- `dictionary-editing`：改善大词典编辑性能和词典导航可用性。

### Clarifications

- 用户报告选择 `wubi86_jidian.dict.yaml` 后持续转圈，重启会因记忆该文件继续卡住。
- 用户报告选择目录后，“选择目录/选择文件”操作从界面消失。
- 验收标准据此确定为：九万条词典在两秒内进入可操作状态；首屏 DOM 不超过 300 条；两个选择操作始终存在。
