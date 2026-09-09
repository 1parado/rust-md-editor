# rust-md-editor

轻量级 **桌面** Markdown 编辑器（Rust + egui）。

增量预览（streamdown 风格）· 代码块高亮 · 打开/保存 · 查找替换。

## 下载

从 [Releases](https://github.com/1parado/rust-md-editor/releases) 下载对应平台二进制（**GitHub CI 构建**，无需本地编译正式包）。

| 平台 | 产物 |
|------|------|
| Linux x86_64 | `rust-md-editor-linux-x86_64` |
| macOS Intel | `rust-md-editor-macos-x86_64` |
| macOS Apple Silicon | `rust-md-editor-macos-aarch64` |
| Windows x86_64 | `rust-md-editor-windows-x86_64.exe` |

## 功能

- 左右分栏：源码编辑 + 实时预览
- **增量 block 渲染**（只重算变化块）
- 代码 fence 高亮（syntect）
- 文件打开 / 保存（系统对话框）
- 查找 / 全部替换
- 未闭合 fence 显示 `streaming…`

## 快捷键

| 按键 | 功能 |
|------|------|
| `Cmd/Ctrl+S` | 保存 |
| `Cmd/Ctrl+O` | 打开 |
| `Cmd/Ctrl+F` | 查找/替换窗口 |

## 开发

```bash
cargo run
cargo run -- README.md
```

正式发版只走 GitHub Actions，见 [Agent.md](./Agent.md) / [PRD.md](./PRD.md)。

## License

MIT
