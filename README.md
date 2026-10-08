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
- Soft-delete (Trash) + restore
- Password & passphrase generator (EFF wordlist)
- Categories, favorites, recently used
- Auto-lock + clipboard auto-clear
- Change master password, encrypted export/import
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
* **Windows:** `%APPDATA%\\laspl\\vault.laspl`

A previous-good copy is kept next to it as `vault.laspl.bak` after each successful save.

## Downloads & verification

**Only download builds from [this repo’s Releases](https://github.com/Supe232323/Laspl/releases).**

Laspl is **not code-signed or notarized** (Apple’s developer program is paid). That is intentional for now — not a claim that the binary is signed.

Each release includes a **`SHA256SUMS`** file. Verify before installing:

```bash
# Linux / macOS
sha256sum -c SHA256SUMS

# Or check one file
sha256sum Laspl_*.AppImage   # compare to the line in SHA256SUMS
```

```powershell
# Windows (PowerShell)
Get-FileHash .\\Laspl_*.msi -Algorithm SHA256
# Compare to the matching line in SHA256SUMS
```

## macOS installation (unsigned)

macOS may block the first launch.

1. Move **Laspl.app** to the Applications folder.
2. Try to open Laspl once and dismiss the security warning.
3. Open **System Settings → Privacy & Security**.
4. Scroll to **Security**, find the Laspl message, click **Open Anyway**.
5. Authenticate, then confirm **Open Anyway**.

“Open Anyway” is only offered for about an hour after the blocked launch.

### Terminal fallback

Only if you downloaded from this repo’s Releases and verified the checksum:

```bash
xattr -dr com.apple.quarantine /Applications/Laspl.app
```

## Security

- Master password never stored
- Key derived with Argon2id (64 MiB memory, 3 iterations)
- Vault encrypted with AES-256-GCM (authenticated)
- Atomic vault writes + automatic `.bak` backup
- Memory zeroized on lock
- No telemetry, no cloud, no phone-home

## Project Structure

```
Laspl/
├── .github/workflows/     # CI + Release
├── src/                   # Svelte frontend
│   ├── components/
│   ├── lib/
│   └── app.css
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── crypto.rs
│   │   ├── vault.rs
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

## CI / CD

- **CI** (`.github/workflows/ci.yml`): frontend build + Rust fmt / clippy / test / build
- **Release** (`.github/workflows/release.yml`): installers for Linux / Windows / macOS on `v*` tags, plus `SHA256SUMS`

## License

GNU Affero General Public License v3.0 — see [LICENSE](LICENSE)

---

Made by [Supe232323](https://github.com/Supe232323)
