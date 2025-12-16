# FLAD 2.0 - Sistema de Gestión de Contratos de Mantención

## 🚀 Build Automático Windows

Este repositorio incluye GitHub Actions configurado para build automático en Windows.

### Características del Build
- ✅ Build automático en cada push a `main`
- ✅ Genera instalador MSI (Windows Installer)
- ✅ Genera instalador NSIS (ejecutable)
- ✅ Cache de dependencias Rust y Node.js
- ✅ Artifacts descargables en cada build

### Arquitectura
- **Frontend**: SvelteKit + Vite
- **Backend**: Tauri 2.9.5 + Rust
- **Base de Datos**: SQLite
- **UI**: HTML5 + CSS3

### Dependencias Principales
- Node.js 20.x
- Rust stable
- Tauri CLI 2.9.5
- SQLx 0.8.6

### Build Local

#### Windows
```bash
npm install
npm run tauri:build
```

Instaladores generados en:
- `src-tauri/target/release/bundle/msi/FLAD_2.0.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/FLAD_2.0.0_x64-setup.exe`

#### macOS
```bash
npm install
npm run tauri:build
```

App generada en:
- `src-tauri/target/release/bundle/macos/FLAD.app`
- `src-tauri/target/release/bundle/dmg/FLAD_2.0.0_aarch64.dmg`

### Estructura del Proyecto
```
.
├── .github/workflows/      # GitHub Actions
├── src/                    # Frontend SvelteKit
├── src-tauri/             # Backend Rust/Tauri
│   ├── src/               # Código Rust
│   ├── sql/               # Schemas y migrations
│   └── Cargo.toml         # Dependencias Rust
├── static/                # Assets estáticos
└── package.json           # Dependencias Node.js
```

### Changelog v2.0.0
- ✅ Migración completa a SQLite
- ✅ Fix camelCase/snake_case en Rust
- ✅ Sistema de alertas para requerimientos vencidos
- ✅ Importación Excel/CSV mejorada
- ✅ Gestión de firmas digitales
- ✅ Cross-platform: Windows + macOS

### Descargar Builds

Los builds automáticos están disponibles en:
1. **Actions** → Último workflow exitoso → **Artifacts**
2. **Releases** → Versiones etiquetadas (v2.0.0, etc.)
