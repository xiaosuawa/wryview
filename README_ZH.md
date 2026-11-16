# wryview

<div align="center">
<img src="https://img.shields.io/pypi/v/wryview" alt="PyPI version">
<img src="https://img.shields.io/pypi/l/wryview" alt="License">
<img src="https://img.shields.io/pypi/pyversions/wryview" alt="Python versions">
</div>

| [English](README.md) | 简体中文 |

---

## 状态：Alpha

wryview 是一个小而专注的 [wry](https://github.com/tauri-apps/wry) Python 绑定。
它最初是 [QtWebView](https://github.com/xiaosuawa/QtWebView) 的 WebView 层，
现在是独立组件，同时被 QtWebView 和
[LumiView](https://github.com/xiaosuawa/LumiView) 使用。Python API 仍处于 0.x：
minor 版本可能发生变动。欢迎反馈和贡献。

---

## 📖 简介

wryview 把 wry 的 WebView 嵌入到你自己的窗口里。它不创建顶层窗口，也不运行事件循环——
GUI 框架保留自己的循环，只需要传入原生父句柄。`as_child=False` 时由 wry 让
WebView 跟随宿主窗口大小；默认的子控件模式下，边界由你管理。

基于 [PyO3](https://pyo3.rs) 构建，以预编译 wheel 分发，wheel 用户无需 Rust 工具链。

## 💡 想要完整的桌面应用框架？

wryview 是**组件**而非框架：它只提供 webview，窗口、事件循环和 UI 脚手架都需要你自己搭建。需要把 webview 嵌入已有的 Qt/tkinter/原生窗口？wryview 正是为此而生。

从零开始做一个桌面应用？试试 **[LumiView](https://github.com/xiaosuawa/LumiView)** —— 基于 wryview 构建的 Pythonic 桌面应用框架。它开箱即用地提供原生窗口、事件循环、JS Bridge（IPC）、自定义协议和 async-first API。LumiView 以 wryview 作为 WebView 引擎，渲染核心相同，但省去了所有管道工作。

## ✨ 特性

- **不自带事件循环** — 循环归你的框架所有。Linux 上如果宿主循环不是 GTK
  （例如 Qt 的 xcb 后端），需要周期性调用 `pump_events()`，并在创建第一个
  WebView 之前调用一次 `ensure_gtk_init()`（两者都由 `wryview` 导出）。
- **广泛的 wry 覆盖** — 导航、JS 执行、IPC、自定义协议、Cookie、拖放、代理、
  DevTools、缩放、下载、主题（Windows）、共享 `WebContext` 等。wry 的平台专属
  builder 扩展（macOS `accept_first_mouse`、Windows `browser_accelerator_keys`、
  Linux `extension_path`）尚未封装。
- **跨平台** — Windows（WebView2）、macOS（WKWebView）、Linux（WebKitGTK 4.1）。
  Python API 一致；平台差异见下文。
- **框架无关** — 可嵌入 Qt、tkinter/wxPython（取决于平台），或任何原生窗口
  （HWND / NSView / X11 XID）。Wayland 下需要传 GTK container 并设置
  `parent_hwnd_kind=WindowHandleKind.Gtk`。
- **预编译 Wheel** — `pip install wryview`，支持 CPython 3.10–3.15，包括
  3.14 自由线程（`cp314t`）与 3.15+ 自由线程（`abi3.abi3t` wheel）。

## ⬇️ 安装

```bash
pip install wryview
```

**Windows：** 需要 WebView2 Runtime（Windows 11 和较新的 Windows 10 通常已内置，
或[下载](https://developer.microsoft.com/microsoft-edge/webview2/)）。
**macOS：** WKWebView 系统内置。
**Linux：** 需要 WebKitGTK 4.1 和 GTK 3 运行时库
（Debian/Ubuntu：`libwebkit2gtk-4.1-0`、`libgtk-3-0`；Fedora：
`webkit2gtk4.1`、`gtk3`）。`-dev` 包只在从源码构建时需要。

**Python：** 3.10+；3.14 自由线程有独立的 `cp314t` wheel，3.15+ 使用
`abi3.abi3t` wheel，GIL 与自由线程构建通用。从源码构建需要 Rust 1.85+（PyO3 0.30）。

## 🧑‍💻 使用示例

```python
from wryview import WebView, WindowHandleKind

# 嵌入任意原生窗口作为子控件（默认模式）
wv = WebView(parent_hwnd, url="https://example.com", devtools=True)

# 填充独立窗口——由 wry 让 webview 跟随宿主窗口大小
wv = WebView(anchor_hwnd, url="https://example.com", as_child=False)

# Wayland：父句柄是 GTK container，不是 X11 XID
# wv = WebView(gtk_container_ptr, parent_hwnd_kind=WindowHandleKind.Gtk, url=...)

# JS 执行
wv.eval_js("document.body.style.background = '#333'")

# JS → Python IPC（JS 侧：window.ipc.postMessage('hello')）
wv.set_ipc_handler(lambda msg: print(f"JS 发来: {msg}"))

# 页面生命周期
wv.set_on_page_load(lambda event, url: print(f"{event}: {url}"))

# 自定义协议：请求/响应头都是 (name, value) 元组列表
def my_handler(method, uri, headers, body, respond):
    # respond 只能调用一次，可以从任意线程调用
    respond(200, [("Content-Type", "text/html")], b"<h1>来自 Python 的问候！</h1>")

wv = WebView(parent_hwnd, custom_protocols={"myapp": my_handler}, url="myapp://localhost/")
```

> **Linux 嵌入说明：** WebKitGTK 在 GLib 主上下文上渲染。如果宿主循环不是 GTK，
> 需要周期性调用 `pump_events()`（例如每 16ms 一次）。在创建第一个
> WebView 之前调用一次 `ensure_gtk_init()`。这两个辅助函数都由 `wryview` 导出。

## 📦 API 概览

| 类别 | 方法 |
|------|------|
| 内容 | `load_url`, `load_url_with_headers`, `load_html`, `reload`, `url` |
| JavaScript | `eval_js`, `eval_js_with_callback` |
| IPC | `set_ipc_handler`, `clear_ipc_handler` |
| 回调 | `set_on_navigation`, `set_on_page_load`, `set_on_title_changed`, `set_on_new_window`, `set_drag_drop_handler`, `set_on_download_started`, `set_on_download_completed` |
| 几何 | `set_bounds`, `bounds`, `reparent` |
| 外观 | `set_visible`, `set_background_color`, `set_theme`（Windows）, `focus` |
| DevTools | `open_devtools`, `close_devtools`, `is_devtools_open` |
| Cookie | `cookies`, `cookies_for_url`, `set_cookie`, `delete_cookie` |
| 生命周期 | `close` |
| 其他 | `zoom`, `print`, `clear_all_browsing_data` |
| Linux 辅助 | `pump_events`, `ensure_gtk_init` |

构造参数：`width`、`height`、`url`、`html`、`transparent`、`background_color`、
`visible`、`devtools`、`incognito`、`user_agent`、`focused`、`autoplay`、
`javascript_enabled`、`hotkeys_zoom`、`initialization_script`、`ipc_handler`、
`on_navigation`、`on_page_load`、`on_title_changed`、`on_new_window`、
`drag_drop_handler`、`custom_protocols`、`proxy`、`back_forward_gestures`、
`clipboard`、`data_directory`、`web_context`、`headers`、`https_scheme`、
`default_context_menus`、`theme`（Windows）、`on_download_started`、
`on_download_completed`、`as_child`、`parent_hwnd_kind`。

`WebContext` 可以在多个 WebView 之间共享，以共享 Cookie、缓存和存储。
完整签名见 `python/wryview/_core.pyi`。

## 🧩 使用案例

- **[LumiView](https://github.com/xiaosuawa/LumiView)** — 基于 wryview 的 Pythonic 桌面应用框架（原生窗口、事件循环、JS Bridge、自定义协议）。**在 wryview 之上构建完整桌面应用的首选方式。**
- **[QtWebView](https://github.com/xiaosuawa/QtWebView)** — 基于 wryview 的跨平台 Qt webview 组件（PySide/PyQt）。将 wry WebView 作为原生子窗口嵌入 Qt widget，提供 JS Bridge 和 WSGI 支持。

## 🤝 贡献

Rust 层刻意保持很小（`src/lib.rs`），Python 包在 `python/wryview`。欢迎代码、
文档和想法方面的贡献。

## License

Copyright (c) 2026 Xiaosu.

Distributed under the terms of the [Mozilla Public License Version 2.0](https://github.com/xiaosuawa/wryview/blob/main/LICENSE).
