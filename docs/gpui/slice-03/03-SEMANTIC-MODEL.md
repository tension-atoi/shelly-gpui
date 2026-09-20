# Shelly GPUI — Slice-03 Semantic Model Specification

**Module**: `src/state/semantic.rs`  
**Purpose**: Provide strongly-typed, test-verified domain primitives for package inspection, dependency navigation, capability detection, and CLI command generation.

======================================================================
1. DEPENDENCY DOMAIN MODEL
======================================================================

### 1.1 `DependencyKind`
Categorizes dependencies according to their lifecycle requirements:
- `Runtime`: Direct runtime execution dependencies (e.g. `glibc`, `pcre2`).
- `Build`: Compilation and packaging requirements (e.g. `cargo`, `go`, `git`).
- `Optional`: Non-mandatory features or integrations (e.g. `bat: colored pkgbuild printing`).
- `RequiredBy`: Reverse dependencies requiring the current package.

### 1.2 `DependencyRef`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyRef {
    pub name: String,
    pub constraint: Option<String>,
    pub description: Option<String>,
    pub kind: DependencyKind,
}
```

#### Parsing Rules (`DependencyRef::parse(raw: &str, kind: DependencyKind)`)
1. Description splitting: If the raw string contains a colon (`:`), the prefix is taken as the dependency specification, and the suffix (trimmed) is stored as `description`.
2. Constraint detection: Looks for version comparison operators (`>=`, `<=`, `=`, `>`, `<`).
   - If found: Splits into clean package `name` and `constraint` (operator + version).
   - If not found: `name` is the trimmed input, `constraint` is `None`.
3. Test Coverage: Deterministically tested against unversioned (`curl`), versioned (`libalpm.so>=14`, `glibc=2.38`), and described strings (`bat: syntax highlighter`).

======================================================================
2. SEMANTIC TARGET NAVIGATION
======================================================================

### 2.1 `SemanticTarget`
Represents actionable, interactive items rendered across inspector tabs:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticTarget {
    Package(String),
    ExternalUrl(String),
    FilePath(String),
    CopyText(String),
}
```

#### Dispatch Semantics
- `Package(name)`: Triggers a package search in `AppSession` for the target dependency name.
- `ExternalUrl(url)`: Invokes `cx.open_url(&url)` to launch the user's default browser.
- `FilePath(path)`: Formats and displays localized filesystem paths.
- `CopyText(text)`: Copies string contents to clipboard with visual confirmation feedback.

======================================================================
3. PACKAGE CAPABILITY MATRIX
======================================================================

### 3.1 `PackageCapabilities`
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageCapabilities {
    pub can_install: bool,
    pub can_remove: bool,
    pub can_update: bool,
    pub has_pkgbuild: bool,
    pub has_file_tree: bool,
    pub has_dependencies: bool,
}
```

#### Derivation Rules (`PackageCapabilities::derive(pkg: &UnifiedPackage, alpm_detail: Option<&AlpmPackage>)`)
- `can_install`: True if `!pkg.is_installed()`.
- `can_remove`: True if `pkg.is_installed()`.
- `can_update`: True if `pkg.is_installed()` and `pkg.is_update_available()` (or part of update vector).
- `has_pkgbuild`: True strictly if `pkg.source == PackageSourceKind::Aur`.
- `has_file_tree`: False across current Shelly CLI releases (honest reporting).
- `has_dependencies`: True for ALPM and AUR packages; false for Flatpak/AppImage standalone bundles.

======================================================================
4. CANONICAL INSTALL COMMAND GENERATION
======================================================================

### 4.1 Syntax Alignment
Generated commands strictly mirror the official Shelly CLI command surface (`Seafoam-Labs/Shelly-ALPM`):
- **ALPM / Official Packages**: `shelly install standard <name>`
- **AUR Packages**: `shelly install aur <name>`
- **Flatpak Applications**: `shelly install flatpak <name>`
- **AppImages**: Display notice that AppImages are managed via file placement or updater.

Verified in test: `test_canonical_install_command`.
