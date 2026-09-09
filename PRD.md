# PRD — rust-md-editor

## Product

轻量级终端 Markdown 编辑器。目标：体积小、启动快、增量预览（streamdown 风格）、只在 GitHub CI 发版。

## Process (loop)

每个版本发布后：

1. 对照本 PRD + CI 日志做差距分析
2. 更新本 PRD 的「本轮目标」
3. 实现 → 推送 main → workflow_dispatch 发版
4. 若 CI 失败，修编译错误后重发，不阻塞下一轮

**禁止**：本地 `cargo build --release` 作为发版手段。

---

## v0.2.0（已完成）

- 模块化：`buffer` / `highlight` / `preview` / `app` / `draw` / `events`
- 增量 block 哈希缓存 + incomplete fence `streaming…`
- syntect 源码轻量着色 + code fence 高亮
- 滚动同步、鼠标点击/滚轮
- 多平台 CI Release

---

## v0.3.0（本轮）

### P0 — 正确性

| ID | 问题 | 方案 |
|----|------|------|
| P0-1 | 光标按字节 `split_at`，中文等易 panic | 按 char boundary 安全切分 |
| P0-2 | 脏文件直接退出无提示 | Ctrl+Q 二次确认 |
| P0-3 | 预览在 draw 里同步刷新，卡输入 | 仅事件循环 debounce 刷新 |

### P1 — 编辑体验

| ID | 问题 | 方案 |
|----|------|------|
| P1-1 | 无 Home / End | 行首行尾 |
| P1-2 | 无 Ctrl+Left/Right | 按词跳转 |
| P1-3 | 滚动同步用相同行 delta | 按比例映射 |
| P1-4 | 帮助文案过简 | 补全快捷键 |

### P2 — 仓库卫生

| ID | 方案 |
|----|------|
| P2-1 | 删除 `src/main.rs.gz.b64`、`src/sections/` 等遗留 |
| P2-2 | version → 0.3.0，README 同步 |

### 非目标（本轮不做）

- 插件系统、LSP、多标签、图形界面
- 完整 WYSIWYG 表格编辑
- 本地 release 构建

### 验收

- `cargo build --release` 在 CI 五平台成功
- 中文输入不 panic
- 未保存退出需确认
- Release 产物可下载

---

## v0.4.0（候选）

- 水平滚动 / 长行
- 查找（Ctrl+F）
- 打开文件对话框或路径输入
- 更完整 GFM（表格对齐显示）
- 二进制体积进一步裁剪（可选 features）

---

## Metrics

- 启动：打开后首次预览 < 200ms（小文件）
- 增量：编辑单 block 时 reuse 率尽量高
- 体积：release strip + LTO + opt-level=z
