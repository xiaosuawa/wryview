/// wryview — Comprehensive Python binding for wry.
///
/// Exposes wry's full API: WebView creation (child or standalone window),
/// JS evaluation, IPC handler, custom protocol, navigation callbacks,
/// cookies, devtools, and more.
///
/// No event loop, no window management — just the webview.
use pyo3::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::num::NonZero;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// ── Helper: convert wry Rect to/from simple values ─────────────────────────

fn make_rect(x: f64, y: f64, width: f64, height: f64) -> wry::Rect {
    wry::Rect {
        position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(x, y)),
        size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(width, height)),
    }
}

fn rect_from_bounds(x: f64, y: f64, width: f64, height: f64) -> wry::Rect {
    make_rect(x, y, width, height)
}

// ── Callbacks: all Python callbacks in one struct (single Arc) ─────────────

struct Callbacks {
    ipc: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
    nav: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
    page_load: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
    title: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
    new_win: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
    drag_drop: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
    download_started: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
    download_completed: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
}

// ── Build native window handle (platform-specific, extracted for clarity) ──

fn build_window_handle(
    parent_hwnd: isize,
    #[allow(unused_variables)] parent_hwnd_kind: Option<&WindowHandleKind>,
) -> PyResult<raw_window_handle::WindowHandle<'_>> {
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
        let hwnd_nz = NonZero::new(parent_hwnd).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("parent_hwnd is null")
        })?;
        let win32 = Win32WindowHandle::new(hwnd_nz);
        let raw = RawWindowHandle::Win32(win32);
        // SAFETY: `borrow_raw` takes `RawWindowHandle` by value (move).
        // The HWND value is copied into WindowHandle; the caller (Python side)
        // is responsible for keeping the parent window alive.
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }

    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::{AppKitWindowHandle, RawWindowHandle};
        use std::ptr::NonNull;
        let ptr = parent_hwnd as *mut std::ffi::c_void;
        let ns_view = NonNull::new(ptr).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("parent_hwnd is null")
        })?;
        let appkit = AppKitWindowHandle::new(ns_view);
        let raw = RawWindowHandle::AppKit(appkit);
        // SAFETY: `borrow_raw` takes `RawWindowHandle` by value (move).
        // The NSView pointer is copied into WindowHandle; the caller is
        // responsible for keeping the parent view alive.
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // Only X11 uses raw window handles. GTK uses build_gtk (handled in
        // the caller), and RawWindowHandle doesn't have a Wayland variant
        // that wry accepts — wry's build/build_as_child only work with X11.
        use raw_window_handle::{RawWindowHandle, XlibWindowHandle};
        if parent_hwnd == 0 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "parent_hwnd is null",
            ));
        }
        let raw = RawWindowHandle::Xlib(XlibWindowHandle::new(parent_hwnd as u64));
        // SAFETY: `borrow_raw` takes `RawWindowHandle` by value (move).
        // The XID is copied into WindowHandle; the caller is responsible
        // for keeping the parent window alive.
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }
}

// ── SharedWebContext: holds either an owned or Rc-shared wry::WebContext ──

#[allow(dead_code)]
enum SharedWebContext {
    Owned(wry::WebContext),
    Shared(Rc<RefCell<wry::WebContext>>),
}

// ── WebContext: Python-facing shared browser context ──────────────────────

/// Sharable WebContext for persistent state (cookies, cache, storage).
///
/// Create once and pass to multiple [`WebView`] instances to share
/// browsing data — cookies set in one WebView are visible in another.
///
/// ```python
/// ctx = wryview.WebContext(data_directory="/path/to/profile")
/// wv1 = WebView(hwnd1, web_context=ctx, ...)
/// wv2 = WebView(hwnd2, web_context=ctx, ...)  # shares cookies/cache
/// ```
#[pyclass(unsendable)]
struct WebContext {
    inner: Rc<RefCell<wry::WebContext>>,
}

#[pymethods]
impl WebContext {
    #[new]
    #[pyo3(signature = (data_directory = None))]
    fn new(data_directory: Option<String>) -> Self {
        let ctx = wry::WebContext::new(data_directory.map(std::path::PathBuf::from));
        WebContext {
            inner: Rc::new(RefCell::new(ctx)),
        }
    }

    #[getter]
    fn data_directory(&self) -> Option<String> {
        self.inner
            .borrow()
            .data_directory()
            .map(|p| p.to_string_lossy().to_string())
    }
}

// ── The Python class ───────────────────────────────────────────────────────

#[pyclass(unsendable)]
struct WebView {
    // inner is only accessed from the Python thread (WebView is unsendable),
    // so RefCell is the appropriate interior-mutability primitive here.
    inner: RefCell<Option<wry::WebView>>,
    callbacks: Arc<Callbacks>,
    _web_context: Option<SharedWebContext>,
}

#[pymethods]
impl WebView {
    // ── Constructor ────────────────────────────────────────────────────────

    #[new]
    #[pyo3(signature = (
        parent_hwnd,
        *,
        width = 800u32,
        height = 600u32,
        url = None,
        html = None,
        transparent = false,
        background_color = None,
        visible = true,
        devtools = false,
        incognito = false,
        user_agent = None,
        focused = true,
        autoplay = false,
        javascript_enabled = true,
        hotkeys_zoom = true,
        initialization_script = None,
        ipc_handler = None,
        on_navigation = None,
        on_page_load = None,
        on_title_changed = None,
        on_new_window = None,
        drag_drop_handler = None,
        custom_protocols = None,
        proxy = None,
        back_forward_gestures = false,
        clipboard = true,
        https_scheme = false,
        default_context_menus = true,
        data_directory = None,
        web_context = None,
        headers = None,
        on_download_started = None,
        on_download_completed = None,
        as_child = true,
        parent_hwnd_kind = None,
    ))]
    fn new(
        py: Python<'_>,
        parent_hwnd: isize,
        width: u32,
        height: u32,
        url: Option<String>,
        html: Option<String>,
        transparent: bool,
        background_color: Option<(u8, u8, u8, u8)>,
        visible: bool,
        devtools: bool,
        incognito: bool,
        user_agent: Option<String>,
        focused: bool,
        autoplay: bool,
        javascript_enabled: bool,
        hotkeys_zoom: bool,
        initialization_script: Option<String>,
        ipc_handler: Option<pyo3::Py<pyo3::PyAny>>,
        on_navigation: Option<pyo3::Py<pyo3::PyAny>>,
        on_page_load: Option<pyo3::Py<pyo3::PyAny>>,
        on_title_changed: Option<pyo3::Py<pyo3::PyAny>>,
        on_new_window: Option<pyo3::Py<pyo3::PyAny>>,
        drag_drop_handler: Option<pyo3::Py<pyo3::PyAny>>,
        custom_protocols: Option<HashMap<String, pyo3::Py<pyo3::PyAny>>>,
        proxy: Option<HashMap<String, String>>,
        back_forward_gestures: bool,
        clipboard: bool,
        https_scheme: bool,
        default_context_menus: bool,
        data_directory: Option<String>,
        web_context: Option<pyo3::Py<WebContext>>,
        headers: Option<pyo3::Py<pyo3::PyAny>>,
        on_download_started: Option<pyo3::Py<pyo3::PyAny>>,
        on_download_completed: Option<pyo3::Py<pyo3::PyAny>>,
        as_child: bool,
        #[allow(unused_variables)] parent_hwnd_kind: Option<WindowHandleKind>,
    ) -> PyResult<Self> {
        // ── Callback slots (single Arc, 8 Mutexes inside) ───────────────
        let callbacks = Arc::new(Callbacks {
            ipc: Mutex::new(ipc_handler),
            nav: Mutex::new(on_navigation),
            page_load: Mutex::new(on_page_load),
            title: Mutex::new(on_title_changed),
            new_win: Mutex::new(on_new_window),
            drag_drop: Mutex::new(drag_drop_handler),
            download_started: Mutex::new(on_download_started),
            download_completed: Mutex::new(on_download_completed),
        });

        let protocols: Arc<Mutex<HashMap<String, pyo3::Py<pyo3::PyAny>>>> =
            Arc::new(Mutex::new(custom_protocols.unwrap_or_default()));

        // ── wry IPC handler ─────────────────────────────────────────────
        let cb = callbacks.clone();
        let ipc_handler_wry = move |req: wry::http::Request<String>| {
            let body = req.body().clone();
            Python::attach(|py| {
                // clone_ref only bumps refcount — no Python code, safe inside the lock
                let func = cb
                    .ipc
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|f| f.clone_ref(py)));
                if let Some(ref func) = func {
                    if let Err(e) = func.call1(py, (body,)) {
                        e.write_unraisable(py, None);
                    }
                }
            })
        };

        // ── wry navigation handler ──────────────────────────────────────
        let cb = callbacks.clone();
        let nav_handler = move |url: String| {
            Python::attach(|py| {
                // clone_ref only bumps refcount — no Python code, safe inside the lock
                let func = cb
                    .nav
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|f| f.clone_ref(py)));
                if let Some(ref func) = func {
                    if let Ok(result) = func.call1(py, (url.as_str(),)) {
                        return result.extract::<bool>(py).unwrap_or(true);
                    }
                }
                true // default: allow navigation
            })
        };

        // ── wry page-load handler ───────────────────────────────────────
        let cb = callbacks.clone();
        let pageload_handler = move |event: wry::PageLoadEvent, url: String| {
            let evt = match event {
                wry::PageLoadEvent::Started => PageLoadEvent::Started,
                wry::PageLoadEvent::Finished => PageLoadEvent::Finished,
            };
            Python::attach(|py| {
                // clone_ref only bumps refcount — no Python code, safe inside the lock
                let func = cb
                    .page_load
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|f| f.clone_ref(py)));
                if let Some(ref func) = func {
                    if let Err(e) = func.call1(py, (evt, url.as_str())) {
                        e.write_unraisable(py, None);
                    }
                }
            })
        };

        // ── wry title handler ───────────────────────────────────────────
        let cb = callbacks.clone();
        let title_handler = move |title: String| {
            Python::attach(|py| {
                // clone_ref only bumps refcount — no Python code, safe inside the lock
                let func = cb
                    .title
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|f| f.clone_ref(py)));
                if let Some(ref func) = func {
                    if let Err(e) = func.call1(py, (title.as_str(),)) {
                        e.write_unraisable(py, None);
                    }
                }
            })
        };

        // ── wry new-window handler ──────────────────────────────────────
        let cb = callbacks.clone();
        let newwin_handler =
            move |url: String, _features: wry::NewWindowFeatures| -> wry::NewWindowResponse {
                Python::attach(|py| {
                    // clone_ref only bumps refcount — no Python code, safe inside the lock
                    let func = cb
                        .new_win
                        .lock()
                        .ok()
                        .and_then(|g| g.as_ref().map(|f| f.clone_ref(py)));
                    if let Some(ref func) = func {
                        if let Ok(result) = func.call1(py, (url.as_str(),)) {
                            if let Ok(resp) = result.extract::<NewWindowResponse>(py) {
                                return match resp {
                                    NewWindowResponse::Deny => wry::NewWindowResponse::Deny,
                                    NewWindowResponse::Allow => wry::NewWindowResponse::Allow,
                                };
                            }
                        }
                    }
                    wry::NewWindowResponse::Allow
                })
            };

        // ── wry drag-drop handler ───────────────────────────────────────
        let cb = callbacks.clone();
        let drag_drop_handler = move |event: wry::DragDropEvent| -> bool {
            let (evt_type, paths, position) = match &event {
                wry::DragDropEvent::Enter { paths: p, position } => {
                    (DragDropEvent::Enter, p.clone(), *position)
                }
                wry::DragDropEvent::Over { position } => (DragDropEvent::Over, vec![], *position),
                wry::DragDropEvent::Drop { paths: p, position } => {
                    (DragDropEvent::Drop, p.clone(), *position)
                }
                wry::DragDropEvent::Leave => (DragDropEvent::Leave, vec![], (0, 0)),
                _ => (DragDropEvent::Unknown, vec![], (0, 0)),
            };
            Python::attach(|py| {
                // clone_ref only bumps refcount — no Python code, safe inside the lock
                let func = cb
                    .drag_drop
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|f| f.clone_ref(py)));
                if let Some(ref func) = func {
                    let paths_str: Vec<String> = paths
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    let pos = (position.0, position.1);
                    if let Ok(result) = func.call1(py, (evt_type, paths_str, pos)) {
                        return result.extract::<bool>(py).unwrap_or(false);
                    }
                }
                false
            })
        };

        // ── wry download-started handler ────────────────────────────────
        let cb = callbacks.clone();
        let download_started_handler = move |url: String, path: &mut std::path::PathBuf| -> bool {
            Python::attach(|py| {
                // clone_ref only bumps refcount — no Python code, safe inside the lock
                let func = cb
                    .download_started
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|f| f.clone_ref(py)));
                if let Some(ref func) = func {
                    let path_str = path.to_string_lossy().to_string();
                    if let Ok(result) = func.call1(py, (url.as_str(), path_str)) {
                        if let Ok(new_path) = result.extract::<String>(py) {
                            *path = std::path::PathBuf::from(new_path);
                        }
                        return result.extract::<bool>(py).unwrap_or(true);
                    }
                }
                true // default: allow download
            })
        };

        // ── wry download-completed handler ──────────────────────────────
        let cb = callbacks.clone();
        let download_completed_handler =
            move |url: String, path: Option<std::path::PathBuf>, success: bool| {
                Python::attach(|py| {
                    // clone_ref only bumps refcount — no Python code, safe inside the lock
                    let func = cb
                        .download_completed
                        .lock()
                        .ok()
                        .and_then(|g| g.as_ref().map(|f| f.clone_ref(py)));
                    if let Some(ref func) = func {
                        let path_str = path.map(|p| p.to_string_lossy().to_string());
                        if let Err(e) = func.call1(py, (url.as_str(), path_str, success)) {
                            e.write_unraisable(py, None);
                        }
                    }
                })
            };

        // ── Determine WebContext (shared > owned > none) ──────────────
        // Extract Rc from a Python WebContext, or create an owned one
        // from data_directory.  web_context takes priority.
        let shared_rc: Option<Rc<RefCell<wry::WebContext>>> = web_context
            .as_ref()
            .map(|py_ctx| py_ctx.borrow(py).inner.clone());

        let mut owned_ctx: Option<wry::WebContext> = if shared_rc.is_none() {
            data_directory.map(|d| wry::WebContext::new(Some(std::path::PathBuf::from(d))))
        } else {
            None
        };

        // ── Build wry WebViewBuilder ────────────────────────────────────
        // We wrap the entire builder section in a block so shared_guard /
        // builder borrows are released before we move shared_rc / owned_ctx
        // into the return value below.
        let webview = {
            let mut shared_guard;
            let mut builder;
            if let Some(ref rc) = shared_rc {
                shared_guard = rc.borrow_mut();
                builder = wry::WebViewBuilder::new_with_web_context(&mut *shared_guard);
            } else if let Some(ref mut ctx) = owned_ctx {
                builder = wry::WebViewBuilder::new_with_web_context(ctx);
            } else {
                builder = wry::WebViewBuilder::new();
            }
            builder = builder
                .with_bounds(rect_from_bounds(0.0, 0.0, width as f64, height as f64))
                .with_transparent(transparent)
                .with_visible(visible)
                .with_devtools(devtools)
                .with_incognito(incognito)
                .with_focused(focused)
                .with_autoplay(autoplay)
                .with_hotkeys_zoom(hotkeys_zoom)
                .with_back_forward_navigation_gestures(back_forward_gestures)
                .with_clipboard(clipboard)
                .with_ipc_handler(ipc_handler_wry)
                .with_navigation_handler(nav_handler)
                .with_on_page_load_handler(pageload_handler)
                .with_document_title_changed_handler(title_handler)
                .with_new_window_req_handler(newwin_handler)
                .with_drag_drop_handler(drag_drop_handler)
                .with_download_started_handler(download_started_handler)
                .with_download_completed_handler(download_completed_handler);

            // ── Custom protocols ────────────────────────────────────────────
            let protocol_handlers: Vec<(String, pyo3::Py<pyo3::PyAny>)> = {
                let guard = protocols.lock().unwrap();
                guard
                    .iter()
                    .map(|(name, h)| (name.clone(), h.clone_ref(py)))
                    .collect()
            };

            for (name, handler) in protocol_handlers {
                let handler_arc = Arc::new(handler);
                builder = builder.with_asynchronous_custom_protocol(
                    name,
                    move |_id: wry::WebViewId,
                          request: wry::http::Request<Vec<u8>>,
                          responder: wry::RequestAsyncResponder| {
                        let h = Arc::clone(&handler_arc);
                        Python::attach(|py| {
                            let handler = h.as_ref().clone_ref(py);
                            let method = request.method().to_string();
                            let uri = request.uri().to_string();
                            let mut headers: Vec<(String, String)> = Vec::new();
                            for (k, v) in request.headers().iter() {
                                headers.push((
                                    k.as_str().to_string(),
                                    v.to_str().unwrap_or("").to_string(),
                                ));
                            }
                            let body = request.body().clone();

                            // Wrap responder as a Python callable.
                            let cell = Arc::new(Mutex::new(Some(responder)));
                            let respond =
                                pyo3::types::PyCFunction::new_closure(
                                    py,
                                    None,
                                    None,
                                    move |args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
                                          _kwargs: Option<
                                        &pyo3::Bound<'_, pyo3::types::PyDict>,
                                    >| {
                                        if let Ok(mut r_opt) = cell.lock() {
                                            if let Some(r) = r_opt.take() {
                                                let status: u16 = args
                                                    .get_item(0)
                                                    .ok()
                                                    .and_then(|v| v.extract::<u16>().ok())
                                                    .unwrap_or(500);
                                                let resp_headers: Vec<(String, String)> = args
                                                    .get_item(1)
                                                    .ok()
                                                    .and_then(|v| v.extract().ok())
                                                    .unwrap_or_default();
                                                let resp_body: Vec<u8> = args
                                                    .get_item(2)
                                                    .ok()
                                                    .and_then(|v| v.extract().ok())
                                                    .unwrap_or_default();
                                                let mut builder =
                                                    wry::http::Response::builder().status(status);
                                                for (k, v) in &resp_headers {
                                                    builder =
                                                        builder.header(k.as_str(), v.as_str());
                                                }
                                                let response = builder
                                                    .body(std::borrow::Cow::Owned(resp_body))
                                                    .unwrap();
                                                drop(r_opt);
                                                let py = args.py();
                                                py.detach(move || {
                                                    let _ = r.respond(response);
                                                });
                                            }
                                        }
                                        Ok::<_, pyo3::PyErr>(args.py().None())
                                    },
                                )
                                .unwrap();

                            if let Err(e) =
                                handler.call(py, (method, uri, headers, body, respond), None)
                            {
                                e.write_unraisable(py, None);
                            }
                        })
                    },
                );
            }

            // ── Proxy ───────────────────────────────────────────────────────
            if let Some(ref p) = proxy {
                use wry::ProxyConfig;
                let ep = wry::ProxyEndpoint {
                    host: p.get("host").cloned().unwrap_or_default(),
                    port: p.get("port").cloned().unwrap_or_default(),
                };
                let cfg = match p.get("type").map(|s| s.as_str()) {
                    Some("socks5") => ProxyConfig::Socks5(ep),
                    _ => ProxyConfig::Http(ep),
                };
                builder = builder.with_proxy_config(cfg);
            }

            // ── Remaining builder options ───────────────────────────────────
            if !javascript_enabled {
                builder = builder.with_javascript_disabled();
            }
            // https_scheme: custom protocol uses https:// prefix → secure context.
            // Windows (WebView2) only — WebView2 converts custom schemes to
            // http://{name}.{path} by default; https_scheme makes it https://.
            // macOS / Linux use native <scheme>://{path} and don't need this.
            if https_scheme {
                #[cfg(target_os = "windows")]
                {
                    use wry::WebViewBuilderExtWindows;
                    builder = builder.with_https_scheme(true);
                }
                // non-Windows: silently ignored (already native scheme, no workaround)
            }
            // default_context_menus: Windows only — enable/disable native right-click menu.
            if !default_context_menus {
                #[cfg(target_os = "windows")]
                {
                    use wry::WebViewBuilderExtWindows;
                    builder = builder.with_default_context_menus(false);
                }
            }
            if let Some(bg) = background_color {
                builder = builder.with_background_color(bg);
            }
            if let Some(ref ua) = user_agent {
                builder = builder.with_user_agent(ua);
            }
            // url must come before headers: with_url clears headers internally
            if let Some(ref u) = url {
                builder = builder.with_url(u);
            }
            if let Some(ref h) = headers {
                let bound = h.bind(py);
                let pairs: Vec<(String, String)> = bound
                    .extract::<Vec<(String, String)>>()
                    .ok()
                    .or_else(|| {
                        let dict: HashMap<String, String> = bound.extract().ok()?;
                        Some(dict.into_iter().collect())
                    })
                    .unwrap_or_default();
                let mut header_map = wry::http::HeaderMap::new();
                for (k, v) in pairs {
                    if let (Ok(name), Ok(value)) = (
                        wry::http::header::HeaderName::from_bytes(k.as_bytes()),
                        wry::http::header::HeaderValue::from_str(&v),
                    ) {
                        header_map.insert(name, value);
                    }
                }
                builder = builder.with_headers(header_map);
            }
            if let Some(ref script) = initialization_script {
                builder = builder.with_initialization_script(script);
            }
            if let Some(ref h) = html {
                builder = builder.with_html(h);
            }

            // ── GTK init (Linux only, idempotent) ──────────────────────────
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                match gtk::init() {
                    Ok(()) => { /* GTK initialized */ }
                    Err(ref e) => {
                        // "already initialized" is harmless — gtk::init() is idempotent
                        if !gtk::is_initialized() {
                            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                                format!("GTK init failed: {}. Is $DISPLAY set?", e),
                            ));
                        }
                    }
                }
            }

            // ── Build the webview (inner block returns the Result) ──────────
            {
                #[cfg(all(unix, not(target_os = "macos")))]
                if parent_hwnd_kind == Some(WindowHandleKind::Gtk) {
                    // GTK path — embeds directly in a GTK container.
                    // Works on both X11 and Wayland via GTK's backend abstraction.
                    use gtk::glib::translate::FromGlibPtrNone;
                    use wry::WebViewBuilderExtUnix;
                    let container =
                        unsafe { gtk::Container::from_glib_none(parent_hwnd as *mut _) };
                    builder.build_gtk(&container)
                } else {
                    let window_handle =
                        build_window_handle(parent_hwnd, parent_hwnd_kind.as_ref())?;
                    if as_child {
                        builder.build_as_child(&window_handle)
                    } else {
                        builder.build(&window_handle)
                    }
                }
                #[cfg(not(all(unix, not(target_os = "macos"))))]
                {
                    let window_handle =
                        build_window_handle(parent_hwnd, parent_hwnd_kind.as_ref())?;
                    if as_child {
                        builder.build_as_child(&window_handle)
                    } else {
                        builder.build(&window_handle)
                    }
                }
            }
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?
        }; // block end — shared_guard, builder (and their borrows) dropped here

        // ── Store the context (keeps it alive as long as the WebView) ─
        let stored_ctx = match (shared_rc, owned_ctx) {
            (Some(rc), _) => Some(SharedWebContext::Shared(rc)),
            (None, Some(ctx)) => Some(SharedWebContext::Owned(ctx)),
            (None, None) => None,
        };

        Ok(Self {
            inner: RefCell::new(Some(webview)),
            callbacks,
            _web_context: stored_ctx,
        })
    }

    // ── Content ────────────────────────────────────────────────────────────

    fn load_url(&self, url: &str) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.load_url(url)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn load_html(&self, html: &str) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.load_html(html)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn load_url_with_headers(
        &self,
        _py: Python<'_>,
        url: &str,
        headers: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        let mut header_map = wry::http::HeaderMap::new();
        let pairs: Vec<(String, String)> = headers
            .extract::<Vec<(String, String)>>()
            .ok()
            .or_else(|| {
                let dict: HashMap<String, String> = headers.extract().ok()?;
                Some(dict.into_iter().collect())
            })
            .unwrap_or_default();
        for (k, v) in pairs {
            if let (Ok(name), Ok(value)) = (
                wry::http::header::HeaderName::from_bytes(k.as_bytes()),
                wry::http::header::HeaderValue::from_str(&v),
            ) {
                header_map.insert(name, value);
            }
        }
        wv.load_url_with_headers(url, header_map)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn reload(&self) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.reload()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn url(&self) -> Option<String> {
        self.inner
            .try_borrow()
            .ok()
            .and_then(|g| g.as_ref().and_then(|w| w.url().ok()))
    }

    // ── JavaScript ─────────────────────────────────────────────────────────

    fn eval_js(&self, script: &str) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.evaluate_script(script)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn eval_js_with_callback(&self, script: &str, callback: pyo3::Py<pyo3::PyAny>) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.evaluate_script_with_callback(script, move |result: String| {
            Python::attach(|py| {
                if let Err(e) = callback.call1(py, (result,)) {
                    e.write_unraisable(py, None);
                }
            })
        })
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    // ── IPC ────────────────────────────────────────────────────────────────

    fn set_ipc_handler(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut guard) = self.callbacks.ipc.lock() {
            *guard = Some(handler);
        }
    }

    fn clear_ipc_handler(&self) {
        if let Ok(mut guard) = self.callbacks.ipc.lock() {
            *guard = None;
        }
    }

    // ── Callback setters ───────────────────────────────────────────────────

    fn set_on_navigation(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.callbacks.nav.lock() {
            *g = Some(handler);
        }
    }

    fn set_on_page_load(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.callbacks.page_load.lock() {
            *g = Some(handler);
        }
    }

    fn set_on_title_changed(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.callbacks.title.lock() {
            *g = Some(handler);
        }
    }

    fn set_on_new_window(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.callbacks.new_win.lock() {
            *g = Some(handler);
        }
    }

    fn set_drag_drop_handler(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.callbacks.drag_drop.lock() {
            *g = Some(handler);
        }
    }

    fn set_on_download_started(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.callbacks.download_started.lock() {
            *g = Some(handler);
        }
    }

    fn set_on_download_completed(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.callbacks.download_completed.lock() {
            *g = Some(handler);
        }
    }

    // ── Reparent ────────────────────────────────────────────────────────────

    /// Re-attach the webview to a different parent window.
    ///
    /// **Windows**: *new_parent* must be a valid ``HWND``.
    ///
    /// **macOS**: *new_parent* **must** be an ``NSWindow`` pointer.  Passing
    /// an ``NSView`` or any other value will crash.  Set a breakpoint or log
    /// before calling this if you're unsure.
    ///
    /// **Linux**: not supported — the underlying API needs a GTK container,
    /// not a raw XID.  Returns ``NotImplementedError``.
    fn reparent(&self, new_parent: isize) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        #[cfg(target_os = "windows")]
        {
            use wry::WebViewExtWindows;
            wv.reparent(new_parent)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        }
        #[cfg(target_os = "macos")]
        {
            use wry::WebViewExtMacOS;
            wv.reparent(new_parent as *mut _)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let _ = (new_parent, wv);
            return Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                "reparent is not supported on Linux X11/Wayland — wry needs a GTK container, not a raw XID",
            ));
        }
        #[cfg(not(all(unix, not(target_os = "macos"))))]
        {
            Ok(())
        }
    }

    // ── Geometry ───────────────────────────────────────────────────────────

    fn set_bounds(&self, x: f64, y: f64, width: f64, height: f64) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.set_bounds(make_rect(x, y, width, height))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        self.inner.try_borrow().ok().and_then(|g| {
            g.as_ref().and_then(|w| {
                w.bounds().ok().map(|r| {
                    let (x, y) = match r.position {
                        wry::dpi::Position::Logical(l) => (l.x, l.y),
                        wry::dpi::Position::Physical(p) => (p.x as f64, p.y as f64),
                    };
                    let (w, h) = match r.size {
                        wry::dpi::Size::Logical(l) => (l.width, l.height),
                        wry::dpi::Size::Physical(p) => (p.width as f64, p.height as f64),
                    };
                    (x, y, w, h)
                })
            })
        })
    }

    // ── Visibility ─────────────────────────────────────────────────────────

    fn set_visible(&self, visible: bool) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.set_visible(visible)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn set_background_color(&self, r: u8, g: u8, b: u8, a: u8) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.set_background_color((r, g, b, a))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn focus(&self) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.focus()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    // ── Zoom ───────────────────────────────────────────────────────────────

    fn zoom(&self, scale: f64) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.zoom(scale)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    // ── DevTools ───────────────────────────────────────────────────────────

    fn open_devtools(&self) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.open_devtools();
        Ok(())
    }

    fn close_devtools(&self) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.close_devtools();
        Ok(())
    }

    fn is_devtools_open(&self) -> bool {
        self.inner.try_borrow().ok().map_or(false, |g| {
            g.as_ref().map_or(false, |w| w.is_devtools_open())
        })
    }

    // ── Cookies ────────────────────────────────────────────────────────────

    fn cookies(&self) -> PyResult<Vec<CookieDict>> {
        let guard = self
            .inner
            .try_borrow()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        let cookies = wv
            .cookies()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        Ok(cookies.into_iter().map(CookieDict::from_cookie).collect())
    }

    fn cookies_for_url(&self, url: &str) -> PyResult<Vec<CookieDict>> {
        let guard = self
            .inner
            .try_borrow()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        let cookies = wv
            .cookies_for_url(url)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        Ok(cookies.into_iter().map(CookieDict::from_cookie).collect())
    }

    fn set_cookie(
        &self,
        name: &str,
        value: &str,
        domain: Option<&str>,
        path: Option<&str>,
    ) -> PyResult<()> {
        use wry::cookie::Cookie;
        let mut builder = Cookie::build((name, value));
        if let Some(d) = domain {
            builder = builder.domain(d);
        }
        if let Some(p) = path {
            builder = builder.path(p);
        }
        let cookie = builder.build();
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.set_cookie(&cookie)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn delete_cookie(&self, name: &str, url: &str) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        let cookies = wv
            .cookies_for_url(url)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        for c in cookies {
            if c.name() == name {
                wv.delete_cookie(&c).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e))
                })?;
            }
        }
        Ok(())
    }

    /// Explicitly destroy the underlying WebView.
    ///
    /// Drops the native ``wry::WebView``, which runs WebView2 / WKWebView
    /// cleanup (including window class unregistration on Windows) while
    /// the parent window is still alive.
    ///
    /// Call this before program exit to avoid harmless but noisy errors
    /// like "Failed to unregister class Chrome_WidgetWin_0" from deferred
    /// cleanup.
    fn close(&self) -> PyResult<()> {
        let mut guard = self.inner.try_borrow_mut().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        *guard = None; // take and drop the wry::WebView
        Ok(())
    }

    // ── Misc ───────────────────────────────────────────────────────────────

    fn print(&self) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.print()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    fn clear_all_browsing_data(&self) -> PyResult<()> {
        let guard = self.inner.try_borrow().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "RefCell borrow error: {}",
                e
            ))
        })?;
        let wv = guard.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("WebView is already closed")
        })?;
        wv.clear_all_browsing_data()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }
}

// ── Enums ──────────────────────────────────────────────────────────────────

#[pyclass(eq, frozen, hash, from_py_object)]
#[derive(Clone, PartialEq, Hash)]
enum WindowHandleKind {
    /// Windows HWND
    Win32,
    /// macOS NSView pointer
    AppKit,
    /// Linux X11 XID (default on Linux)
    X11,
    /// Linux GTK container pointer (recommended for Wayland compatibility)
    Gtk,
}

#[pyclass(eq, frozen, hash, skip_from_py_object)]
#[derive(Clone, PartialEq, Hash)]
enum PageLoadEvent {
    Started,
    Finished,
}

#[pyclass(eq, frozen, hash, from_py_object)]
#[derive(Clone, PartialEq, Hash)]
enum NewWindowResponse {
    Allow,
    Deny,
}

#[pyclass(eq, frozen, hash, skip_from_py_object)]
#[derive(Clone, PartialEq, Hash)]
enum DragDropEvent {
    Enter,
    Over,
    Drop,
    Leave,
    Unknown,
}

// ── Cookie dict helper ────────────────────────────────────────────────────

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct CookieDict {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    value: String,
    #[pyo3(get)]
    domain: Option<String>,
    #[pyo3(get)]
    path: Option<String>,
    #[pyo3(get)]
    secure: bool,
    #[pyo3(get)]
    http_only: bool,
}

impl CookieDict {
    fn from_cookie(c: wry::cookie::Cookie<'_>) -> Self {
        Self {
            name: c.name().to_string(),
            value: c.value().to_string(),
            domain: c.domain().map(|s| s.to_string()),
            path: c.path().map(|s| s.to_string()),
            secure: c.secure().unwrap_or(false),
            http_only: c.http_only().unwrap_or(false),
        }
    }
}

// ── Platform event pump ─────────────────────────────────────────────────
///
/// Pump platform-specific events (no-op on Windows/macOS).
///
/// On Linux, call this periodically (e.g. via ``QTimer`` at ~60 fps) to
/// keep WebKitGTK rendering when the host app runs a non-GTK event loop
/// such as Qt's xcb backend.
///
/// This is a module-level function — it does not belong to any single
/// ``WebView`` instance.
#[pyfunction]
fn pump_events() {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // Process pending GTK/GLib events without blocking.
        // Equivalent to Python: while Gtk.events_pending(): Gtk.main_iteration_do(False)
        while gtk::main_iteration_do(false) {
            // keep pumping while events remain
        }
    }
    // Windows / macOS: nothing needed — the native webview integrates with
    // the same event loop that Qt uses (Windows message pump / NSRunLoop).
}

/// Initialise GTK from Rust's perspective (no-op on Windows/macOS).
///
/// Python's ``gi.repository.Gtk.init()`` calls C's ``gtk_init()``, which
/// initialises GTK at the OS level.  However the Rust ``gtk`` crate tracks
/// initialisation state in a *separate* static ``AtomicBool``, and the
/// ``webkit2gtk`` crate (which wry uses internally on Linux) checks that
/// Rust-level flag.
///
/// Call this **once** before creating the first ``WebView`` on Linux.  It is
/// idempotent — safe to call after Python's ``Gtk.init()`` or multiple times.
#[pyfunction]
fn ensure_gtk_init() {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = gtk::init(); // idempotent; sets the Rust IS_INITIALIZED atomic
    }
    // Windows / macOS: nothing needed
}

// ── Module ─────────────────────────────────────────────────────────────────

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<WebContext>()?;
    m.add_class::<WebView>()?;
    m.add_class::<CookieDict>()?;
    m.add_class::<WindowHandleKind>()?;
    m.add_class::<PageLoadEvent>()?;
    m.add_class::<NewWindowResponse>()?;
    m.add_class::<DragDropEvent>()?;
    m.add_function(wrap_pyfunction!(pump_events, m)?)?;
    m.add_function(wrap_pyfunction!(ensure_gtk_init, m)?)?;
    Ok(())
}
