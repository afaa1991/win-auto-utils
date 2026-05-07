# 文档索引

欢迎使用 win-auto-utils 文档！本索引帮助你找到所需的文档。

## 🚀 快速开始

**不知道从哪里开始？** 查看我们的 [快速开始指南](QUICK_START.md) - 它会根据你的目标帮你找到所需内容！

## 📚 文档结构

### 主文档
- [English README](../../README.md) - 项目概览和快速开始
- [中文 README](../zh/README.md) - 项目概览和快速开始

### 模块文档

#### 英文 (English)
- [Modules Overview](modules/overview.md) - 所有模块的高层视图
- [Memory Operations](modules/memory.md) - 基础读写操作
- [Memory Hooking](modules/memory_hook.md) - 函数拦截和寄存器提取
- [Address Resolution](modules/memory_resolver.md) - 符号地址解析
- [AOB Scanning](modules/memory_aobscan.md) - SIMD 模式扫描
- [Script Engine](modules/script_engine.md) - 自动化脚本解释器
- [Input Control](modules/input.md) - 键盘和鼠标模拟
- [Process & Window](modules/process_window.md) - 进程和窗口管理
- [Screen Capture](modules/dxgi.md) - 基于 DXGI 的捕获
- [Template Matching](modules/template_matcher.md) - 基于图像的 UI 检测
- [DLL Injection](modules/dll_injector.md) - 跨架构注入

#### 中文
- [模块概览](modules/overview.md) - 所有模块的高层视图
- [内存操作](modules/memory.md) - 基础读写操作
- [内存钩子](modules/memory_hook.md) - 函数拦截和寄存器提取
- [内存锁定](modules/memory_lock.md) - 持续监控和恢复内存值
- [地址解析](modules/memory_resolver.md) - 符号地址解析
- [字节扫描](modules/memory_aobscan.md) - SIMD 模式扫描
- [脚本引擎](modules/script_engine.md) - 自动化脚本解释器
- [输入控制](modules/input.md) - 键盘和鼠标模拟
- [进程与窗口](modules/process_window.md) - 进程和窗口管理
- [屏幕捕获](modules/dxgi.md) - 基于 DXGI 的捕获
- [模板匹配](modules/template_matcher.md) - 基于图像的 UI 检测
- [DLL注入](modules/dll_injector.md) - 跨架构注入

## 🎯 按用例快速导航

### 应用程序自动化
1. [进程管理](modules/process_window.md) - 查找目标进程
2. [内存管理器](modules/memory_manager.md) - 统一管理多个功能（推荐）
3. [内存操作](modules/memory.md) - 读写进程状态
4. [内存钩子](modules/memory_hook.md) - 拦截函数
5. [字节扫描](modules/memory_aobscan.md) - 查找动态地址
6. [输入控制](modules/input.md) - 模拟用户操作
7. [屏幕捕获](modules/dxgi.md) - 监控屏幕画面

### UI 测试
1. [窗口管理](modules/process_window.md) - 控制应用程序窗口
2. [输入控制](modules/input.md) - 与 UI 元素交互
3. [模板匹配](modules/template_matcher.md) - 验证 UI 外观
4. [脚本引擎](modules/script_engine.md) - 编写测试脚本

### 逆向工程
1. [内存操作](modules/memory.md) - 检查进程内存
2. [字节扫描](modules/memory_aobscan.md) - 定位代码模式
3. [地址解析](modules/memory_resolver.md) - 解析指针
4. [内存管理器](modules/memory_manager.md) - 统一管理钩子和锁定（推荐）
5. [内存钩子](modules/memory_hook.md) - 监控函数调用
6. [寄存器提取](modules/memory_hook.md) - 捕获 CPU 状态
7. [DLL注入](modules/dll_injector.md) - 注入调试工具

### 桌面自动化
1. [脚本引擎](modules/script_engine.md) - 创建自动化脚本
2. [输入控制](modules/input.md) - 控制键鼠
3. [窗口管理](modules/process_window.md) - 管理应用程序
4. [屏幕捕获](modules/dxgi.md) - 视觉验证

## 📖 学习路径

### 初学者
1. 从 [README](../../README.md) 开始了解项目概览
2. 阅读 [模块概览](modules/overview.md) 理解架构
3. 尝试简单示例：
   - [memory_operations.rs](../../examples/memory_operations.rs)
   - [mouse_control.rs](../../examples/mouse_control.rs)
4. 根据需要探索各个模块文档

### 中级用户
1. 学习高级模块：
   - [内存钩子](modules/memory_hook.md)
   - [字节扫描](modules/memory_aobscan.md)
2. 查看基准测试示例：
   - [aobscan_benchmark.rs](../../examples/aobscan_benchmark.rs)
   - [template_matching_benchmark.rs](../../examples/template_matching_benchmark.rs)
3. 了解 [Cargo.toml](../../Cargo.toml) 中的 feature flags

### 高级用户
1. 深入研究源代码中的实现细节
2. 贡献改进或新功能
3. 针对特定用例优化性能
4. 在库的基础上构建自定义扩展

## 🔍 查找信息

### 按 Feature Flag
- `process` → [进程与窗口](modules/process_window.md)
- `keyboard`, `mouse` → [输入控制](modules/input.md)
- `memory` → [内存操作](modules/memory.md)
- `memory_hook` → [内存钩子](modules/memory_hook.md)
- `memory_aobscan` → [字节扫描](modules/memory_aobscan.md)
- `dxgi` → [屏幕捕获](modules/dxgi.md)
- `template_matcher` → [模板匹配](modules/template_matcher.md)
- `script_engine` → [脚本引擎](modules/script_engine.md)
- `dll_injector` → [DLL注入](modules/dll_injector.md)

### 按示例文件
所有示例都在 `examples/` 目录中：
- `_debug_*.rs` - 调试/测试示例
- `*_benchmark.rs` - 性能基准测试
- 其他 - 功能演示

## 💡 提示

1. **从简单开始**: 在学习钩子和扫描之前先掌握基础模块
2. **使用 Feature Flags**: 只启用所需功能以减少编译时间
3. **查看示例**: 大多数模块都有可运行的工作示例
4. **阅读错误信息**: Rust 编译器提供有用的指导
5. **社区支持**: 查看 GitHub issues 了解常见问题和解决方案

## 🤝 贡献文档

如果你发现错误或想改进文档：

1. Fork 仓库
2. 编辑 markdown 文件
3. 提交 pull request
4. 确保同时更新中英文版本

## 📞 支持

- **GitHub Issues**: 报告错误或请求功能
- **示例代码**: 运行示例代码查看功能演示
- **源代码**: 阅读 Rust 源文件中的内联文档

---

**语言**: [English](../en/INDEX.md) | [中文](INDEX.md)
