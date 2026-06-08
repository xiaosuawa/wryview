"""Type stubs for wryview._core — the native WebView binding."""

from typing import Callable, Optional, Union


class CookieDict:
    """A single cookie returned by :meth:`WebView.cookies`."""

    @property
    def name(self) -> str:
        """Cookie name."""

    @property
    def value(self) -> str:
        """Cookie value."""

    @property
    def domain(self) -> Optional[str]:
        """Cookie domain, if set."""

    @property
    def path(self) -> Optional[str]:
        """Cookie path, if set."""

    @property
    def secure(self) -> bool:
        """Whether the cookie is HTTPS-only."""

    @property
    def http_only(self) -> bool:
        """Whether the cookie is inaccessible to JavaScript."""


class WebView:
    """Cross-platform webview backed by the system WebView engine.

    Created as a child of an existing native window (``parent_hwnd``).
    No event loop — your GUI framework owns the event loop.
    """

    def __init__(
            self,
            parent_hwnd: int,
            *,
            width: int = 800,
            height: int = 600,
            url: Optional[str] = None,
            html: Optional[str] = None,
            transparent: bool = False,
            background_color: Optional[tuple[int, int, int, int]] = None,
            visible: bool = True,
            devtools: bool = False,
            incognito: bool = False,
            user_agent: Optional[str] = None,
            focused: bool = True,
            autoplay: bool = False,
            javascript_enabled: bool = True,
            hotkeys_zoom: bool = True,
            initialization_script: Optional[str] = None,
            ipc_handler: Optional[Callable[[str], None]] = None,
            on_navigation: Optional[Callable[[str], bool]] = None,
            on_page_load: Optional[Callable[[str, str], None]] = None,
            on_title_changed: Optional[Callable[[str], None]] = None,
            on_new_window: Optional[Callable[[str], str]] = None,
            drag_drop_handler: Optional[Callable[[str, list[str], tuple[int, int]], bool]] = None,
            custom_protocols: Optional[
                dict[str, Callable[[str, str, list[tuple[str, str]], bytes,
                                    Callable[[int, list[tuple[str, str]], bytes], None]], None]]
            ] = None,
            proxy: Optional[dict[str, str]] = None,
            back_forward_gestures: bool = False,
            clipboard: bool = True,
            data_directory: Optional[str] = None,
            headers: Optional[Union[dict[str, str], list[tuple[str, str]]]] = None,
            as_child: bool = True,
    ) -> None:
        """Create a WebView.

        Args:
            parent_hwnd: Native window handle to attach to.
                Windows: ``HWND``.  macOS: ``NSView`` pointer.  Linux (X11): ``XID``.
            width: Initial width in logical pixels (ignored if ``as_child=False``).
            height: Initial height in logical pixels (ignored if ``as_child=False``).
            url: URL to load after creation.
            html: HTML string to load after creation (overrides *url*).
            transparent: Enable transparent background.  Disables the default
                white page background.
            background_color: RGBA tuple, e.g. ``(255, 255, 255, 255)``.
            visible: Whether the webview is initially visible.
            devtools: Enable browser DevTools (F12 / right-click → Inspect).
            incognito: Incognito / private mode — no persistent storage.
            user_agent: Custom User-Agent string.
            focused: Whether the webview should receive keyboard focus.
            autoplay: Allow media autoplay.
            javascript_enabled: Enable JavaScript execution.  Default ``True``.
            hotkeys_zoom: Enable ``Ctrl +`` / ``Ctrl -`` zoom shortcuts.
            initialization_script: JavaScript injected on every page load
                (before any page scripts run).
            ipc_handler: Callable ``(message: str)`` — receives messages from
                ``window.ipc.postMessage()`` on the JS side.
            on_navigation: Callable ``(url: str) -> bool`` — return ``False`` to
                block navigation.
            on_page_load: Callable ``(event: str, url: str)`` — *event* is
                ``"Started"`` or ``"Finished"``.
            on_title_changed: Callable ``(title: str)`` — fires when
                ``document.title`` changes.
            on_new_window: Callable ``(url: str) -> str`` — return ``"allow"`` or
                ``"deny"``.
            drag_drop_handler: Callable ``(event_type, paths, position) -> bool``.
                *event_type* is one of ``"Enter"``, ``"Over"``, ``"Drop"``, ``"Leave"``.
            custom_protocols: Dict mapping scheme names to async handlers.
                Handler signature: ``(method, uri, headers, body, respond)``.
                Call ``respond(status, headers, body)`` to reply (any thread OK).
            proxy: Dict with ``"type"`` (``"http"`` or ``"socks5"``), ``"host"``,
                and ``"port"``.
            back_forward_gestures: Enable swipe-back/forward (macOS only).
            clipboard: Enable clipboard access.  Default ``True``.
            data_directory: Path for persistent user data (cache, cookies, etc.).
                Creates the directory if it doesn't exist.
            headers: Custom HTTP headers sent with every request.  Accepts
                ``dict[str, str]`` or ``list[tuple[str, str]]`` (latter preserves
                duplicate header names).
            as_child: If ``True`` (default), creates the WebView as a child window
                inside *parent_hwnd* — you manage size via :meth:`set_bounds`.
                If ``False``, the WebView **fills** *parent_hwnd* and auto-resizes
                when the parent resizes — no :meth:`set_bounds` needed.
        """

    # ── Content ────────────────────────────────────────────────────────────

    def load_url(self, url: str) -> None:
        """Navigate to *url*."""

    def load_url_with_headers(self, url: str, headers: Union[dict[str, str], list[tuple[str, str]]]) -> None:
        """Navigate to *url* with custom HTTP headers."""

    def load_html(self, html: str) -> None:
        """Load *html* string directly (no network request)."""

    def reload(self) -> None:
        """Reload the current page."""

    def url(self) -> Optional[str]:
        """Return the current URL, or ``None``."""

    # ── JavaScript ─────────────────────────────────────────────────────────

    def eval_js(self, script: str) -> None:
        """Execute JavaScript.  Return value is discarded; use
        :meth:`eval_js_with_callback` if you need the result."""

    def eval_js_with_callback(self, script: str, callback: Callable[[str], None]) -> None:
        """Execute JavaScript and pass the JSON-serialised result to *callback*."""

    # ── IPC ────────────────────────────────────────────────────────────────

    def set_ipc_handler(self, handler: Callable[[str], None]) -> None:
        """Set the IPC message handler (JS: ``window.ipc.postMessage(msg)``)."""

    def clear_ipc_handler(self) -> None:
        """Remove the IPC message handler."""

    # ── Callback setters ───────────────────────────────────────────────────

    def set_on_navigation(self, handler: Callable[[str], bool]) -> None:
        """Set navigation handler.  Return ``False`` to block."""

    def set_on_page_load(self, handler: Callable[[str, str], None]) -> None:
        """Set page-load handler.  Receives ``(event, url)``."""

    def set_on_title_changed(self, handler: Callable[[str], None]) -> None:
        """Set title-changed handler."""

    def set_on_new_window(self, handler: Callable[[str], str]) -> None:
        """Set new-window handler.  Return ``"allow"`` or ``"deny"``."""

    def set_drag_drop_handler(self, handler: Callable[[str, list[str], tuple[int, int]], bool]) -> None:
        """Set drag-drop handler.  Receives ``(event_type, paths, position)``."""

    # ── Reparent ───────────────────────────────────────────────────────────

    def reparent(self, new_parent: int) -> None:
        """Re-attach the webview to a different parent window.

        Windows: *new_parent* must be a valid ``HWND``.
        macOS: *new_parent* **must** be an ``NSWindow`` pointer, not an
        ``NSView``.  Passing an NSView will crash.  You almost never need
        this on macOS — native views aren't destroyed on hide/show.
        Linux: no-op — needs a GTK container, not an XID.  X11 doesn't
        destroy windows on hide/show either.
        """

    # ── Geometry / Visibility ──────────────────────────────────────────────

    def set_bounds(self, x: float, y: float, width: float, height: float) -> None:
        """Move and resize the webview relative to its parent.
        Ignored when ``as_child=False`` — the webview fills the parent."""

    def bounds(self) -> Optional[tuple[float, float, float, float]]:
        """Return ``(x, y, width, height)`` in logical pixels, or ``None``."""

    def set_visible(self, visible: bool) -> None:
        """Show or hide the webview."""

    def set_background_color(self, r: int, g: int, b: int, a: int) -> None:
        """Set background colour after creation.  RGBA, each 0-255."""

    def focus(self) -> None:
        """Move keyboard focus to the webview."""

    # ── Zoom ───────────────────────────────────────────────────────────────

    def zoom(self, scale: float) -> None:
        """Set zoom level.  ``1.0`` = 100%, ``1.5`` = 150%."""

    # ── DevTools ───────────────────────────────────────────────────────────

    def open_devtools(self) -> None:
        """Open the browser DevTools window."""

    def close_devtools(self) -> None:
        """Close the browser DevTools window (macOS / Linux only)."""

    def is_devtools_open(self) -> bool:
        """Return whether DevTools is currently open (macOS / Linux only)."""

    # ── Cookies ────────────────────────────────────────────────────────────

    def cookies(self) -> list[CookieDict]:
        """Return all cookies as a list of :class:`CookieDict`."""

    def cookies_for_url(self, url: str) -> list[CookieDict]:
        """Return cookies visible to *url*."""

    def set_cookie(self, name: str, value: str, domain: Optional[str] = None, path: Optional[str] = None) -> None:
        """Set a cookie.  *domain* and *path* are optional."""

    def delete_cookie(self, name: str, url: str) -> None:
        """Delete a cookie by name, scoped to *url*."""

    # ── Misc ───────────────────────────────────────────────────────────────

    def print(self) -> None:
        """Open the system print dialog for the current page."""

    def clear_all_browsing_data(self) -> None:
        """Clear all browsing data (cache, cookies, storage)."""
