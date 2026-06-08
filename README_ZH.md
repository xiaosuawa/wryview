# wryview

<div align="center">
<img src="https://img.shields.io/pypi/v/wryview" alt="PyPI version">
<img src="https://img.shields.io/pypi/l/wryview" alt="License">
<img src="https://img.shields.io/pypi/pyversions/wryview" alt="Python versions">
</div>

| [English](README.md) | 简体中文 |

---

## ⚠️ 早期开发阶段

wryview 处于早期开发阶段，API 可能在不同版本间发生变动。本项目是为 [QtWebView](https://github.com/xiaosuawa/QtWebView) 服务的，作者是 Rust 新手——绑定代码可能不够完善。欢迎反馈、报告 Bug、贡献代码或想法。

---

## 📖 简介

wryview 是 [wry](https://github.com/tauri-apps/wry)（[Tauri](https://tauri.app) 使用的跨平台 WebView 渲染库）的 Python 绑定。封装了完整的 wry API——无事件循环、无窗口管理，纯粹的 webview 组件。

基于 [PyO3](https://pyo3.rs) 构建，以预编译 wheel 分发（终端用户无需 Rust 工具链）。

## ✨ 特性

- **零事件循环** — webview 作为你的窗口的子控件创建。GUI 框架拥有事件循环，无冲突、无死锁。
- **完整 wry API** — 导航、JS 执行、IPC（`window.ipc.postMessage`）、自定义协议、Cookie、拖放、代理、DevTools、缩放等。
- **跨平台** — Windows（WebView2）、macOS（WKWebView）、Linux（WebKitGTK），统一 API。
- **框架无关** — 可嵌入 Qt、tkinter、wxPython 或任何原生窗口（通过 HWND/NSView/XID）。
- **预编译 Wheel** — `pip install wryview` 即可，无需 Rust、cmake、系统依赖（除操作系统自带 webview 外）。

## ⬇️ 安装

```bash
pip install wryview
```

**Windows:** 需要 WebView2 Runtime（Windows 10 1803+ 已内置，或[下载](https://developer.microsoft.com/microsoft-edge/webview2/)）。  
**macOS:** WKWebView 系统内置。  
**Linux:** 需要 `libwebkit2gtk-4.1-dev` 等包（详见 [wry 文档](https://github.com/tauri-apps/wry#readme)）。

## 🧑‍💻 使用示例

```python
from wryview import WebView

# 嵌入任意原生窗口作为子控件（默认模式）
wv = WebView(parent_hwnd, url="https://example.com", devtools=True)

# 填充独立窗口 — wry 通过 WM_SIZE / layout 自动管理大小
wv = WebView(anchor_hwnd, url="https://example.com", as_child=False)

# JS 执行
wv.eval_js("document.body.style.background = '#333'")

# JS → Python IPC（JS 侧: window.ipc.postMessage('hello')）
wv.set_ipc_handler(lambda msg: print(f"JS 发来: {msg}"))

# 页面生命周期回调
wv.set_on_page_load(lambda event, url: print(f"{event}: {url}"))

# 自定义协议（异步拦截请求）
def my_handler(method, uri, headers, body, respond):
    # respond(status, headers, body) 可在任意线程中调用
    respond(200, {"Content-Type": "text/html"}, b"<h1>来自 Python 的问候！</h1>")

wv = WebView(hwnd, custom_protocols={"myapp": my_handler}, url="myapp://localhost/")
```

## 📦 API 概览

| 类别 | 方法 |
|------|------|
| 内容 | `load_url`, `load_html`, `load_url_with_headers`, `reload`, `url` |
| JavaScript | `eval_js`, `eval_js_with_callback` |
| IPC | `set_ipc_handler`, `clear_ipc_handler` |
| 回调 | `set_on_navigation`, `set_on_page_load`, `set_on_title_changed`, `set_on_new_window`, `set_drag_drop_handler` |
| 几何 | `set_bounds`, `bounds` |
| 外观 | `set_visible`, `set_background_color`, `focus` |
| DevTools | `open_devtools`, `close_devtools`, `is_devtools_open` |
| Cookie | `cookies`, `cookies_for_url`, `set_cookie`, `delete_cookie` |
| 其他 | `zoom`, `print`, `clear_all_browsing_data` |

构造参数还包括：`transparent`、`background_color`、`incognito`、`user_agent`、`autoplay`、`initialization_script`、`proxy`、`back_forward_gestures`、`clipboard` 等。

## 🧩 使用案例

- **[QtWebView](https://github.com/xiaosuawa/QtWebView)** — 基于 wryview 的跨平台 Qt webview 组件（PySide/PyQt）。将 wry WebView 作为原生子窗口嵌入 Qt widget，提供 JS Bridge 和 WSGI 支持。

## 🤝 贡献

作者是 Rust 新手——如果你发现任何可以改进的地方，欢迎提 Issue 或 PR。所有形式的贡献（代码、想法、文档）都非常欢迎。

## License

Copyright (c) 2026 Xiaosu.

Distributed under the terms of the [Mozilla Public License Version 2.0](https://github.com/xiaosuawa/wryview/blob/main/LICENSE).
