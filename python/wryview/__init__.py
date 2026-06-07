"""
wryview — Minimal wry webview binding for Python.

Usage:
    from wryview import WebView, CookieDict

    # Create webview as child of a Qt widget's HWND
    wv = WebView(int(widget.winId()))
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

from wryview._core import WebView, CookieDict

__version__ = "0.2.0"
__all__ = ["WebView", "CookieDict"]
