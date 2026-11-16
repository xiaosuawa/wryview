#!/usr/bin/env bash
# Repair the Linux wheel with auditwheel.
#
# GLib/GTK/soup libraries can never be bundled into the wheel: any system
# with webkit2gtk-4.1 installed provides the whole chain as system libs (the
# webkit .deb depends on them), and the GObject type registry is process-
# global — a second copy of a GObject-based library (gdk, gtk, libsoup,
# pango, …) breaks any process that also links the system one (e.g.
# LumiView's tao window → "cannot register existing type
# 'GdkDisplayManager'").
set -euo pipefail

excludes=(
  'libwebkit2gtk-4.1.*'
  'libjavascriptcoregtk-4.1.*'
  'libgtk-3.*'
  'libgdk-3.*'
  'libgdk_pixbuf-2.0.*'
  'libglib-2.0.*'
  'libgobject-2.0.*'
  'libgmodule-2.0.*'
  'libgio-2.0.*'
  'libsoup-3.*'
  'libpango-1.0.*'
  'libpangocairo-1.0.*'
  'libpangoft2-1.0.*'
  'libatk-1.0.*'
  'libatk-bridge-2.0.*'
  'libharfbuzz.*'
  'libfribidi.*'
  'libepoxy.*'
  'libcairo.*'
  'libcairo-gobject.*'
  'libxkbcommon.*'
  'libwayland-client.*'
  'libdbus-1.*'
  'libsqlite3.*'
  'libkrb5.*'
  'libgssapi_krb5.*'
  'libk5crypto.*'
  'libkrb5support.*'
  'libcom_err.*'
  'libkeyutils.*'
  'libnghttp2.*'
  'libpsl.*'
  'libidn2.*'
  'libunistring.*'
  'libbrotli*.*'
)

args=()
for pattern in "${excludes[@]}"; do
  args+=(--exclude "$pattern")
done

shopt -s nullglob
wheels=(dist/*.whl)
if ((${#wheels[@]} == 0)); then
  echo "no wheel found in dist/" >&2
  exit 1
fi

for wheel in "${wheels[@]}"; do
  auditwheel repair "${args[@]}" -w dist "$wheel"
done

# Drop the un-repaired linux_x86_64/linux_aarch64 wheels; only the manylinux
# ones are uploadable to PyPI.
rm -f dist/*-linux_*.whl
