# Agent.md

## 目标

- 轻量桌面 Markdown 编辑器（egui）
- **正式二进制仅由 GitHub Actions 编译并发布**

## 循环

1. 读 PRD → 定本轮目标
2. 实现 → 推 main
3. 触发 release workflow
4. CI 失败则修到绿

## 禁止

- 本地 release 产物入库
- 提交密钥 / `.env`

## 本地允许

```bash
cargo check
cargo run -- file.md
```

## 架构

- `src/app.rs` — egui UI + 状态
- `src/preview.rs` — 增量 block + syntect
- `src/main.rs` — eframe 入口

旧 TUI 模块（buffer/draw/events 等）若仍在树中可删除，不再参与编译。
