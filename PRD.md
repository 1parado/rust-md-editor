# PRD — rust-md-editor

## Process (loop)

1. 对照本 PRD + CI 日志做差距分析
2. 更新「本轮目标」
3. 实现 → 推送 main → workflow_dispatch 发版
4. CI 失败则修到绿再发

**禁止**本地 `cargo build --release` 作为发版手段。

---

## 已完成

### v0.2.0
模块化 · 增量 block · syntect · 滚动同步 · 多平台 CI

### v0.3.0
UTF-8 光标 · 脏退出确认 · Home/End · 按词跳转 · 比例滚动同步 · 预览 debounce

---

## v0.4.0（本轮）

| ID | 目标 | 方案 |
|----|------|------|
| P0-1 | 查找 | Ctrl+F 输入查询，Enter/n 下一个，N 上一个，Esc 退出 |
| P0-2 | 水平滚动 | 长行时自动/方向键保持光标可见 |
| P1-1 | 状态栏显示查找模式 | `Find: query (k/n)` |
| P1-2 | 文档同步 | README / 帮助 |

### 非目标

- 替换（Ctrl+H replace）
- 正则查找
- 完整打开文件 UI

### 验收

- CI 五平台成功
- Ctrl+F 可定位匹配并 n/N 循环
- 长行光标不跑出可视区

---

## v0.5.0（候选）

- 查找并替换
- 行号跳转 Ctrl+G
- GFM 表格更整齐对齐
- 可选：裁掉无用 syntect 语法减小体积
