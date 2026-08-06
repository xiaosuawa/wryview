"""Type stubs for wryview._core — the native WebView binding."""

from typing import Callable, Optional, Union
from enum import Enum

class WindowHandleKind(Enum):
    """Native window handle type passed via ``parent_hwnd``."""

    Win32 = ...
    """Windows HWND (default on Windows)"""
    AppKit = ...
    """macOS NSView pointer (default on macOS)"""
    X11 = ...
    """Linux X11 XID (default on Linux)"""
    Gtk = ...
    """Linux GTK container pointer (works on both X11 and Wayland)"""

class PageLoadEvent(Enum):
    Started = ...
    """Indicates that the content of the page has started loading"""
    Finished = ...
    """Indicates that the page content has finished loading"""

class NewWindowResponse(Enum):
    Allow = ...
    """Allow the window to be opened with the default implementation"""
    Deny = ...
    """Deny the window from being opened"""

class DragDropEvent(Enum):
    Enter = ...
    """A drag operation has entered the webview"""
    Over = ...
    """A drag operation is moving over the window"""
    Drop = ...
    """The file(s) have been dropped onto the window"""
    Leave = ...
    """The drag operation has been cancelled or left the window"""
    Unknown = ...
    """Catch-all for future event types (``DragDropEvent`` is non-exhaustive)"""

class Theme(Enum):
    Auto = ...
    """Follow the system / OS preference (default)"""
    Dark = ...
    """Force dark mode"""
    Light = ...
    """Force light mode"""

class WebContext:
    """Sharable browser context for persistent state (cookies, cache, storage).

    Create once and pass to multiple :class:`WebView` instances to share
    browsing data — cookies set in one WebView are visible in another.

    Usage::

        ctx = wryview.WebContext(data_directory="/path/to/profile")
        wv1 = WebView(hwnd1, web_context=ctx, ...)
        wv2 = WebView(hwnd2, web_context=ctx, ...)  # shares cookies/cache
    """

    def __init__(self, data_directory: Optional[str] = None) -> None:
        """Create a sharable WebContext.

        Args:
            data_directory: Path for persistent user data (cache, cookies,
                localStorage, etc.).  Creates the directory if it does not
                exist.  Pass ``None`` for an ephemeral (in-memory) context.
        """

    @property
    def data_directory(self) -> Optional[str]:
        """Return the data directory path, or ``None`` for ephemeral."""

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
        ipc_handler: Optional[Callable[[str], object]] = None,
        on_navigation: Optional[Callable[[str], bool]] = None,
        on_page_load: Optional[Callable[[PageLoadEvent, str], object]] = None,
        on_title_changed: Optional[Callable[[str], object]] = None,
        on_new_window: Optional[Callable[[str], Union[NewWindowResponse, str]]] = None,
        drag_drop_handler: Optional[
            Callable[[DragDropEvent, list[str], tuple[int, int]], bool]
        ] = None,
        custom_protocols: Optional[
            dict[
                str,
                Callable[
                    [
                        str,
                        str,
                        list[tuple[str, str]],
                        bytes,
                        Callable[[int, list[tuple[str, str]], bytes], None],
                    ],
                    object,
                ],
            ]
        ] = None,
        proxy: Optional[dict[str, str]] = None,
        back_forward_gestures: bool = False,
        clipboard: bool = True,
        data_directory: Optional[str] = None,
        web_context: Optional[WebContext] = None,
        headers: Optional[Union[dict[str, str], list[tuple[str, str]]]] = None,
        https_scheme: bool = False,
        default_context_menus: bool = True,
        theme: Optional[Theme] = None,
        on_download_started: Optional[Callable[[str, str], Union[bool, str]]] = None,
        on_download_completed: Optional[
            Callable[[str, Optional[str], bool], object]
        ] = None,
        as_child: bool = True,
        parent_hwnd_kind: Optional[WindowHandleKind] = None,
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
            devtools: Enable browser DevTools (right-click → Inspect, F12).
            incognito: Incognito / private mode — no persistent storage.
                Windows: requires WebView2 Runtime ≥ 101.0.1210.39.
            user_agent: Custom User-Agent string.
            focused: Whether the webview should receive keyboard focus.
                Windows/Linux only; macOS unsupported.
            autoplay: Allow media autoplay without user interaction.
            javascript_enabled: Enable JavaScript execution.  Default ``True``.
            hotkeys_zoom: Enable ``Ctrl +`` / ``Ctrl -`` zoom shortcuts.
                Windows only.
            initialization_script: JavaScript injected on every page load
                (before any page scripts run).
            ipc_handler: Callable ``(message: str)`` — receives messages from
                ``window.ipc.postMessage()`` on the JS side.
            on_navigation: Callable ``(url: str) -> bool`` — return ``False`` to
                block navigation.
            on_page_load: Callable ``(event: PageLoadEvent, url: str)``.
            on_title_changed: Callable ``(title: str)`` — fires when
                ``document.title`` changes.
            on_new_window: Callable ``(url: str) -> NewWindowResponse``.
            drag_drop_handler: Callable ``(event: DragDropEvent, paths, position) -> bool``.
                **Windows**: passing a handler disables WebView2's built-in
                external-file drops (the wry drop target replaces the
                WebView2 one), so this is decided at construction — see
                :meth:`set_drag_drop_handler`.
            custom_protocols: Dict mapping scheme names to async handlers.
                Handler signature: ``(method, uri, headers, body, respond)``.
                Call ``respond(status, headers, body)`` to reply (any thread OK).
            proxy: Dict with ``"type"`` (``"http"`` or ``"socks5"``), ``"host"``,
                and ``"port"``.
            back_forward_gestures: Enable two-finger swipe for back/forward
                navigation.
            clipboard: Enable clipboard access (Windows/Linux).  Default ``True``.
                macOS always enabled, regardless of this setting.
            https_scheme: Use ``https://`` instead of ``http://`` for custom
                protocol workaround (Windows only).  Makes the page a secure
                context, enabling Service Workers, Geolocation, Web Crypto, etc.
            default_context_menus: Enable native right-click context menu.
                Windows only.  Default ``True``.
            theme: Force a :class:`Theme` (``Dark`` / ``Light`` / ``Auto``).
                Windows only — affects ``prefers-color-scheme`` in the page.
                macOS / Linux have no wry theme API and always follow the
                system preference (silently ignored).
            on_download_started: Callable ``(url, suggested_path) -> bool|str``.
                Return ``False`` to cancel, or a new path string to redirect.
            on_download_completed: Callable ``(url, saved_path, success)``.
                *saved_path* is ``None`` if the download was cancelled.
            data_directory: Path for persistent user data (cache, cookies, etc.).
                Creates the directory if it doesn't exist.
                Ignored when *web_context* is provided.
            web_context: Optional :class:`WebContext` to share browsing data
                (cookies, cache, storage) across multiple :class:`WebView`
                instances.  Takes priority over *data_directory*.
            headers: Custom HTTP headers sent with every request.  Accepts
                ``dict[str, str]`` or ``list[tuple[str, str]]`` (latter preserves
                duplicate header names).
            as_child: If ``True`` (default), creates the WebView as a child window
                inside *parent_hwnd* — you manage size via :meth:`set_bounds`.
                If ``False``, the WebView **fills** *parent_hwnd* and auto-resizes
                when the parent resizes — no :meth:`set_bounds` needed.
            parent_hwnd_kind: Optional :class:`WindowHandleKind`.  Auto-detected
                per platform (Win32 / AppKit / X11).  On Linux, set to
                ``WindowHandleKind.Gtk`` to embed in a GTK container
                (recommended — works on both X11 and Wayland).
        """
    # ── Content ────────────────────────────────────────────────────────────

    def load_url(self, url: str) -> None:
        """Navigate to *url*."""

    def load_url_with_headers(
        self, url: str, headers: Union[dict[str, str], list[tuple[str, str]]]
    ) -> None:
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

    def eval_js_with_callback(
        self, script: str, callback: Callable[[str], object]
    ) -> None:
        """Execute JavaScript and pass the raw string result to *callback*.
        Use ``json.loads()`` in your callback if you need a Python dict/list."""
    # ── IPC ────────────────────────────────────────────────────────────────

    def set_ipc_handler(self, handler: Callable[[str], object]) -> None:
        """Set the IPC message handler (JS: ``window.ipc.postMessage(msg)``)."""

    def clear_ipc_handler(self) -> None:
        """Remove the IPC message handler."""
    # ── Callback setters ───────────────────────────────────────────────────

    def set_on_navigation(self, handler: Callable[[str], bool]) -> None:
        """Set navigation handler.  Return ``False`` to block."""

    def set_on_page_load(self, handler: Callable[[PageLoadEvent, str], object]) -> None:
        """Set page-load handler.  Receives ``(event: PageLoadEvent, url)``."""

    def set_on_title_changed(self, handler: Callable[[str], object]) -> None:
        """Set title-changed handler."""

    def set_on_new_window(
        self, handler: Callable[[str], Union[NewWindowResponse, str]]
    ) -> None:
        """Set new-window handler.  Return ``NewWindowResponse`` or ``"allow"/"deny"`` (backward compat)."""

    def set_drag_drop_handler(
        self, handler: Callable[[DragDropEvent, list[str], tuple[int, int]], bool]
    ) -> None:
        """Replace the drag-drop handler, or raise ``RuntimeError``.

        Only works when a ``drag_drop_handler`` was passed at construction
        — wry registers the native drop target at creation, so it cannot
        be enabled later.  On Windows, having registered it disables
        WebView2's built-in external-file drops.
        """

    def set_on_download_started(
        self, handler: Callable[[str, str], Union[bool, str]]
    ) -> None:
        """Set download-started handler.  Receives ``(url, suggested_path)``.
        Return ``False`` to cancel, a new path string to redirect."""

    def set_on_download_completed(
        self, handler: Callable[[str, Optional[str], bool], object]
    ) -> None:
        """Set download-completed handler.  Receives ``(url, saved_path, success)``.
        *saved_path* is ``None`` if cancelled."""
    # ── Reparent ───────────────────────────────────────────────────────────

    def reparent(self, new_parent: int) -> None:
        """Re-attach the webview to a different parent window.

        Windows: *new_parent* must be a valid ``HWND``.
        macOS: *new_parent* **must** be an ``NSWindow`` pointer, not an
        ``NSView``.  Passing an NSView will crash.  You almost never need
        this on macOS — native views aren't destroyed on hide/show.
        Linux: raises ``NotImplementedError`` — wry needs a GTK container,
        not a raw XID.  Use :class:`WindowHandleKind.Gtk` at construction
        time to create a GTK-embedded WebView instead of reparenting later.
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

    def set_theme(self, theme: Theme) -> None:
        """Change the webview theme at runtime.

        Windows only — affects ``prefers-color-scheme`` in the page.
        Requires WebView2 Runtime 101.0.1210.39 or higher.
        macOS / Linux: raises ``NotImplementedError`` (no wry theme API;
        the webview always follows the system preference).
        """

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

    def set_cookie(
        self,
        name: str,
        value: str,
        domain: Optional[str] = None,
        path: Optional[str] = None,
    ) -> None:
        """Set a cookie.  *domain* and *path* are optional."""

    def delete_cookie(self, name: str, url: str) -> None:
        """Delete a cookie by name, scoped to *url*."""
    # ── Lifecycle ──────────────────────────────────────────────────────────

    def close(self) -> None:
        """Explicitly destroy the native WebView.

        Drops the underlying ``wry::WebView``, running WebView2 / WKWebView
        cleanup (including window class unregistration on Windows) while the
        parent window is still alive.

        Call this before program exit to avoid harmless but noisy errors like
        "Failed to unregister class Chrome_WidgetWin_0" from deferred cleanup.
        """
    # ── Misc ───────────────────────────────────────────────────────────────

    def print(self) -> None:
        """Open the system print dialog for the current page."""

    def clear_all_browsing_data(self) -> None:
        """Clear all browsing data (cache, cookies, storage)."""

# ── Module-level functions ────────────────────────────────────────────────

def pump_events() -> None:
    """Pump pending GTK/GLib events (Linux only, no-op elsewhere)."""

def ensure_gtk_init() -> None:
    """Initialise GTK from Rust's perspective (Linux only, no-op elsewhere)."""
