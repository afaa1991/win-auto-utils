# 快速开始指南 - 找到你需要的文档

[English Version](../en/QUICK_START.md)

本指南帮助你根据目标快速找到所需的文档。

## 🎯 我想要...

### 📖 了解项目
- **初次使用 win-auto-utils？** → 从 [README](../../README.md) 开始
- **想查看所有模块？** → 阅读 [模块概览](modules/overview.md)
- **需要完整导航？** → 查看 [文档索引](INDEX.md)

### 🎮 自动化游戏
**从这里开始：**
1. [进程管理](modules/process_window.md) - 查找游戏进程
2. [内存读取](modules/memory.md) - 读取生命值、弹药、坐标
3. [字节扫描](modules/memory_aobscan.md) - 查找动态地址
4. [输入模拟](modules/input.md) - 发送按键和鼠标点击
5. [屏幕捕获](modules/dxgi.md) - 监控游戏画面

**高级功能：**
- [内存钩子](modules/memory_hook.md) - 拦截游戏函数（无敌模式等）
- [寄存器提取](modules/memory_hook.md) - 自动捕获玩家状态

### 🧪 测试桌面应用程序
**从这里开始：**
1. [窗口管理](modules/process_window.md) - 控制应用窗口
2. [输入控制](modules/input.md) - 点击按钮、输入文本
3. [模板匹配](modules/template_matcher.md) - 验证 UI 元素出现
4. [脚本引擎](modules/script_engine.md) - 编写可重用的测试脚本

### 🔬 逆向工程程序
**从这里开始：**
1. [内存操作](modules/memory.md) - 检查进程内存
2. [字节扫描](modules/memory_aobscan.md) - 定位代码模式
3. [地址解析](modules/memory_resolver.md) - 跟踪指针链
4. [内存钩子](modules/memory_hook.md) - 监控函数调用
5. [寄存器提取](modules/memory_hook.md) - 捕获 CPU 寄存器值
6. [DLL注入](modules/dll_injector.md) - 注入调试工具

### 🤖 构建桌面机器人
**从这里开始：**
1. [脚本引擎](modules/script_engine.md) - 创建自动化脚本
2. [输入控制](modules/input.md) - 控制键盘和鼠标
3. [窗口管理](modules/process_window.md) - 管理应用程序窗口
4. [屏幕捕获](modules/dxgi.md) - 视觉验证
5. [模板匹配](modules/template_matcher.md) - 检测 UI 元素

### 💻 只是浏览示例
查看 `examples/` 目录：
- **基础操作**: `memory_operations.rs`, `mouse_control.rs`
- **高级功能**: `aobscan_benchmark.rs`, `template_matching.rs`
- **调试/测试**: 以 `_debug_` 开头的文件

运行任何示例：
```bash
cargo run --example example_name --features "required_feature"
```

## 🚀 快速代码示例

### 读取进程内存
```rust
use win_auto_utils::memory::read_memory_t;

let value: i32 = read_memory_t(handle, address)?;
```
📖 了解更多：[内存操作](modules/memory.md)

### 模拟键盘输入
```rust
use win_auto_utils::keyboard::key_press;

key_press(handle, 0x41)?; // 按下 'A'
```
📖 了解更多：[输入控制](modules/input.md)

### 查找窗口
```rust
use win_auto_utils::hwnd::get_hwnd_by_title;

let hwnd = get_hwnd_by_title("Notepad")?;
```
📖 了解更多：[进程与窗口](modules/process_window.md)

### 扫描字节模式
```rust
use win_auto_utils::memory_aobscan::AobScanner;

let scanner = AobScanner::new(handle)?;
let results = scanner.scan("48 89 5C 24 ??")?;
```
📖 了解更多：[字节扫描](modules/memory_aobscan.md)

### 钩住函数
```rust
use win_auto_utils::memory_hook::TrampolineHook;

let mut hook = TrampolineHook::x86(handle, addr, shellcode);
hook.install()?;
```
📖 了解更多：[内存钩子](modules/memory_hook.md)

### 运行脚本
```rust
use win_auto_utils::script_engine::ScriptEngine;

let script = r#"
    loop 5
        key VK_SPACE
        sleep 100
    end
"#;

let mut engine = ScriptEngine::new();
engine.execute(script)?;
```
📖 了解更多：[脚本引擎](modules/script_engine.md)

## 📊 按 Feature Flag 选择

如果你知道需要的 feature flag：

| Feature | 模块文档 | 用例 |
|---------|-----------|------|
| `process` | [进程与窗口](modules/process_window.md) | 查找进程 |
| `keyboard` | [输入控制](modules/input.md) | 发送按键 |
| `mouse` | [输入控制](modules/input.md) | 控制鼠标 |
| `memory` | [内存操作](modules/memory.md) | 读写内存 |
| `memory_hook` | [内存钩子](modules/memory_hook.md) | 拦截函数 |
| `memory_aobscan` | [字节扫描](modules/memory_aobscan.md) | 模式扫描 |
| `dxgi` | [屏幕捕获](modules/dxgi.md) | 捕获屏幕 |
| `template_matcher` | [模板匹配](modules/template_matcher.md) | 图像匹配 |
| `script_engine` | [脚本引擎](modules/script_engine.md) | 运行脚本 |
| `dll_injector` | [DLL注入](modules/dll_injector.md) | 注入 DLL |

## 🎓 学习路径

### 初学者（第 1-3 天）
1. 阅读 [README](../../README.md)
2. 运行基础示例：`memory_operations.rs`, `mouse_control.rs`
3. 了解 [Cargo.toml](../../Cargo.toml) 中的 feature flags

### 中级用户（第 1-2 周）
1. 学习 [内存钩子](modules/memory_hook.md)
2. 掌握 [字节扫描](modules/memory_aobscan.md)
3. 探索 [脚本引擎](modules/script_engine.md)
4. 运行基准测试示例

### 高级用户（1 个月以上）
1. 阅读源代码了解实现细节
2. 贡献改进或新功能
3. 针对特定用例优化
4. 构建自定义扩展

## ❓ 常见问题

**Q: 我应该从哪个模块开始？**  
A: 取决于你的目标。参见上面的"我想要..."部分。

**Q: 如何启用 features？**  
A: 在 `Cargo.toml` 中添加：`features = ["memory", "keyboard"]`

**Q: 示例在哪里？**  
A: 在 `examples/` 目录中。使用 `cargo run --example name` 运行

**Q: 有中文文档吗？**  
A: 有！所有文档都有中文版本。查看每个页面顶部的链接。

**Q: 如何贡献？**  
A: Fork 仓库，进行修改，提交 PR。参见 [文档指南](DOCUMENTATION_GUIDE.md)。

## 🔗 有用链接

- **主 README**: [English](../../README.md) | [中文](../zh/README.md)
- **完整索引**: [English](INDEX.md) | [中文](../zh/INDEX.md)
- **模块概览**: [English](modules/overview.md) | [中文](../zh/modules/overview.md)
- **GitHub 仓库**: [win-auto-utils](https://github.com/your-repo/win-auto-utils)
- **示例目录**: `examples/`

---

**仍然迷茫？** 从 [模块概览](modules/overview.md) 开始 - 它解释了一切！

**语言**: [English](../en/QUICK_START.md) | [中文](QUICK_START.md)
