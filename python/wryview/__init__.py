"""
wryview — Minimal wry webview binding for Python.

Usage:
    from wryview import WebView

    # Create webview as child of a Qt widget's HWND
    wv = WebView(int(widget.winId()))
    wv.load_url("https://example.com")
    wv.eval_js("document.body.style.background = 'red'")

    # JS → Python messages
    def on_message(msg: str):
        print(f"JS says: {msg}")

    wv.set_ipc_handler(on_message)

    # JS side:  window.ipc.postMessage('hello from JS')
"""

from wryview._core import WebView

__all__ = ["WebView"]
