"""
wryview — Minimal wry webview binding for Python.

Usage:
    from wryview import WebView, CookieDict, WebContext

    # Embed as child window (default)
    wv = WebView(int(widget.winId()))

    # Fill an independent window (wry manages size automatically)
    wv = WebView(int(anchor.winId()), as_child=False)
    wv.load_url("https://example.com")
    wv.eval_js("document.body.style.background = 'red'")

    # JS → Python messages
    def on_message(msg: str):
        print(f"JS says: {msg}")

    wv.set_ipc_handler(on_message)

    # JS side:  window.ipc.postMessage('hello from JS')

    # Cookies
    cookies = wv.cookies()
    for c in cookies:
        print(f"{c.name} = {c.value}")

    # Shared WebContext (cookies/cache shared across WebViews)
    ctx = WebContext(data_directory="/path/to/profile")
    wv1 = WebView(hwnd1, web_context=ctx)
    wv2 = WebView(hwnd2, web_context=ctx)  # shares cookies/cache
"""

from wryview._core import (
    WebView,
    WebContext,
    CookieDict,
    PageLoadEvent,
    NewWindowResponse,
    DragDropEvent,
    pump_events,
    ensure_gtk_init,
    WindowHandleKind,
)

__version__ = "0.5.2"
__all__ = [
    "WebView",
    "WebContext",
    "CookieDict",
    "PageLoadEvent",
    "NewWindowResponse",
    "DragDropEvent",
    "pump_events",
    "ensure_gtk_init",
    "WindowHandleKind",
]
