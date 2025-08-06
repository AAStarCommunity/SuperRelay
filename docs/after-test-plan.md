# SuperRelay Test Failure Analysis and TODO Plan

This document outlines the identified issues from the test script executions and provides a prioritized plan to address them.

## 1. 核心构建问题 (Core Build Failure)

**问题 (Issue):**
- The `super-relay` binary fails to build. This is the root cause for a majority of the test failures.
- `test_rundler.sh` indicates that unit tests are failing, which prevents the release build from completing.

**待办 (TODO):**
- **1.1.** 深入分析 `cargo test --workspace` 的失败原因，并修复核心单元测试。 (Deep dive into `cargo test --workspace` failures and fix the core unit tests.)
- **1.2.** 成功执行 `cargo build --release --workspace` 以确保所有二进制文件都能正确生成。 (Successfully execute `cargo build --release --workspace` to ensure all binaries are generated correctly.)

## 2. 依赖问题 (Dependency Issues)

**问题 (Issue):**
- `cargo audit` reported a vulnerability in the `ring` crate and several unmaintained dependencies.
- The headless demo test (`test_demo_headless.sh`) fails due to the missing `@playwright/test` npm module.
- The v0.7 spec test (`test_spec_v07.sh`) requires Docker, which is not installed.

**待办 (TODO):**
- **2.1.** 运行 `cargo update` 并审查 `ring` crate 的版本，解决安全漏洞。 (Run `cargo update` and review the version of the `ring` crate to fix the security vulnerability.)
- **2.2.** 评估并替换或更新未维护的依赖项 (`derivative`, `instant`, `paste`, `proc-macro-error`)。 (Evaluate and replace or update unmaintained dependencies.)
- **2.3.** 在 `demo` 目录下运行 `pnpm install` (根据用户偏好) 来修复 Playwright 依赖问题。 (Run `pnpm install` in the `demo` directory to fix the Playwright dependency issue.)
- **2.4.** 在项目文档中明确 Docker 是 v0.7 测试的必要依赖。 (Document that Docker is a prerequisite for v0.7 tests in the project's documentation.)

## 3. 测试环境和脚本问题 (Test Environment & Scripting Issues)

**问题 (Issue):**
- Test scripts are inconsistent in managing the Anvil lifecycle. Some start it, others expect it to be running.
- Scripts like `test_full_pipeline.sh` show errors from the `cast` command, indicating potential issues with how Foundry tools are invoked.
- Health checks consistently fail because the service cannot start without a valid binary.

**待办 (TODO):**
- **3.1.** 统一所有测试脚本中的 Anvil 启动和停止逻辑，确保测试环境的一致性。 (Standardize the Anvil start/stop logic across all test scripts for a consistent test environment.)
- **3.2.** 检查并修复 `test_full_pipeline.sh` 和其他脚本中调用 `cast` 的地方，确保命令格式正确。 (Review and fix the `cast` command invocations in `test_full_pipeline.sh` and other scripts.)
- **3.3.** 一旦核心构建问题解决，重新运行所有测试，以验证服务健康检查现在是否通过。 (Once the core build issue is resolved, re-run all tests to verify that service health checks now pass.)

## 4. 服务功能问题 (Service Functionality Issues)

**问题 (Issue):**
- `test_basic_gateway.sh` 中的健康检查失败，即使在脚本尝试启动服务后也是如此。这表明除了构建失败之外，可能还存在启动或配置问题。

**待办 (TODO):**
- **4.1.** 在成功构建二进制文件后，手动运行 `super-relay gateway` 命令，并使用 `curl` 或其他工具直接测试 `/health` 端点，以隔离问题。 (After a successful build, manually run the `super-relay gateway` command and test the `/health` endpoint directly with `curl` or another tool to isolate the issue.)
