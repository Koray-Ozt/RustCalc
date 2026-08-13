#!/usr/bin/env bash
set -euo pipefail

version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n1)
arch=${RUST_CALC_ARCH:-amd64}
out_dir=${1:-dist}
root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT

cargo build --locked --release
python3 scripts/generate-third-party-licenses.py

pkg="$root/rust-calc_${version}_${arch}"
install -Dm755 target/release/rust-calc "$pkg/usr/bin/rust-calc"
install -Dm644 packaging/dev.koray.rustcalc.desktop \
    "$pkg/usr/share/applications/dev.koray.rustcalc.desktop"
install -Dm644 README.md "$pkg/usr/share/doc/rust-calc/README.md"
install -Dm644 SECURITY.md "$pkg/usr/share/doc/rust-calc/SECURITY.md"
install -Dm644 THIRD_PARTY_NOTICES.md "$pkg/usr/share/doc/rust-calc/THIRD_PARTY_NOTICES.md"
install -Dm644 THIRD_PARTY_LICENSES.md "$pkg/usr/share/doc/rust-calc/THIRD_PARTY_LICENSES.md"
install -Dm644 packaging/copyright "$pkg/usr/share/doc/rust-calc/copyright"

installed_size=$(du -sk "$pkg" | cut -f1)
mkdir -p "$pkg/DEBIAN"
cat > "$pkg/DEBIAN/control" <<EOF
Package: rust-calc
Version: $version
Section: utils
Priority: optional
Architecture: $arch
Installed-Size: $installed_size
Depends: libc6 (>= 2.34), libgcc-s1, libgtk-3-0 | libgtk-3-0t64
Maintainer: Koray Öztürk
Homepage: https://github.com/Koray-Ozt/RustCalc
Description: GTK calculator demonstrating embedded FerriteDB persistence
 RustCalc stores structured calculation history and language preferences in a
 local FerriteDB database.
EOF

mkdir -p "$out_dir"
dpkg-deb --root-owner-group --build "$pkg" "$out_dir/rust-calc_${version}_${arch}.deb"
cp target/release/rust-calc "$out_dir/rust-calc-linux-x86_64"
chmod 0755 "$out_dir/rust-calc-linux-x86_64"
(
    cd "$out_dir"
    sha256sum "rust-calc_${version}_${arch}.deb" rust-calc-linux-x86_64 > SHA256SUMS
)
