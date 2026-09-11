# Design

## Deep module

`DictionaryWorkspace` 封装多文档、跨文件索引、撞码查询和脏文件收集。界面仅面向词条引用，不感知导入递归、文档定位或序列化细节。

## Backend boundary

Rust 命令 `read_dictionary_group` 负责路径解析、递归读取和循环去重，返回根文件与各词典内容。`save_dictionaries` 对每个修改文件复用现有 `safe_save`。

## Performance

词典组只在打开时解析一次；搜索遍历轻量词条引用，DOM 首批限制 100 条。精确撞码通过 `code -> entry refs` 索引常数时间读取。

## Failure handling

缺失的导入文件不阻止打开根词典，但返回警告列表；根词典无法读取或格式不合法时整体失败。保存任一文件失败时返回错误并保留未保存状态。

