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
      editor: { eyebrow: "词条", search: "搜索整个词库组", add: "新增词条", phrase: "词组", code: "编码", weight: "权重", source: "来源", optional: "可选", edit: "编辑", decreaseWeight: "降低权重", increaseWeight: "提高权重", cancel: "取消", confirmAdd: "加入词典", confirmEdit: "保存修改", entries: "条词条", groupSummary: "{files} 个文件 · {entries} 条", collisions: "发现 {count} 个同码词条", moreCollisions: "另有 {count} 条未显示", loadMore: "已显示 {shown}/{total}，加载更多", save: "仅保存", saving: "正在保存…" },
      deploy: { squirrel: "鼠须管", weasel: "小狼毫", librime: "librime", custom: "自定义部署", unavailable: "未配置部署器", detecting: "正在识别部署器…", experimental: "实验性", deploy: "重新部署", deploying: "正在部署…", saveAndDeploy: "保存并重新部署", settings: "重新部署设置", help: "默认会自动识别当前平台的 Rime 部署器；识别失败时可在这里指定程序。", program: "部署程序", browse: "选择", chooseProgram: "选择 Rime 部署程序", arguments: "参数（每行一个）", argumentsHint: "例如：\n--reload", useAutomatic: "恢复自动识别", saveSettings: "保存设置", savingSettings: "正在保存…" },
      message: { loaded: "词库组已加载", loadedWithWarnings: "词库组已加载，{count} 个导入文件未找到", saved: "已为修改文件创建滚动备份并安全保存", deployed: "词典已保存并重新部署完成", deployFailed: "词典已保存，但重新部署失败", deploySettingsSaved: "重新部署设置已保存", invalid: "请检查词组、编码和权重", duplicate: "词库组中已存在相同词组和编码", failed: "操作失败" },
      theme: { title: "外观", mode: "显示模式", color: "主题色", system: "跟随系统", light: "浅色", dark: "深色", blue: "蓝色", teal: "青绿色", orange: "橙色" },
    },
    en: {
      app: { eyebrow: "RIME DICTIONARY WORKBENCH", title: "Dict Studio", subtitle: "Precise, safe editing for every phrase and code." },
      nav: { chooseFile: "Choose file", dictionaryFilter: "Rime dictionary" },
      status: { saved: "Saved", unsaved: "Unsaved changes", loading: "Loading dictionary…", empty: "No matching entries", noFile: "Choose a Rime dictionary to begin", noFileShort: "No dictionary selected" },
      editor: { eyebrow: "ENTRY", search: "Search dictionary group", add: "New entry", phrase: "Phrase", code: "Code", weight: "Weight", source: "Source", optional: "Optional", edit: "Edit", decreaseWeight: "Decrease weight", increaseWeight: "Increase weight", cancel: "Cancel", confirmAdd: "Add entry", confirmEdit: "Save changes", entries: "entries", groupSummary: "{files} files · {entries} entries", collisions: "{count} exact-code matches", moreCollisions: "{count} more not shown", loadMore: "Showing {shown}/{total} · Load more", save: "Save only", saving: "Saving…" },
      deploy: { squirrel: "Squirrel", weasel: "Weasel", librime: "librime", custom: "Custom deployer", unavailable: "Deployer not configured", detecting: "Detecting deployer…", experimental: "Experimental", deploy: "Deploy", deploying: "Deploying…", saveAndDeploy: "Save and deploy", settings: "Deployment settings", help: "The platform Rime deployer is detected automatically. Choose a program here when detection is unavailable.", program: "Deployer program", browse: "Browse", chooseProgram: "Choose Rime deployer", arguments: "Arguments (one per line)", argumentsHint: "For example:\n--reload", useAutomatic: "Use automatic detection", saveSettings: "Save settings", savingSettings: "Saving…" },
      message: { loaded: "Dictionary group loaded", loadedWithWarnings: "Dictionary group loaded; {count} imports were not found", saved: "Rolling backups created and modified dictionaries saved", deployed: "Dictionary saved and Rime deployed", deployFailed: "Dictionary was saved, but deployment failed", deploySettingsSaved: "Deployment settings saved", invalid: "Check phrase, code and weight", duplicate: "The same phrase and code exists in this group", failed: "Operation failed" },
      theme: { title: "Appearance", mode: "Mode", color: "Accent", system: "System", light: "Light", dark: "Dark", blue: "Blue", teal: "Teal", orange: "Orange" },
    },
  },
});
