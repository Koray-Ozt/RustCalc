# RustCalc v0.1.0-alpha.2

This late-alpha release updates RustCalc to FerriteDB `v0.1.0-beta.1` and adds installable Linux artifacts.

## Highlights

- Uses FerriteDB's versioned format 1, strict WAL durability, bounded recovery, and hardened metadata handling.
- Stores application data under the platform data directory by default (`$XDG_DATA_HOME/rust-calc` or `~/.local/share/rust-calc`).
- Provides Ubuntu 24.04/Linux Mint 22 artifacts: a Debian package and a standalone dynamically linked x86_64 binary.
- Adds a tag-driven release workflow with locked tests, clippy, formatting, checksum generation, and prerelease publication.

## Install on Ubuntu 24.04 or Linux Mint 22

Download `rust-calc_0.1.0-alpha.2_amd64.deb`, then run:

```bash
sudo apt install ./rust-calc_0.1.0-alpha.2_amd64.deb
```

The package installs the application menu entry and GTK runtime dependency.

## Upgrade note

RustCalc alpha.1 databases use FerriteDB's unversioned alpha format. FerriteDB beta deliberately refuses to open those databases implicitly. Calculation history is non-critical example data, so this release starts with a new format-1 database at the platform data path. The old repository-relative `data/history.ferrite` directory is left untouched.

## Limitations

RustCalc remains an example project and FerriteDB remains unaudited beta software. Do not use either for production, security-critical workloads, or irreplaceable data. The standalone binary requires GTK 3 runtime libraries; prefer the Debian package for automatic dependency installation.

The GTK 3 Rust bindings used by this late-alpha build are unmaintained and `cargo audit` reports the known `glib::VariantStrIter` unsoundness advisory. RustCalc does not use that iterator API; migrating the UI toolkit remains a requirement before a stable release.

RustCalc embeds FerriteDB under FSL-1.1-ALv2. See the attached `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_LICENSES.md` for attribution and complete bundled dependency license texts.
