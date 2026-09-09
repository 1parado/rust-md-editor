# Agent.md — rust-md-editor

本仓库给 Agent / 自动化使用的约束与发布规范。

## 目标

- **轻量化**：依赖少、二进制小（release：`opt-level = "z"` + LTO + strip）
- **启动快**：纯终端 TUI，无 Electron / WebView
- **不在本地编译发布产物**：开发可本地 `cargo check` / `cargo run`，**正式二进制只在 GitHub Actions 编译并发布**

## 持续改进循环（必须遵守）

1. 读 [PRD.md](./PRD.md)，对照当前版本找差距
2. 更新 PRD「本轮目标」
3. 实现代码 → 推送 `main`
4. 触发 `.github/workflows/release.yml`（`workflow_dispatch` 或 `v*` tag）
5. 若 CI 失败：根据日志修编译/测试错误，再发版，不半途停
6. 发版成功后进入下一版本候选（写回 PRD）

## 禁止

- 不要在本地执行 `cargo build --release` 后把产物提交进仓库
- 不要提交 `.env`、密钥、token、私钥等敏感文件（见 `.gitignore`）
- 不要把本机路径、个人 token 写进 workflow 或文档

## 允许的本地操作

```bash
cargo check
cargo run -- path/to/file.md
cargo test
```

需要可执行文件：从 [Releases](https://github.com/1parado/rust-md-editor/releases) 下载。

## CI 与 Release

- Workflow：`.github/workflows/release.yml`
- 触发：`git tag vX.Y.Z && git push origin vX.Y.Z`，或 Actions 手动填写版本
- 五平台 `cargo build --release` → GitHub Release 上传资产

## 架构原则

1. 保持增量 block 预览（streamdown 思路）
2. 新增依赖评估体积；优先 `default-features = false`
3. 敏感配置只走环境变量 / GitHub Secrets
4. 用户默认从 Release 获取二进制

## 仓库

https://github.com/1parado/rust-md-editor
