# WGPU Canvas Editor - Project Context

## Project Purpose

A Figma-style canvas editor using Rust and wgpu, providing high-performance graphics rendering that works across desktop and web platforms through WebAssembly.

## Technology Stack

- **Rust 2024 Edition**: Memory safety, performance, strong type system
- **wgpu**: Cross-platform graphics API (Vulkan, Metal, DirectX 12, WebGPU)
- **egui**: Immediate mode GUI framework for UI
- **winit**: Cross-platform window management
- **WebAssembly**: Near-native performance in browsers

## Architecture

**Application Layer** (State Management, Input Handling)
↓
**UI Layer** (egui, JSON Editor)
↓
**Renderer Layer** (wgpu Pipeline, Shaders)
↓
**Platform Abstraction** (winit for Desktop, web-sys for WASM)

## Module Structure

```
src/
├── lib.rs              # Common library code, WASM exports
├── main.rs             # Desktop entry point
├── app.rs              # Application UI logic
├── state.rs            # Application state management
├── input.rs            # Input event handling
├── json_editor/        # JSON editor with graph visualizer
│   ├── mod.rs
│   ├── editor.rs       # JSON editing functionality
│   └── graph.rs        # Graph visualization
└── renderer/
    ├── mod.rs
    └── canvas.rs       # Canvas rendering logic
```

## Coding Conventions

### Rust Style
- Follow official Rust style guide (`rustfmt`)
- Use `cargo clippy` for linting
- Prefer explicit error handling with `Result<T, E>`
- Document public APIs with `///` doc comments

### Naming
- Types: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Platform-Specific Code
```rust
#[cfg(target_arch = "wasm32")]
fn platform_specific_init() { /* WASM */ }

#[cfg(not(target_arch = "wasm32"))]
fn platform_specific_init() { /* Desktop */ }
```

## Development Workflow

### General Guidelines
- **Dependency Management**: Always use the latest stable version from crates.io
- **Version Control**: Make git commits for each logical unit of work
- **Testing**: Write tests for new features
- **Documentation**: Update docs when APIs change

### Pre-Commit Verification Checklist

Before making any git commit, **ALWAYS** verify these items:

1. **Code Formatting**
   ```bash
   cargo fmt --all -- --check
   ```
   If this fails, run `cargo fmt --all` to fix.

2. **Linting (Clippy)**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
   Fix all warnings and errors.

3. **Tests**
   ```bash
   cargo test --all
   ```
   Ensure all tests pass.

4. **Desktop Build**
   ```bash
   cargo build --release
   ```
   Verify builds correctly.

5. **WASM Build** (if web-related changes)
   ```bash
   wasm-pack build --target web --out-dir web/pkg
   cd web && npm run serve
   ```
   Test in WebGPU-compatible browser (Chrome 113+, Firefox 121+, Safari 18+).

6. **Documentation**
   - Update relevant documentation if APIs changed
   - Ensure doc comments are accurate

**Note**: These checks mirror CI pipeline requirements. Running them locally catches issues early.

## Current Features

### Phase 1: Foundation ✅
- [x] Basic wgpu renderer setup
- [x] Cross-platform window management (desktop + web)
- [x] egui integration for UI
- [x] JSON editor with syntax validation
- [x] JSON graph visualizer
- [x] Focus management in UI
- [x] WASM build support

### Phase 2: Core Canvas Features (In Progress)
- [ ] Shape primitives (rectangle, circle, line)
- [ ] Shape selection and manipulation
- [ ] Pan and zoom
- [ ] Grid and guides

### Phase 3: Advanced Editing (Future)
- [ ] Layer management
- [ ] Undo/redo system
- [ ] Copy/paste
- [ ] Keyboard shortcuts

### Phase 4: Polish (Future)
- [ ] Export to PNG/SVG
- [ ] Performance optimization
- [ ] Mobile touch support

## Contributing Guidelines

- Write clear commit messages following Conventional Commits format
- Add tests for new features
- Update documentation for API changes
- Ensure cross-platform compatibility (desktop + web)
- Run pre-commit verification checklist before committing

## Resources

- [wgpu Documentation](https://docs.rs/wgpu/)
- [egui Documentation](https://docs.rs/egui/)
- [Rust WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [Learn wgpu Tutorial](https://sotrh.github.io/learn-wgpu/)
- [winit Documentation](https://docs.rs/winit/)
