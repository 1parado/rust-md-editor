# Agent.md — rust-md-editor

本仓库给 Agent / 自动化使用的约束与发布规范。

## 目标

- **轻量化**：依赖少、二进制小（release：`opt-level = "z"` + LTO + strip）
- **启动快**：纯终端 TUI，无 Electron / WebView
- **不在本地编译发布产物**：开发可本地 `cargo check` / `cargo run`，**正式二进制只在 GitHub Actions 编译并发布**

## 禁止

- 不要在本地执行 `cargo build --release` 后把 `target/release/` 或自打包的安装包提交进仓库
- 不要提交 `.env`、密钥、token、私钥、`credentials.json` 等敏感文件（见 `.gitignore`）
- 不要把本机路径、个人 token 写进 workflow 或文档

## 允许的本地操作

```bash
cargo check
cargo run -- path/to/file.md
cargo test   # 若有测试
```

需要可执行文件时：从 [Releases](https://github.com/1parado/rust-md-editor/releases) 下载 CI 产物，不要本地打正式包。

## CI 与 Release

- Workflow：`.github/workflows/release.yml`
- **触发方式**：
  1. Push 符合 `v*` 的 tag（推荐）：`git tag v0.2.0 && git push origin v0.2.0`
  2. 或在 Actions 里手动 `workflow_dispatch` 并填写版本号
- CI 在 `ubuntu-latest` / `macos-latest` / `windows-latest` 上 `cargo build --release`
- 成功后自动创建 **GitHub Release**，并上传对应平台二进制

## 版本与 Tag

- 版本与 `Cargo.toml` 的 `version` 保持一致
- Tag 格式：`v0.2.0`（带 `v` 前缀）
- 发版前更新 `Cargo.toml` version 与 `CHANGELOG`（如有）再打 tag

## 架构原则（给后续改动的 Agent）

1. 保持增量 block 预览（streamdown 思路），避免全量重渲染成为默认路径
2. 新增依赖需评估体积与启动成本；优先 `default-features = false`
3. 敏感配置只走环境变量 / GitHub Secrets，不入库
4. 文档与脚本默认假设用户从 Release 获取二进制，而不是本地编译

## 仓库

https://github.com/1parado/rust-md-editor
