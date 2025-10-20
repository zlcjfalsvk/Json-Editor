# WGPU Canvas Editor - Project Context

## Project Purpose

This project aims to build a Figma-style canvas editor using Rust and wgpu, providing a high-performance graphics rendering engine that works across desktop and web platforms through WebAssembly.

## Technology Choices

### Rust 2024 Edition
- Memory safety without garbage collection
- Excellent performance for graphics-intensive applications
- Strong type system prevents common bugs at compile time
- Zero-cost abstractions for high-level code with low-level performance

### wgpu
- Modern, safe, portable graphics API
- Built on top of Vulkan, Metal, DirectX 12, and WebGPU
- Cross-platform by design (desktop and web)
- Future-proof with WebGPU standard alignment

### WebAssembly (WASM)
- Near-native performance in web browsers
- Allows sharing the same codebase between desktop and web
- Growing ecosystem and browser support

### winit
- Cross-platform window creation and event handling
- Integrates well with wgpu
- Abstracts platform-specific windowing APIs

## Architecture Overview

```
┌─────────────────────────────────────┐
│         Application Layer           │
│  (State Management, Input Handling) │
└─────────────┬───────────────────────┘
              │
┌─────────────┴───────────────────────┐
│         Renderer Layer              │
│    (wgpu Pipeline, Shaders)         │
└─────────────┬───────────────────────┘
              │
┌─────────────┴───────────────────────┐
│      Platform Abstraction           │
│   (winit for Desktop, web-sys       │
│        for WASM)                    │
└─────────────────────────────────────┘
```

### Key Components

1. **State Management** (`src/state.rs`)
   - Maintains application state (canvas objects, selections, etc.)
   - Handles state updates and synchronization
   - Platform-agnostic core logic

2. **Renderer** (`src/renderer/`)
   - wgpu initialization and configuration
   - Render pipeline management
   - Shader compilation and management
   - Draw call orchestration

3. **Input Handler** (`src/input.rs`)
   - Mouse and keyboard event processing
   - Touch input for web (future)
   - Gesture recognition (future)

4. **Platform Entry Points**
   - Desktop: `src/main.rs` - Native window with winit
   - Web: `src/lib.rs` - WASM exports with wasm-bindgen

## Coding Conventions

### Rust Style
- Follow official Rust style guide (rustfmt)
- Use `cargo clippy` for linting
- Prefer explicit error handling with `Result<T, E>`
- Use `?` operator for error propagation
- Document public APIs with `///` doc comments

### Naming Conventions
- Types: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Error Handling
```rust
// Good: Explicit error types
fn initialize_renderer() -> Result<Renderer, RendererError> {
    // ...
}

// Use custom error types for different error categories
#[derive(Debug)]
enum RendererError {
    InitializationFailed(String),
    DeviceNotFound,
    // ...
}
```

### Platform-Specific Code
```rust
// Use cfg attributes for conditional compilation
#[cfg(target_arch = "wasm32")]
fn platform_specific_init() {
    // WASM-specific code
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_specific_init() {
    // Desktop-specific code
}
```

### Async/Await
- Use async for I/O operations
- Use `pollster::block_on` for desktop
- Use `wasm-bindgen-futures` for web

## Module Structure

```
src/
├── lib.rs              # Common library code, WASM exports
├── main.rs             # Desktop entry point
├── renderer/
│   ├── mod.rs          # Renderer module exports
│   └── canvas.rs       # Canvas rendering logic
├── state.rs            # Application state management
└── input.rs            # Input event handling
```

## Development Workflow

1. **Feature Development**
   - Create feature branch from main
   - Implement feature with tests
   - Run `cargo test` and `cargo clippy`
   - Format with `cargo fmt`
   - Test both desktop and WASM builds

2. **Testing Strategy**
   - Unit tests in each module
   - Integration tests in `tests/` directory
   - Manual testing on multiple platforms
   - Visual regression tests (future)

3. **Build Verification**
   ```bash
   # Desktop
   cargo build --release
   cargo run

   # WASM
   wasm-pack build --target web
   cd web && npm run serve
   ```

## Performance Considerations

- Minimize state updates to reduce re-renders
- Use instanced rendering for multiple objects
- Implement dirty flagging for selective updates
- Profile with `cargo-flamegraph` on desktop
- Use browser DevTools for WASM profiling

## Future Roadmap

### Phase 1: Foundation (Current)
- [x] Basic wgpu renderer setup
- [x] Cross-platform window management
- [x] Simple shape rendering (triangle)
- [ ] Basic input handling

### Phase 2: Core Canvas Features
- [ ] Shape primitives (rectangle, circle, line)
- [ ] Shape selection and manipulation
- [ ] Pan and zoom
- [ ] Grid and guides

### Phase 3: Advanced Editing
- [ ] Layer management
- [ ] Undo/redo system
- [ ] Copy/paste
- [ ] Keyboard shortcuts

### Phase 4: Polish
- [ ] Export to PNG/SVG
- [ ] Performance optimization
- [ ] Mobile touch support
- [ ] Collaborative editing (stretch goal)

## Contributing Guidelines

- Write clear commit messages following Conventional Commits
- Add tests for new features
- Update documentation for API changes
- Ensure cross-platform compatibility
- Run full test suite before submitting PR

## Resources

- [wgpu Documentation](https://docs.rs/wgpu/)
- [Rust WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [Learn wgpu Tutorial](https://sotrh.github.io/learn-wgpu/)
- [winit Documentation](https://docs.rs/winit/)
