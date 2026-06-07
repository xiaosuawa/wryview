# wryview

<div align="center">
<img src="https://img.shields.io/pypi/v/wryview" alt="PyPI version">
<img src="https://img.shields.io/pypi/l/wryview" alt="License">
<img src="https://img.shields.io/pypi/pyversions/wryview" alt="Python versions">
</div>

| English | [简体中文](README_ZH.md) |

---

## ⚠️ Status: Pre-Alpha

wryview is in early development. The API may change without notice between versions. It was created to serve [QtWebView](https://github.com/xiaosuawa/QtWebView), a Qt webview widget project, and the author is a Rust beginner — the binding code may not be idiomatic. Feedback, bug reports, and contributions (code or ideas) are very welcome.

---

## 📖 Introduction

wryview is a Python binding for [wry](https://github.com/tauri-apps/wry), the cross-platform WebView rendering library used by [Tauri](https://tauri.app). It wraps the full wry API — no event loop, no window management, just the webview.

Built on [PyO3](https://pyo3.rs) and distributed as pre-compiled wheels (no Rust toolchain required for end users).

## ✨ Features

- **Zero Event Loop** — wryview creates the webview as a child of *your* window. Your GUI framework owns the event loop. No conflicts, no deadlocks.
- **Full wry API** — Navigation, JavaScript execution, IPC (`window.ipc.postMessage`), custom protocols, cookies, drag & drop, proxy, devtools, zoom, and more.
- **Cross-Platform** — Windows (WebView2), macOS (WKWebView), Linux (WebKitGTK). Same API everywhere.
- **Framework-Agnostic** — Embed into Qt, tkinter, wxPython, or any native window via HWND/NSView/XID.
- **Pre-Built Wheels** — `pip install wryview` just works. No Rust, no cmake, no system deps (beyond the OS webview).

## ⬇️ Installation

```bash
pip install wryview
```

**Windows:** WebView2 Runtime is required (pre-installed on Windows 10 1803+, or [download](https://developer.microsoft.com/microsoft-edge/webview2/)).  
**macOS:** WKWebView is built-in.  
**Linux:** Requires `libwebkit2gtk-4.1-dev` and related packages (see [wry docs](https://github.com/tauri-apps/wry#readme)).

## 🧑‍💻 Usage

```python
from wryview import WebView

# Create a webview as a child of any native window
wv = WebView(parent_hwnd, url="https://example.com", devtools=True)

# JS execution
wv.eval_js("document.body.style.background = '#333'")

# JS → Python IPC (JS side: window.ipc.postMessage('hello'))
wv.set_ipc_handler(lambda msg: print(f"JS says: {msg}"))

# Navigation callback
wv.set_on_page_load(lambda event, url: print(f"{event}: {url}"))

# Custom protocol (intercept requests asynchronously)
def my_handler(method, uri, headers, body, respond):
    # respond(status, headers, body) must be called from any thread
    respond(200, {"Content-Type": "text/html"}, b"<h1>Hello from Python!</h1>")

wv = WebView(hwnd, custom_protocols={"myapp": my_handler}, url="myapp://localhost/")
```

## 📦 API Summary

| Category | Methods |
|----------|---------|
| Content | `load_url`, `load_html`, `load_url_with_headers`, `reload`, `url` |
| JavaScript | `eval_js`, `eval_js_with_callback` |
| IPC | `set_ipc_handler`, `clear_ipc_handler` |
| Callbacks | `set_on_navigation`, `set_on_page_load`, `set_on_title_changed`, `set_on_new_window`, `set_drag_drop_handler` |
| Geometry | `set_bounds`, `bounds` |
| Appearance | `set_visible`, `set_background_color`, `focus` |
| DevTools | `open_devtools`, `close_devtools`, `is_devtools_open` |
| Cookies | `cookies`, `cookies_for_url`, `set_cookie`, `delete_cookie` |
| Other | `zoom`, `print`, `clear_all_browsing_data` |

Full constructor options: `transparent`, `background_color`, `incognito`, `user_agent`, `autoplay`, `initialization_script`, `proxy`, `back_forward_gestures`, `clipboard`, and more.

## 🧩 Used By

- **[QtWebView](https://github.com/xiaosuawa/QtWebView)** — A cross-platform webview widget for Qt (PySide/PyQt), powered by wryview. Embeds a wry WebView as a native child window inside any Qt widget, with a seamless JS bridge and WSGI support.

## 🤝 Contributing

The author is a Rust beginner — if you see something that could be improved, please open an issue or PR. All contributions (code, ideas, documentation) are welcome.

## License

Copyright (c) 2026 Xiaosu.

Distributed under the terms of the [Mozilla Public License Version 2.0](https://github.com/xiaosuawa/wryview/blob/main/LICENSE).
