# wryview

<div align="center">
<img src="https://img.shields.io/pypi/v/wryview" alt="PyPI version">
<img src="https://img.shields.io/pypi/l/wryview" alt="License">
<img src="https://img.shields.io/pypi/pyversions/wryview" alt="Python versions">
</div>

| English | [简体中文](README_ZH.md) |

---

## Status: Alpha

wryview is a small, focused Python binding for [wry](https://github.com/tauri-apps/wry).
It started as the WebView layer of [QtWebView](https://github.com/xiaosuawa/QtWebView)
and is now a standalone component used by QtWebView and
[LumiView](https://github.com/xiaosuawa/LumiView). The Python API is still 0.x:
minor releases may change it. Bug reports and contributions are welcome.

---

## 📖 Introduction

wryview embeds wry's WebView into a window owned by your application. It does
not create a top-level window and does not run an event loop — your GUI
framework keeps its own loop and passes a native parent handle. With
`as_child=False`, wry keeps the WebView sized to the host window; in the default
child mode the WebView is a child control and you manage its bounds.

Built on [PyO3](https://pyo3.rs) and distributed as prebuilt wheels, so wheel
users do not need a Rust toolchain.

## 💡 Looking for a Full Desktop Framework?

wryview is a **component**, not a framework: it gives you the webview, but you bring your own window, event loop, and UI scaffolding. Embedding a webview into an existing Qt/tkinter/native window? wryview is for you.

Starting a new desktop app from scratch? Try **[LumiView](https://github.com/xiaosuawa/LumiView)** — a Pythonic desktop app framework built on wryview. It adds native windows, an event loop, a JS bridge (IPC), custom protocols, and an async-first API, all out of the box. wryview powers LumiView's WebView engine, so you get the same rendering core with none of the plumbing.

## ✨ Features

- **No event loop of its own** — your framework owns the loop. On Linux, when
  the host loop is not GTK (for example Qt's xcb backend), call `pump_events()`
  periodically and `ensure_gtk_init()` once before creating the first WebView
  (both are exported by `wryview`).
- **Broad wry coverage** — navigation, JavaScript evaluation, IPC, custom
  protocols, cookies, drag & drop, proxy, DevTools, zoom, downloads, themes
  (Windows), shared `WebContext`, and more. Platform-specific wry builder
  extensions (macOS `accept_first_mouse`, Windows `browser_accelerator_keys`,
  Linux `extension_path`) are not wrapped yet.
- **Cross-platform** — Windows (WebView2), macOS (WKWebView), Linux
  (WebKitGTK 4.1). The Python API is the same; platform-specific details are
  noted below.
- **Framework-agnostic** — embed into Qt, tkinter/wxPython (platform-dependent),
  or any native window via HWND / NSView / X11 XID. On Wayland, pass a GTK
  container and `parent_hwnd_kind=WindowHandleKind.Gtk`.
- **Prebuilt wheels** — `pip install wryview` for CPython 3.10–3.15, including
  free-threaded 3.15 (`abi3.abi3t` wheels).

## ⬇️ Installation

```bash
pip install wryview
```

**Windows:** WebView2 Runtime is required (often preinstalled on Windows 11 and
recent Windows 10, or [download](https://developer.microsoft.com/microsoft-edge/webview2/)).
**macOS:** WKWebView is built in.
**Linux:** WebKitGTK 4.1 and GTK 3 runtime libraries are required
(Debian/Ubuntu: `libwebkit2gtk-4.1-0`, `libgtk-3-0`; Fedora:
`webkit2gtk4.1`, `gtk3`). The `-dev` packages are only needed when building
from source.

**Python:** 3.10+; the 3.15 wheels support both the GIL and free-threaded
builds. Free-threaded 3.14 has no wheels — the `abi3t` stable ABI starts at
3.15. Building from source requires Rust 1.85+ (PyO3 0.30).

## 🧑‍💻 Usage

```python
from wryview import WebView, WindowHandleKind

# Embed as a child of a native window (default)
wv = WebView(parent_hwnd, url="https://example.com", devtools=True)

# Fill an independent window — wry keeps the webview sized to the host window
wv = WebView(anchor_hwnd, url="https://example.com", as_child=False)

# Wayland: the parent is a GTK container, not an X11 XID
# wv = WebView(gtk_container_ptr, parent_hwnd_kind=WindowHandleKind.Gtk, url=...)

# JS execution
wv.eval_js("document.body.style.background = '#333'")

# JS → Python IPC (JS side: window.ipc.postMessage('hello'))
wv.set_ipc_handler(lambda msg: print(f"JS says: {msg}"))

# Page lifecycle
wv.set_on_page_load(lambda event, url: print(f"{event}: {url}"))

# Custom protocol: request/response headers are lists of (name, value) pairs
def my_handler(method, uri, headers, body, respond):
    # Call respond exactly once. It may be called from any thread.
    respond(200, [("Content-Type", "text/html")], b"<h1>Hello from Python!</h1>")

wv = WebView(parent_hwnd, custom_protocols={"myapp": my_handler}, url="myapp://localhost/")
```

> **Linux embedding note:** WebKitGTK renders on the GLib main context. If the
> host loop is not GTK, call `pump_events()` periodically (for example from a
> 16 ms timer). Call `ensure_gtk_init()` once before creating the first
> WebView. Both helpers are exported by `wryview`.

## 📦 API Summary

| Category | Methods |
|----------|---------|
| Content | `load_url`, `load_url_with_headers`, `load_html`, `reload`, `url` |
| JavaScript | `eval_js`, `eval_js_with_callback` |
| IPC | `set_ipc_handler`, `clear_ipc_handler` |
| Callbacks | `set_on_navigation`, `set_on_page_load`, `set_on_title_changed`, `set_on_new_window`, `set_drag_drop_handler`, `set_on_download_started`, `set_on_download_completed` |
| Geometry | `set_bounds`, `bounds`, `reparent` |
| Appearance | `set_visible`, `set_background_color`, `set_theme` (Windows), `focus` |
| DevTools | `open_devtools`, `close_devtools`, `is_devtools_open` |
| Cookies | `cookies`, `cookies_for_url`, `set_cookie`, `delete_cookie` |
| Lifecycle | `close` |
| Other | `zoom`, `print`, `clear_all_browsing_data` |
| Linux helpers | `pump_events`, `ensure_gtk_init` |

Constructor options: `width`, `height`, `url`, `html`, `transparent`,
`background_color`, `visible`, `devtools`, `incognito`, `user_agent`,
`focused`, `autoplay`, `javascript_enabled`, `hotkeys_zoom`,
`initialization_script`, `ipc_handler`, `on_navigation`, `on_page_load`,
`on_title_changed`, `on_new_window`, `drag_drop_handler`, `custom_protocols`,
`proxy`, `back_forward_gestures`, `clipboard`, `data_directory`,
`web_context`, `headers`, `https_scheme`, `default_context_menus`, `theme`
(Windows), `on_download_started`, `on_download_completed`, `as_child`,
`parent_hwnd_kind`.

`WebContext` can be shared across WebViews to share cookies, cache and
storage. See `python/wryview/_core.pyi` for the complete signatures.

## 🧩 Used By

- **[LumiView](https://github.com/xiaosuawa/LumiView)** — A Pythonic desktop app framework (native windows, event loop, JS bridge, custom protocols) with wryview as its WebView engine. **The recommended way to build a complete desktop app on wryview.**
- **[QtWebView](https://github.com/xiaosuawa/QtWebView)** — A cross-platform webview widget for Qt (PySide/PyQt), powered by wryview. Embeds a wry WebView as a native child window inside any Qt widget, with a seamless JS bridge and WSGI support.

## 🤝 Contributing

The Rust layer is intentionally small (`src/lib.rs`); the Python package lives
in `python/wryview`. Contributions of code, documentation, and ideas are
welcome.

## License

Copyright (c) 2026 Xiaosu.

Distributed under the terms of the [Mozilla Public License Version 2.0](https://github.com/xiaosuawa/wryview/blob/main/LICENSE).
