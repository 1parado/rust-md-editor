# PRD — rust-md-editor

## 产品形态（v1.0 起）

**桌面 GUI 应用**（egui / eframe），不再是终端 TUI。

目标：轻量、启动快、增量 Markdown 预览、CI 多平台发版。

## 已完成

- v0.2–v0.5：终端版（ratatui）迭代
- **v1.0.0**：迁移为 egui 桌面应用
  - 分栏编辑 + 预览
  - 增量 block 缓存
  - 打开/保存（rfd）
  - 查找 / 全部替换
  - Linux/macOS/Windows CI

## v1.1.0（本版：UI / 交互优化）

目标：把"能用"的界面升级为"好用"的界面——主题、布局、预览质感、查找体验全面打磨。

### 已实现

- **亮 / 暗主题切换**：调色板抽象为 `Palette`（LIGHT / DARK），工具栏 🌙/☀️ 按钮与"视图"菜单均可切换；syntect 代码高亮主题随之联动（light → base16-ocean.light，dark → base16-ocean.dark）。
- **可拖拽分栏**：编辑/预览卡片之间新增分隔条，支持拖拽调节比例（0.28–0.72），悬停高亮 + `ResizeHorizontal` 光标；"视图"菜单滑杆保留。
- **预览质感重做**：
  - 代码块卡片化：圆角深色卡片 + 语言标签，真正彩色语法高亮（`CodeSpan`，之前只保留纯文本没上色）；
  - 引用块：强调色竖条 + 斜体；分隔线渲染为真实分隔符；行内 `code` 单独着色。
- **查找 / 替换升级**：匹配总数展示、上一处 / 下一处循环定位、**跳转编辑器光标**（TextEditState + CCursorRange）、Enter 查找、Esc 关闭窗口。
- **交互补充**：工具栏快捷按钮（新建 / 打开 / 保存 / 查找）、Ctrl+N 新建（脏文档二次确认）、拖放 .md 文件打开（脏文档二次确认）、状态栏显示光标 Ln/Col。

### 验收标准

- 明暗主题下预览、代码块、选区、按钮均无对比度问题
- 拖拽分栏平滑无跳动，比例在 0.28–0.72 之间
- 查找可循环定位且编辑器滚动跟随光标

## v1.2（候选 Backlog）

- 预览行内粗体 / 斜体 / 删除线样式（需把段落缓冲改为 span 结构）
- 编辑器与预览滚动同步（需要 block → 行号映射）
- 主题与分栏比例持久化（eframe storage）
- 预览字体缩放
- 清理死代码：旧 TUI 遗留 `src/draw.rs`、`src/buffer.rs`、`src/events.rs`、`src/highlight.rs`、`src/sections/`、`src/main.rs.gz.b64`（未被 `main.rs` 引用）

## 约束

- 不在本地 `cargo build --release` 作为发版手段
- 新增依赖评估体积与启动成本
