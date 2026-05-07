# Memory Hook Module

[中文文档](../../zh/modules/memory_hook.md) | [Back to Overview](overview.md)

The `memory_hook` module provides advanced function interception capabilities through inline hooks and trampoline hooks, allowing you to modify or monitor program execution at runtime.

## Features

- **Inline Hooks**: Replace target instructions with custom shellcode
- **Trampoline Hooks**: Preserve original functionality while intercepting calls
- **Register Extraction**: Automatically capture CPU register values at hook points
- **Automatic Memory Management**: RAII-based cleanup prevents memory leaks
- **Warning**: In this mode, your shellcode must handle all logic, including returning to the caller when needed.

## Use Cases

- **Logging**: Monitor function calls and arguments
- **Modification**: Alter function behavior at runtime
- **Security**: Implement security checks or restrictions
