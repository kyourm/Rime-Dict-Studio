# Rime Dict Studio

一个安全、跨平台的 Rime `.dict.yaml` 词典编辑器。

## 功能

- 自动识别 macOS、Windows、Linux 常见 Rime 用户目录
- 手动选择并记忆词典
- 搜索、新增、修改五笔编码与权重
- 保留词典头部、注释和分组
- 保存时只维护一份 `<词典>.backup` 滚动备份

## 开发

需要 Node.js 20+、Rust 和当前平台的 Tauri 系统依赖。

```bash
npm install
npm test
npm run tauri dev
```

保存词典后，请自行重新部署 Rime。
