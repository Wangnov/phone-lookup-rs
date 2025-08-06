# Claude Code SubAgents Collection

本目录包含为 Phone Lookup RS 项目开发的专业化 SubAgent 集合。这些 SubAgent 基于实际项目开发经验总结而成，旨在提高开发效率和代码质量。

## 📋 SubAgent 列表

### 1. 🔄 Git Commit Agent
**文件**: `git-commit-agent.md`  
**用途**: 专业的 Git 提交管理  
**触发场景**: 所有 git commit 操作、仓库状态分析  
**核心功能**:
- 规范化提交信息生成
- 智能变更分析和分类
- 分批提交策略制定
- 提交质量检查

### 2. 🦀 Rust Development Agent  
**文件**: `rust-dev-agent.md`  
**用途**: Rust 代码开发专家  
**触发场景**: Rust 代码编写、架构设计、性能优化  
**核心功能**:
- 高质量 Rust 代码开发
- 异步编程和并发安全
- 性能优化策略
- 生态系统集成

### 3. 🧪 Rust Test Agent
**文件**: `rust-test-agent.md`  
**用途**: Rust 测试专家  
**触发场景**: 所有 Rust 测试活动  
**核心功能**:
- 测试策略设计
- 单元测试和集成测试
- 性能测试和并发测试
- 测试工具链配置

### 4. 🌐 Full-Stack Development Agent
**文件**: `fullstack-dev-agent.md`  
**用途**: 全栈应用开发专家  
**触发场景**: 多层应用开发、前后端集成  
**核心功能**:
- 三端统一架构设计
- React + Rust + Tauri 集成
- 跨技术栈开发协调
- 部署和监控策略

### 5. 📝 Documentation Agent
**文件**: `documentation-agent.md`  
**用途**: 技术文档专家  
**触发场景**: 文档创建和维护  
**核心功能**:
- 项目文档结构设计
- API 文档和用户指南
- 多语言技术写作
- 文档质量保证

## 🚀 使用指南

### 自动调用
Claude Code 会根据任务内容自动选择合适的 SubAgent：

```bash
# 这些命令会自动触发相应的 SubAgent
git status              # → git-commit-agent
cargo test             # → rust-test-agent  
cargo run              # → rust-dev-agent
```

### 手动调用
您也可以明确指定使用特定的 SubAgent：

```bash
# 使用 Git 提交专家分析变更
> Use the git-commit-agent to analyze current changes and create commits

# 使用 Rust 开发专家优化代码
> Use the rust-dev-agent to optimize this function for better performance

# 使用测试专家添加测试覆盖
> Use the rust-test-agent to create comprehensive tests for the query module

# 使用全栈专家设计前后端集成
> Use the fullstack-dev-agent to design the API integration between React and Rust

# 使用文档专家完善 README
> Use the documentation-agent to improve the project README
```

## 🎯 设计原则

### 1. 专业化分工
每个 SubAgent 专注于特定领域，避免职责重叠：
- **Git Commit Agent**: 专注版本控制
- **Rust Dev Agent**: 专注后端开发
- **Test Agent**: 专注质量保证
- **Full-Stack Agent**: 专注架构整合
- **Documentation Agent**: 专注技术写作

### 2. 工具精准配置
每个 SubAgent 只配置必要的工具，提高执行效率：
- 读写工具：`Read`, `Edit`, `Write`
- 搜索工具：`Grep`, `Glob`
- 执行工具：`Bash`
- 研究工具：`WebSearch`, `Task`

### 3. 实战经验驱动
基于 Phone Lookup RS 项目的实际开发经验：
- 三端部署架构经验
- Rust 异步编程最佳实践
- 前后端集成方案
- 测试策略和质量保证
- 技术文档编写规范

## 📊 SubAgent 使用统计

基于项目开发经验，各 SubAgent 的预期使用频率：

| SubAgent | 使用频率 | 主要场景 |
|----------|----------|----------|
| Git Commit Agent | ⭐⭐⭐⭐⭐ | 每次代码提交 |
| Rust Dev Agent | ⭐⭐⭐⭐ | 核心功能开发 |
| Test Agent | ⭐⭐⭐ | 测试编写和维护 |
| Full-Stack Agent | ⭐⭐⭐ | 架构设计和集成 |
| Documentation Agent | ⭐⭐ | 文档更新和维护 |

## 🔧 最佳实践

### 1. 任务匹配原则
- **明确任务类型**：根据任务性质选择合适的 SubAgent
- **避免跨领域**：不要让单个 SubAgent 处理多个领域的任务
- **善用组合**：复杂任务可以使用多个 SubAgent 协作完成

### 2. 上下文管理
- **保持专业性**：每个 SubAgent 维护其专业领域的上下文
- **信息传递**：SubAgent 之间通过明确的接口传递信息
- **状态隔离**：避免 SubAgent 之间的状态污染

### 3. 质量保证
- **定期更新**：根据项目发展更新 SubAgent 的知识库
- **经验积累**：将新的最佳实践及时集成到 SubAgent 中
- **反馈循环**：收集使用反馈，持续改进 SubAgent 效果

## 🚧 扩展计划

根据项目发展需要，考虑添加以下专业化 SubAgent：

### 短期规划
- [ ] **Performance Analysis Agent**: 性能分析和优化专家
- [ ] **Security Audit Agent**: 安全审计和漏洞检查专家  
- [ ] **CI/CD Agent**: 持续集成和部署专家
- [ ] **Database Agent**: 数据库设计和优化专家

### 长期规划
- [ ] **Architecture Review Agent**: 架构审查专家
- [ ] **Code Review Agent**: 代码审查专家
- [ ] **Monitoring Agent**: 监控和运维专家
- [ ] **Translation Agent**: 国际化和本地化专家

## 📚 学习资源

### Claude Code SubAgent 官方文档
- [SubAgent 概述](https://docs.anthropic.com/en/docs/claude-code/sub-agents)
- [创建和管理 SubAgent](https://docs.anthropic.com/en/docs/claude-code/sub-agents#managing-subagents)
- [最佳实践指南](https://docs.anthropic.com/en/docs/claude-code/common-workflows)

### 相关技术文档
- [Rust 开发指南](https://doc.rust-lang.org/book/)
- [React 开发文档](https://react.dev/)
- [Tauri 应用开发](https://tauri.app/)
- [Git 最佳实践](https://git-scm.com/docs)

## 🤝 贡献指南

如果您想改进这些 SubAgent 或添加新的 SubAgent：

1. **Fork 项目**
2. **创建特性分支**: `git checkout -b feature/new-subagent`
3. **编写 SubAgent**: 参考现有模板和最佳实践
4. **测试验证**: 确保 SubAgent 正常工作
5. **提交更改**: `git commit -m 'feat: 添加新的SubAgent'`
6. **创建 PR**: 提交 Pull Request 并描述改进内容

## 📄 许可证

这些 SubAgent 采用与主项目相同的 MIT 许可证。

---

**致谢**: 这些 SubAgent 的设计和实现基于 Phone Lookup RS 项目的实际开发经验，感谢所有参与项目开发的贡献者。