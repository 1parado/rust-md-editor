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

## v1.1.0（候选）

- 预览内更好的表格与列表样式
- 查找高亮并跳转光标（TextEditState）
- 拖放打开文件
- 可配置主题（亮/暗）
- 撤销栈

## 约束

- 不在本地 `cargo build --release` 作为发版手段
- 新增依赖评估体积与启动成本
