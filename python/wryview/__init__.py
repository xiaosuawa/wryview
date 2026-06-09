"""
wryview — Minimal wry webview binding for Python.

Usage:
    from wryview import WebView, CookieDict

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
"""

from wryview._core import WebView, CookieDict, pump_events, ensure_gtk_init

__version__ = "0.3.2"
__all__ = ["WebView", "CookieDict", "pump_events", "ensure_gtk_init"]
