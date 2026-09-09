# rust-md-editor

轻量级纯 Rust 终端 Markdown 编辑器（增量预览 · 语法高亮 · 滚动同步）。

灵感来自 [streamdown](https://github.com/vercel/streamdown) / Lobe UI 的流式渲染思路。

## 下载（推荐）

**不要在本地编译正式包。** 请从 GitHub Releases 下载 CI 构建的二进制：

→ [Releases](https://github.com/1parado/rust-md-editor/releases)

| 平台 | 产物名 |
|------|--------|
| Linux x86_64 | `rust-md-editor-linux-x86_64` |
| Linux aarch64 | `rust-md-editor-linux-aarch64` |
| macOS Intel | `rust-md-editor-macos-x86_64` |
| macOS Apple Silicon | `rust-md-editor-macos-aarch64` |
| Windows x86_64 | `rust-md-editor-windows-x86_64.exe` |

## 特性

- **增量解析**：按 block 哈希缓存，只重渲染变化块；未闭合 fence 显示 `streaming…`
- **语法高亮**：syntect（源码轻量着色 + 预览 code fence 完整高亮）
- **滚动同步 + 鼠标**：Tab 切换焦点，滚轮同步，点击定位光标

## 本地开发（仅 check / run）

```bash
cargo check
cargo run -- README.md
```

正式二进制由 **GitHub Actions** 在 tag `v*` 或手动 workflow 时编译并发布。规范见 [Agent.md](./Agent.md)。

## 快捷键

| 按键 | 功能 |
|------|------|
| `Ctrl+S` | 保存 |
| `Ctrl+Q` / `Ctrl+C` | 退出 |
| `Ctrl+H` / `?` | 帮助 |
| `Tab` | 切换焦点 |
| `Shift+S`（预览焦点） | 开关滚动同步 |
| 方向键 / 鼠标 | 移动与滚动 |

## 发版

```bash
# 改 Cargo.toml version 后
git tag v0.2.0
git push origin v0.2.0
# CI 自动 multi-platform build → GitHub Release
```

或在 Actions 中 `workflow_dispatch` 填写版本号。

## License

MIT
