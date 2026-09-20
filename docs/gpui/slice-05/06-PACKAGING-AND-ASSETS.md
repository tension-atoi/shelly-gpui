# Shelly GPUI — Slice-05 Packaging & Assets

## 1. Desktop Entry Specification

The application provides a compliant XDG desktop entry at `Shelly.Ui.Gpui/assets/com.shellyorg.shelly-gpui.desktop`.

### Desktop File Contents
```ini
[Desktop Entry]
Name=Shelly
GenericName=Package Manager
GenericName[fr]=Gestionnaire de paquets Arch Linux
Comment=Arch Linux package manager — official repos, AUR, Flatpak and AppImage
Comment[fr]=Gestionnaire de paquets Arch Linux — dépôts officiels, AUR, Flatpak et AppImage
Exec=shelly-gpui
Icon=shelly-gpui
Terminal=false
Type=Application
Categories=System;PackageManager;Settings;
Keywords=pacman;aur;flatpak;appimage;package;manager;arch;alpm;
StartupNotify=true
StartupWMClass=shelly-gpui
```

### Standards Compliance Guarantees
- **English Default Primary Strings**: `GenericName` and `Comment` use proper English syntax as the default desktop entry fields.
- **Localized Alternatives**: French translations are preserved via `[fr]` localization keys.
- **XDG Application Category**: Classified under `System;PackageManager;Settings;` for automatic inclusion in GNOME Software, KDE Discover, and desktop application menus.
- **StartupWMClass**: Set to `shelly-gpui` to ensure window grouping on Wayland compositors (Hyprland, Sway, GNOME Shell, KWin).

---

## 2. Compile-Time Asset Embedding

To guarantee that the application runs reliably across different launch contexts (e.g. launched from `/usr/bin`, user home directory, desktop icon, or systemd user service), all application icons are embedded into the compiled binary via `include_bytes!`:

```rust
impl AssetSource for AppIcons {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            "icons/browse.svg" => Some(include_bytes!("icons/browse.svg")),
            "icons/search.svg" => Some(include_bytes!("icons/search.svg")),
            "icons/installed.svg" => Some(include_bytes!("icons/installed.svg")),
            "icons/updates.svg" => Some(include_bytes!("icons/updates.svg")),
            "icons/news.svg" => Some(include_bytes!("icons/news.svg")),
            "icons/settings.svg" => Some(include_bytes!("icons/settings.svg")),
            "icons/overview.svg" => Some(include_bytes!("icons/overview.svg")),
            "icons/dependencies.svg" => Some(include_bytes!("icons/dependencies.svg")),
            "icons/files-build.svg" => Some(include_bytes!("icons/files-build.svg")),
            "icons/cards.svg" => Some(include_bytes!("icons/cards.svg")),
            "icons/table.svg" => Some(include_bytes!("icons/table.svg")),
            "icons/copy.svg" => Some(include_bytes!("icons/copy.svg")),
            "icons/external-url.svg" => Some(include_bytes!("icons/external-url.svg")),
            "icons/file-path.svg" => Some(include_bytes!("icons/file-path.svg")),
            "icons/collapse.svg" => Some(include_bytes!("icons/collapse.svg")),
            "icons/expand.svg" => Some(include_bytes!("icons/expand.svg")),
            "icons/trash.svg" => Some(include_bytes!("icons/trash.svg")),
            "icons/shelly.svg" => Some(include_bytes!("icons/shelly.svg")),
            "icons/check.svg" => Some(include_bytes!("icons/check.svg")),
            "icons/close.svg" => Some(include_bytes!("icons/close.svg")),
            "icons/info.svg" => Some(include_bytes!("icons/info.svg")),
            "icons/warning.svg" => Some(include_bytes!("icons/warning.svg")),
            _ => None,
        };
        Ok(bytes.map(Cow::Borrowed))
    }
}
```

### Self-Contained Execution Verification
Because `load()` references static byte arrays compiled into the binary text segment:
- No working directory dependency: launching `shelly-gpui` from `/tmp` or `/` succeeds without missing icon assets.
- Instantaneous resolution: zero filesystem syscalls (`stat`, `open`, `read`) during rendering.

---

## 3. Strict Arch Linux Packaging (`PKGBUILD-gpui`)

The package build recipe (`PKGBUILD-gpui`) enforces strict quality gates across all build phases:

### Locked & Non-Concealing Build Phases
```sh
prepare() {
    cd "${srcdir}/${pkgname}/Shelly.Ui.Gpui"
    cargo fetch --locked
}

build() {
    cd "${srcdir}/${pkgname}"

    # 1. Compile Shelly CLI backend (Zig)
    echo ">>> Compiling Shelly CLI backend (Zig)..."
    cd Shelly.Cli.Zig
    zig build -Doptimize=ReleaseSafe
    cd ..

    # 2. Compile Shelly GPUI frontend (Rust)
    echo ">>> Compiling Shelly GPUI frontend (Rust)..."
    cd Shelly.Ui.Gpui
    export CARGO_TARGET_DIR="target"
    SHELLY_BIN="${srcdir}/${pkgname}/Shelly.Cli.Zig/zig-out/bin/shelly" \
        cargo build --release --locked
}

check() {
    cd "${srcdir}/${pkgname}/Shelly.Ui.Gpui"
    export CARGO_TARGET_DIR="target"
    SHELLY_BIN="${srcdir}/${pkgname}/Shelly.Cli.Zig/zig-out/bin/shelly" \
        cargo test --release --locked
}
```

### Packaging Hardening Steps
1. **Zero Error Concealment**: Removed all `|| true`, `2>/dev/null`, and fallback build commands. Every cargo invocation (`fetch`, `build`, `test`) enforces `--locked`.
2. **Deterministic Release Verification**: Tests run in `check()` under `--release --locked` with `SHELLY_BIN` explicitly bound, verifying optimized runtime invariants and strict `#![deny(dead_code)]` compliance.
3. **Deterministic Staging**:
   - `/usr/lib/shelly/shelly`: Native Zig CLI binary.
   - `/usr/lib/shelly/shelly-gpui-bin`: Native Rust GPUI binary.
   - `/usr/bin/shelly-gpui`: Wrapper script injecting `SHELLY_BIN=/usr/lib/shelly/shelly`.
   - `/usr/share/applications/com.shellyorg.shelly-gpui.desktop`: XDG desktop entry.
   - `/usr/share/icons/hicolor/scalable/apps/shelly-gpui.svg`: System application icon.
   - `/usr/share/polkit-1/actions/com.shellyorg.shelly-gpui.policy`: Strictly installed Polkit authentication action without error suppression.
