#!/usr/bin/env python3
import json
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parent.parent
metadata = json.loads(
    subprocess.check_output(
        [
            "cargo",
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--filter-platform",
            "x86_64-unknown-linux-gnu",
        ],
        cwd=root,
        text=True,
    )
)

resolved = {node["id"] for node in metadata["resolve"]["nodes"]}
sections = []
for package in sorted(metadata["packages"], key=lambda item: (item["name"], item["version"])):
    if package["id"] not in resolved:
        continue
    if package["name"] == "rust-calc":
        continue

    manifest_dir = Path(package["manifest_path"]).parent
    license_files = []
    current = manifest_dir
    for _ in range(5):
        for pattern in ("LICENSE*", "COPYING*", "COPYRIGHT*"):
            license_files.extend(path for path in current.glob(pattern) if path.is_file())
        if license_files or current.parent == current:
            break
        current = current.parent

    unique_files = sorted(set(license_files))
    if not unique_files:
        raise SystemExit(f"no license file found for {package['name']} {package['version']}")

    source = package.get("source") or package.get("repository") or "local source"
    body = [
        f"## {package['name']} {package['version']}",
        "",
        f"- Declared license: `{package.get('license') or 'not declared'}`",
        f"- Source: `{source}`",
        "",
    ]
    for path in unique_files:
        body.extend([f"### {path.name}", "", "```text", path.read_text(errors="replace").rstrip(), "```", ""])
    sections.append("\n".join(body).rstrip())

output = "# Third-party license texts\n\n" + "\n\n".join(sections) + "\n"
(root / "THIRD_PARTY_LICENSES.md").write_text(output)
