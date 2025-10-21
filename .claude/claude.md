# WGPU Canvas Editor - Project Context

## Table of Contents
1. [Project Purpose](#project-purpose)
2. [Technology Stack](#technology-stack)
3. [Architecture](#architecture)
4. [Module Structure](#module-structure)
5. [Module Organization & Optimization](#module-organization--optimization)
6. [Coding Conventions](#coding-conventions)
7. [Development Workflow](#development-workflow)
8. [Current Features](#current-features)
9. [Contributing Guidelines](#contributing-guidelines)
10. [Resources](#resources)

---

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

## Module Organization & Optimization

### File and Module Organization Principles

**When to Create a New Module/File:**
- Feature has **3+ related functions/structs** that form a cohesive unit
- File exceeds **300-500 lines** and has distinct responsibilities
- Code can be **reused across multiple modules**
- Clear **separation of concerns** can be achieved
- Logic is complex enough to benefit from **isolated testing**

### Module Size Guidelines

**Optimal File Sizes:**
- **Single file**: 100-300 lines (ideal)
- **Consider splitting**: 300-500 lines
- **Must split**: 500+ lines (unless single large data structure)
- **Module directory**: 3-10 files (ideal for cognitive load)

**Complexity Indicators (time to split):**
- More than 10 public functions/methods
- Multiple distinct responsibilities
- Difficult to understand without scrolling
- Tests become unwieldy (>500 lines)

### Module Hierarchy Best Practices

```rust
// Good: Feature-based organization
src/
├── shapes/              // Feature module
│   ├── mod.rs          // Public API, re-exports
│   ├── rectangle.rs    // Shape implementation
│   ├── circle.rs       // Shape implementation
│   ├── traits.rs       // Shared traits
│   └── utils.rs        // Shape-specific utilities
└── rendering/          // Feature module
    ├── mod.rs
    ├── pipeline.rs     // Render pipeline
    ├── shaders.rs      // Shader management
    └── batch.rs        // Batch rendering

// Bad: Type-based organization
src/
├── traits/             // All traits together
├── structs/            // All structs together
└── utils/              // Generic utils dump
```

### File Splitting Strategy

**When splitting a large file:**

1. **Identify logical boundaries**
   - Group by feature/responsibility
   - Minimize cross-dependencies
   - Keep related code together

2. **Create a module directory**
   ```bash
   # Before: src/shapes.rs (1000 lines)
   # After:
   src/shapes/
   ├── mod.rs          # Public API only
   ├── rectangle.rs    # Rectangle implementation
   ├── circle.rs       # Circle implementation
   └── common.rs       # Shared utilities
   ```

3. **Use mod.rs for re-exports**
   ```rust
   // src/shapes/mod.rs
   mod rectangle;
   mod circle;
   mod common;

   pub use rectangle::Rectangle;
   pub use circle::Circle;
   // Keep internal items private
   ```

### Dependency Management

**Module Dependency Rules:**
- **Minimize coupling**: Modules should depend on traits, not concrete types
- **Avoid circular dependencies**: Use dependency injection or traits
- **Layer dependencies**: Higher layers depend on lower layers only
  ```
  app → state → renderer → wgpu
  (UI)  (logic) (graphics) (API)
  ```

### Code Reusability Guidelines

**Extract reusable code when:**
- Same logic used in **2+ places** (DRY principle)
- Logic is **pure/stateless** (easier to reuse)
- Functionality is **generic enough** for multiple contexts

**Create utility modules for:**
- Mathematical operations (geometry, transforms)
- Data structure conversions
- Common validation logic
- Platform abstraction helpers

### Module Documentation

**Every module (mod.rs) should have:**
```rust
//! Brief one-line description.
//!
//! # Purpose
//! Detailed explanation of what this module provides.
//!
//! # Example
//! ```
//! use crate::module::Type;
//! // Usage example
//! ```
```

### Refactoring Triggers

**Refactor when you notice:**
- God objects (classes doing too much)
- Feature envy (method using data from another class extensively)
- Long parameter lists (>4 parameters)
- Duplicated code across modules
- Tight coupling between unrelated features

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

### Development Cycle

1. **Plan**: Break down features into small tasks
2. **Implement**: Follow coding conventions and module organization principles
3. **Test**: Write unit tests and integration tests
4. **Verify**: Run pre-commit checklist (below)
5. **Commit**: Use Conventional Commits format
6. **Review**: Ensure code quality and cross-platform compatibility

### General Guidelines

**Dependency Management:**
- Use latest stable versions from crates.io
- Run `cargo update` regularly to check for updates
- Document why specific versions are pinned (if any)

**Version Control:**
- One logical change per commit
- Keep commits atomic and reversible
- Write meaningful commit messages

**Testing Strategy:**
- Unit tests for individual functions/modules
- Integration tests for feature workflows
- Manual testing on both desktop and web

**Documentation:**
- Update inline `///` doc comments for public APIs
- Update this CLAUDE.md when architecture changes
- Keep README.md user-facing and concise

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

### Code Organization
- Follow **Module Organization & Optimization** principles above
- Keep files under 300-500 lines when possible
- Create feature-based modules, not type-based modules
- Extract reusable code into utility modules

### Code Quality
- Write clear commit messages following Conventional Commits format
- Add tests for new features (minimum 70% coverage for new code)
- Update documentation for API changes
- Document all public APIs with `///` doc comments
- Ensure cross-platform compatibility (desktop + web)

### Before Committing
- Run **pre-commit verification checklist** (see Development Workflow)
- Ensure no clippy warnings
- Check that all tests pass
- Verify both desktop and WASM builds work

## Resources

- [wgpu Documentation](https://docs.rs/wgpu/)
- [egui Documentation](https://docs.rs/egui/)
- [Rust WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [Learn wgpu Tutorial](https://sotrh.github.io/learn-wgpu/)
- [winit Documentation](https://docs.rs/winit/)
