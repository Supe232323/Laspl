# Laspl

**Simple. Intuitive. Secure.**

A local-first password manager with a clean [Standard Notes](https://standardnotes.com)-inspired interface.

![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
![Tauri](https://img.shields.io/badge/Tauri-2-orange)
![Rust](https://img.shields.io/badge/Rust-1.77+-red)
![Svelte](https://img.shields.io/badge/Svelte-5-ff3e00)

## Features

- Master password unlock (Argon2id)
- AES-256-GCM encrypted vault file
- Add / view / edit / delete / search / favorite entries
- Password generator (length + symbols)
- Categories & favorites
- Fully offline — no network required
- Dark theme matching Standard Notes aesthetic
- Cross-platform (Windows, macOS, Linux)

## Tech Stack

| Layer     | Tech                          |
|-----------|-------------------------------|
| Backend   | Rust + Tauri 2                |
| Crypto    | Argon2id + AES-256-GCM        |
| Frontend  | Svelte 5 + Vite + Tailwind    |
| UI        | Standard Notes inspired dark  |

## Prerequisites

- [Rust](https://rustup.rs/) (1.77+)
- [Node.js](https://nodejs.org/) 20+
- System dependencies for Tauri (see [Tauri docs](https://v2.tauri.app/start/prerequisites/))

```bash
# Install Tauri CLI
cargo install tauri-cli --version "^2"
```

## Quick Start

```bash
git clone https://github.com/Supe232323/Laspl.git
cd Laspl
npm install
npm run tauri dev
```

First unlock will create the vault at:

* **Linux:** `~/.local/share/laspl/vault.laspl`
* **macOS:** `~/Library/Application Support/laspl/vault.laspl`
* **Windows:** `%APPDATA%\laspl\vault.laspl`

## Project Structure

```
Laspl/
├── .github/workflows/     # CI + Release
├── src/                   # Svelte frontend
│   ├── components/        # UI (Sidebar, List, Detail, Modal…)
│   ├── lib/               # stores, types, Tauri bindings
│   └── app.css
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── crypto.rs      # Argon2id + AES-GCM
│   │   ├── vault.rs       # Vault file + commands
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── capabilities/
│   ├── icons/
│   └── tauri.conf.json
├── package.json
└── README.md
```

## Scripts

| Command               | Description                |
|-----------------------|----------------------------|
| `npm run tauri dev`   | Dev mode with hot reload   |
| `npm run tauri build` | Production build           |
| `npm run build`       | Frontend only              |
| `npm run dev`         | Vite only (no Tauri)       |

## Security

- Master password never stored
- Key derived with Argon2id (64 MiB memory, 3 iterations)
- Vault encrypted with AES-256-GCM (authenticated)
- Memory zeroized on lock
- No telemetry, no cloud, no phone-home

## CI / CD

- **CI** (`.github/workflows/ci.yml`): frontend build + Rust clippy/fmt/check on every push/PR
- **Release** (`.github/workflows/release.yml`): builds installers for Linux / Windows / macOS on `v*` tags

## macOS installation

Laspl is not currently signed or notarized by Apple. macOS may block its
first launch.

1. Move **Laspl.app** to the Applications folder.
2. Try to open Laspl once and dismiss the security warning.
3. Open **System Settings → Privacy & Security**.
4. Scroll down to the **Security** section.
5. Find the message that Laspl was blocked and click **Open Anyway**.
6. Authenticate with your password or Touch ID, then confirm **Open Anyway**.

The Open Anyway option is only available for about one hour after attempting
to launch the app.

### Terminal fallback

Only use this if you downloaded Laspl from this official GitHub release and
trust the file:

```bash
xattr -dr com.apple.quarantine /Applications/Laspl.app
```

Alternatively, after attempting to open the app, go to:

**System Settings → Privacy & Security → Open Anyway**

## License

GNU Affero General Public License v3.0 — see [LICENSE](LICENSE)

---

Made by [Supe232323](https://github.com/Supe232323)

