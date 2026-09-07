import { createI18n } from "vue-i18n";

export const i18n = createI18n({
  legacy: false,
  locale: "zh-CN",
  fallbackLocale: "en",
  messages: {
    "zh-CN": {
      app: { eyebrow: "RIME DICTIONARY WORKBENCH", title: "词库工坊", subtitle: "从词条到编码，每次改动都清晰、可控。" },
      nav: { chooseFile: "选择文件", dictionaryFilter: "Rime 词典" },
      status: { saved: "已保存", unsaved: "有未保存修改", loading: "正在读取词典…", empty: "没有匹配的词条", noFile: "选择一个 Rime 词典开始编辑", noFileShort: "尚未选择词典" },
      editor: { eyebrow: "词条", search: "搜索词组或编码", add: "新增词条", phrase: "词组", code: "五笔编码", weight: "权重", optional: "可选", edit: "编辑", cancel: "取消", confirmAdd: "加入词典", confirmEdit: "保存修改", entries: "条词条", loadMore: "已显示 {shown}/{total}，加载更多", save: "保存到词典", saving: "正在保存…", manualDeploy: "保存后请手动重新部署 Rime" },
      message: { loaded: "词典已加载", saved: "已创建滚动备份并安全保存", invalid: "请检查词组、编码和权重", duplicate: "相同词组和编码已存在", failed: "操作失败" },
      theme: { title: "外观", mode: "显示模式", color: "主题色", system: "跟随系统", light: "浅色", dark: "深色", blue: "蓝色", teal: "青绿色", orange: "橙色" },
    },
    en: {
      app: { eyebrow: "RIME DICTIONARY WORKBENCH", title: "Dict Studio", subtitle: "Precise, safe editing for every phrase and code." },
      nav: { chooseFile: "Choose file", dictionaryFilter: "Rime dictionary" },
      status: { saved: "Saved", unsaved: "Unsaved changes", loading: "Loading dictionary…", empty: "No matching entries", noFile: "Choose a Rime dictionary to begin", noFileShort: "No dictionary selected" },
      editor: { eyebrow: "ENTRY", search: "Search phrase or code", add: "New entry", phrase: "Phrase", code: "Wubi code", weight: "Weight", optional: "Optional", edit: "Edit", cancel: "Cancel", confirmAdd: "Add entry", confirmEdit: "Save changes", entries: "entries", loadMore: "Showing {shown}/{total} · Load more", save: "Save dictionary", saving: "Saving…", manualDeploy: "Deploy Rime manually after saving" },
      message: { loaded: "Dictionary loaded", saved: "Rolling backup created and dictionary saved", invalid: "Check phrase, code and weight", duplicate: "The same phrase and code already exists", failed: "Operation failed" },
      theme: { title: "Appearance", mode: "Mode", color: "Accent", system: "System", light: "Light", dark: "Dark", blue: "Blue", teal: "Teal", orange: "Orange" },
    },
  },
});
