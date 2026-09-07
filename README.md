# Rime Dict Studio

一个安全、跨平台的 Rime `.dict.yaml` 词典编辑器。

## 功能

- 手动选择并记忆最后使用的词典
- 搜索、新增、修改五笔编码与权重
- 保留词典头部、注释和分组
- 保存时只维护一份 `<词典>.backup` 滚动备份
- 跟随系统的明暗模式和可记忆主题色

## 开发

需要 Node.js 20+、Rust 和当前平台的 Tauri 系统依赖。

```bash
npm install
npm test
npm run tauri dev
```

保存词典后，请自行重新部署 Rime。
