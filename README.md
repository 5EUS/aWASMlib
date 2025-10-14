# aWASMlib

A high-performance, extensible media aggregation platform built with Rust and WebAssembly Component Model (WASM). aWASMlib serves as a robust backend engine for media aggregation applications, providing unified interfaces for searching and accessing manga, anime, comic books, educational books, TV and movie content through sandboxed plugins.

## Features

- 🔌 **Plugin Architecture**: Load and execute WASM plugins safely in a sandboxed environment
- 🔒 **Security**: WASI-based sandboxing ensures plugins cannot access host system resources or undocumented web hosts
- 🚀 **Performance**: Built with Rust and Wasmtime for optimal performance
- 🌐 **Web Standards**: Uses WebAssembly Component Model for interoperability
- 📚 **Multi-Media**: Support for many types of media
- 🔧 **Backend Engine**: Rust core designed to power frontend applications (Dart/Flutter/CLI)
- 💾 **Data Management**: Plugin management, database storage, and configuration handling
- 📱 **Cross-Platform**: Backend suitable for mobile, desktop, and web applications

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  Frontend Apps  │───▶│                  │───▶│  WASM Plugins   │
│ (Dart/Flutter)  │    │   aWASM Backend  │    │                 │
└─────────────────┘    │                  │    └─────────────────┘
┌─────────────────┐    │  ┌─────────────┐ │           │
│   CLI Frontend  │───▶│  │   Plugin    │ │           ▼
└─────────────────┘    │  │  Manager    │ │    ┌──────────────┐
                       │  └─────────────┘ │    │  WASI/HTTP   │
                       │  ┌─────────────┐ │    │  Sandboxing  │
                       │  │  Database   │ │    └──────────────┘
                       │  │   & Config  │ │
                       │  └─────────────┘ │
                       └──────────────────┘
                              │
                              ▼
                       ┌──────────────┐
                       │  Wasmtime    │
                       │  Engine      │
                       └──────────────┘
```

### Security Considerations

Plugins run in a sandboxed WASI environment with the following restrictions:

- ✅ **Allowed**: HTTP requests (via WASI-HTTP) within documented limits (.toml file)
- ✅ **Allowed**: Memory allocation within limits
- ✅ **Allowed**: Basic computation and string manipulation
- ❌ **Blocked**: File system access
- ❌ **Blocked**: Process spawning
- ❌ **Blocked**: Environment variable access
- ❌ **Blocked**: Network access outside HTTP
