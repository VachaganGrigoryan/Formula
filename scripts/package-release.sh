#!/usr/bin/env bash
# Build the desktop app for the OS this runs on and pack it into dist/ (one file).
# Used by .github/workflows/release.yml; also works locally.
#   Linux    dist/formula-<version>-linux-x86_64.tar.gz
#   Windows  dist/formula-<version>-windows-x86_64.zip   (run from Git Bash)
#   macOS    dist/formula-<version>-macos-universal.zip  (Formula.app, Apple Silicon + Intel)
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
build=(cargo build --release --features native-audio)
rm -rf dist
mkdir -p dist

case "$(uname -s)" in
Linux)
  "${build[@]}"
  tar czf "dist/formula-$version-linux-x86_64.tar.gz" -C target/release formula -C ../.. README.md
  ;;
MINGW* | MSYS* | CYGWIN*)
  "${build[@]}"
  7z a -tzip "dist/formula-$version-windows-x86_64.zip" ./target/release/formula.exe ./README.md >/dev/null
  ;;
Darwin)
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  "${build[@]}" --target aarch64-apple-darwin
  "${build[@]}" --target x86_64-apple-darwin

  app=target/Formula.app
  rm -rf "$app"
  mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
  lipo -create -output "$app/Contents/MacOS/Formula" \
    target/aarch64-apple-darwin/release/formula target/x86_64-apple-darwin/release/formula

  iconset=target/Formula.iconset
  rm -rf "$iconset"
  mkdir -p "$iconset"
  for size in 16 32 128 256 512; do
    sips -z $size $size assets/icon/formula.png --out "$iconset/icon_${size}x${size}.png" >/dev/null
    sips -z $((size * 2)) $((size * 2)) assets/icon/formula.png --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
  done
  iconutil -c icns "$iconset" -o "$app/Contents/Resources/Formula.icns"

  cat >"$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Formula</string>
  <key>CFBundleDisplayName</key><string>Formula</string>
  <key>CFBundleIdentifier</key><string>dev.vachagan.formula</string>
  <key>CFBundleExecutable</key><string>Formula</string>
  <key>CFBundleIconFile</key><string>Formula</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF
  # Ad-hoc signature: not notarized, but Apple Silicon refuses to run unsigned code at all.
  codesign --force --deep --sign - "$app"
  ditto -c -k --keepParent "$app" "dist/formula-$version-macos-universal.zip"
  ;;
*)
  echo "unsupported OS: $(uname -s)" >&2
  exit 1
  ;;
esac

ls -l dist
