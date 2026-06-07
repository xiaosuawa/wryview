/// wryview — Comprehensive Python binding for wry.
///
/// Exposes wry's full API: WebView creation (child or standalone window),
/// JS evaluation, IPC handler, custom protocol, navigation callbacks,
/// cookies, devtools, and more.
///
/// No event loop, no window management — just the webview.
use pyo3::prelude::*;
use std::num::NonZero;
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

// ── The Python class ───────────────────────────────────────────────────────

#[pyclass]
struct WebView {
    inner: Mutex<Option<wry::WebView>>,
    ipc_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>>,
    nav_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>>,
    pageload_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>>,
    title_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>>,
    newwin_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>>,
    drag_drop_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>>,
    _web_context: Option<wry::WebContext>,
}

use std::collections::HashMap;

// SAFETY: Python GIL ensures single-threaded access to the webview.
unsafe impl Send for WebView {}
unsafe impl Sync for WebView {}

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
        data_directory = None,
        headers = None,
    ))]
    fn new(
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
        data_directory: Option<String>,
        headers: Option<pyo3::Py<pyo3::PyAny>>,
    ) -> PyResult<Self> {
        // ── Build native window handle ─────────────────────────────────
        use raw_window_handle::{RawWindowHandle, Win32WindowHandle};

        let hwnd_nz = NonZero::new(parent_hwnd).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("parent_hwnd is null")
        })?;
        let win32 = Win32WindowHandle::new(hwnd_nz);
        let raw = RawWindowHandle::Win32(win32);
        let window_handle = unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) };

        // ── Callback slots ─────────────────────────────────────────────
        let ipc_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>> = Arc::new(Mutex::new(ipc_handler));
        let nav_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>> = Arc::new(Mutex::new(on_navigation));
        let pageload_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>> =
            Arc::new(Mutex::new(on_page_load));
        let title_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>> =
            Arc::new(Mutex::new(on_title_changed));
        let newwin_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>> =
            Arc::new(Mutex::new(on_new_window));
        let drag_drop_cb: Arc<Mutex<Option<pyo3::Py<pyo3::PyAny>>>> =
            Arc::new(Mutex::new(drag_drop_handler));
        let protocols: Arc<Mutex<HashMap<String, pyo3::Py<pyo3::PyAny>>>> =
            Arc::new(Mutex::new(custom_protocols.unwrap_or_default()));

        // ── wry callbacks (capture Arc clones) ─────────────────────────
        let ipc_cb_clone = ipc_cb.clone();
        let ipc_handler_wry = move |req: wry::http::Request<String>| {
            let body = req.body().clone();
            Python::attach(|py| {
                if let Ok(guard) = ipc_cb_clone.lock() {
                    if let Some(ref func) = *guard {
                        let _ = func.call1(py, (body,));
                    }
                }
            })
        };

        let nav_cb_clone = nav_cb.clone();
        let nav_handler = move |url: String| {
            Python::attach(|py| {
                if let Ok(guard) = nav_cb_clone.lock() {
                    if let Some(ref func) = *guard {
                        if let Ok(result) = func.call1(py, (url.as_str(),)) {
                            return result.extract::<bool>(py).unwrap_or(true);
                        }
                    }
                }
                true // default: allow navigation
            })
        };

        let pageload_cb_clone = pageload_cb.clone();
        let pageload_handler = move |event: wry::PageLoadEvent, url: String| {
            let evt = match event {
                wry::PageLoadEvent::Started => "Started",
                wry::PageLoadEvent::Finished => "Finished",
            };
            Python::attach(|py| {
                if let Ok(guard) = pageload_cb_clone.lock() {
                    if let Some(ref func) = *guard {
                        let _ = func.call1(py, (evt, url.as_str()));
                    }
                }
            })
        };

        let title_cb_clone = title_cb.clone();
        let title_handler = move |title: String| {
            Python::attach(|py| {
                if let Ok(guard) = title_cb_clone.lock() {
                    if let Some(ref func) = *guard {
                        let _ = func.call1(py, (title.as_str(),));
                    }
                }
            })
        };

        let newwin_cb_clone = newwin_cb.clone();
        let newwin_handler =
            move |url: String, _features: wry::NewWindowFeatures| -> wry::NewWindowResponse {
                Python::attach(|py| {
                    if let Ok(guard) = newwin_cb_clone.lock() {
                        if let Some(ref func) = *guard {
                            if let Ok(result) = func.call1(py, (url.as_str(),)) {
                                if let Ok(response) = result.extract::<String>(py) {
                                    return match response.as_str() {
                                        "deny" => wry::NewWindowResponse::Deny,
                                        _ => wry::NewWindowResponse::Allow,
                                    };
                                }
                            }
                        }
                    }
                    wry::NewWindowResponse::Allow
                })
            };

        // drag_drop handler
        let drag_drop_cb_clone = drag_drop_cb.clone();
        let drag_drop_handler = move |event: wry::DragDropEvent| -> bool {
            Python::attach(|py| {
                if let Ok(guard) = drag_drop_cb_clone.lock() {
                    if let Some(ref func) = *guard {
                        let (evt_type, paths, position) = match &event {
                            wry::DragDropEvent::Enter { paths: p, position } => {
                                ("Enter", p.clone(), *position)
                            }
                            wry::DragDropEvent::Over { position } => ("Over", vec![], *position),
                            wry::DragDropEvent::Drop { paths: p, position } => {
                                ("Drop", p.clone(), *position)
                            }
                            wry::DragDropEvent::Leave => ("Leave", vec![], (0, 0)),
                            _ => return false,
                        };
                        let paths_str: Vec<String> = paths
                            .iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                        let pos = (position.0, position.1);
                        if let Ok(result) = func.call1(py, (evt_type, paths_str, pos)) {
                            return result.extract::<bool>(py).unwrap_or(false);
                        }
                    }
                }
                false
            })
        };

        // ── Build ──────────────────────────────────────────────────────
        let mut web_context = data_directory
            .map(|d| wry::WebContext::new(Some(std::path::PathBuf::from(d))));
        let mut builder = if let Some(ref mut ctx) = web_context {
            wry::WebViewBuilder::new_with_web_context(ctx)
        } else {
            wry::WebViewBuilder::new()
        };
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
            .with_drag_drop_handler(drag_drop_handler);

        // custom protocols — extract before iterating to avoid borrow conflict
        let protocol_handlers: Vec<(String, pyo3::Py<pyo3::PyAny>)> = {
            let guard = protocols.lock().unwrap();
            let py = unsafe { Python::assume_attached() };
            guard
                .iter()
                .map(|(name, h)| (name.clone(), h.clone_ref(py)))
                .collect()
        };

        for (name, handler) in protocol_handlers {
            let handler_clone = handler.clone_ref(unsafe { Python::assume_attached() });
            builder = builder.with_asynchronous_custom_protocol(
                name,
                move |_id: wry::WebViewId, request: wry::http::Request<Vec<u8>>, responder: wry::RequestAsyncResponder| {
                    let h = handler_clone.clone_ref(unsafe { Python::assume_attached() });
                    Python::attach(|py| {
                        let handler = h.clone_ref(py);
                        let method = request.method().to_string();
                        let uri = request.uri().to_string();
                        let mut headers: Vec<(String, String)> = Vec::new();
                        for (k, v) in request.headers().iter() {
                            headers.push((k.as_str().to_string(), v.to_str().unwrap_or("").to_string()));
                        }
                        let body = request.body().clone();

                        // Wrap responder as Python callable
                        struct SendCell(std::sync::Mutex<Option<wry::RequestAsyncResponder>>);
                        unsafe impl Send for SendCell {}
                        let cell = SendCell(std::sync::Mutex::new(Some(responder)));
                        let respond = pyo3::types::PyCFunction::new_closure(
                            py, None, None,
                            move |args: &pyo3::Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>| {
                                let mut guard = cell.0.lock().unwrap();
                                if let Some(r) = guard.take() {
                                    let status: u16 = args.get_item(0).ok().and_then(|v| v.extract::<u16>().ok()).unwrap_or(500);
                                    let resp_headers: Vec<(String, String)> = args.get_item(1).ok().and_then(|v| v.extract().ok()).unwrap_or_default();
                                    let resp_body: Vec<u8> = args.get_item(2).ok().and_then(|v| v.extract().ok()).unwrap_or_default();
                                    let mut builder = wry::http::Response::builder().status(status);
                                    for (k, v) in &resp_headers {
                                        builder = builder.header(k.as_str(), v.as_str());
                                    }
                                    let _ = r.respond(builder.body(std::borrow::Cow::Owned(resp_body)).unwrap());
                                }
                                Ok::<_, pyo3::PyErr>(unsafe { Python::assume_attached() }.None())
                            }
                        ).unwrap();

                        let _ = handler.call(py, (method, uri, headers, body, respond), None);
                    })
                },
            );
        }

        // proxy
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

        if !javascript_enabled {
            builder = builder.with_javascript_disabled();
        }
        if let Some(bg) = background_color {
            builder = builder.with_background_color(bg);
        }
        if let Some(ref ua) = user_agent {
            builder = builder.with_user_agent(ua);
        }
        if let Some(ref h) = headers {
            let py = unsafe { Python::assume_attached() };
            let bound = h.bind(py);
            let pairs: Vec<(String, String)> = bound.extract::<Vec<(String, String)>>().ok()
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
        if let Some(ref u) = url {
            builder = builder.with_url(u);
        }
        if let Some(ref h) = html {
            builder = builder.with_html(h);
        }

        let webview = builder
            .build_as_child(&window_handle)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

        Ok(Self {
            inner: Mutex::new(Some(webview)),
            ipc_cb,
            nav_cb,
            pageload_cb,
            title_cb,
            newwin_cb,
            drag_drop_cb,
            _web_context: web_context,
        })
    }

    // ── Content ────────────────────────────────────────────────────────────

    fn load_url(&self, url: &str) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.load_url(url);
            }
        }
    }

    fn load_html(&self, html: &str) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.load_html(html);
            }
        }
    }

    fn load_url_with_headers(&self, _py: Python<'_>, url: &str, headers: pyo3::Bound<'_, pyo3::PyAny>) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let mut header_map = wry::http::HeaderMap::new();
                // Try Vec<(String, String)> first, then HashMap<String, String>
                let pairs: Vec<(String, String)> = headers.extract::<Vec<(String, String)>>().ok()
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
                let _ = wv.load_url_with_headers(url, header_map);
            }
        }
    }

    fn reload(&self) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.reload();
            }
        }
    }

    fn url(&self) -> Option<String> {
        self.inner
            .lock()
            .ok()
            .and_then(|g| g.as_ref().and_then(|w| w.url().ok()))
    }

    // ── JavaScript ─────────────────────────────────────────────────────────

    fn eval_js(&self, script: &str) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.evaluate_script(script);
            }
        }
    }

    fn eval_js_with_callback(&self, script: &str, callback: pyo3::Py<pyo3::PyAny>) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.evaluate_script_with_callback(script, move |result: String| {
                    Python::attach(|py| {
                        let _ = callback.call1(py, (result,));
                    })
                });
            }
        }
    }

    // ── IPC ────────────────────────────────────────────────────────────────

    fn set_ipc_handler(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut guard) = self.ipc_cb.lock() {
            *guard = Some(handler);
        }
    }

    fn clear_ipc_handler(&self) {
        if let Ok(mut guard) = self.ipc_cb.lock() {
            *guard = None;
        }
    }

    // ── Callback setters ───────────────────────────────────────────────────

    fn set_on_navigation(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.nav_cb.lock() {
            *g = Some(handler);
        }
    }

    fn set_on_page_load(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.pageload_cb.lock() {
            *g = Some(handler);
        }
    }

    fn set_on_title_changed(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.title_cb.lock() {
            *g = Some(handler);
        }
    }

    fn set_on_new_window(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.newwin_cb.lock() {
            *g = Some(handler);
        }
    }

    fn set_drag_drop_handler(&self, handler: pyo3::Py<pyo3::PyAny>) {
        if let Ok(mut g) = self.drag_drop_cb.lock() {
            *g = Some(handler);
        }
    }

    // ── Geometry ───────────────────────────────────────────────────────────

    fn set_bounds(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.set_bounds(make_rect(x, y, width, height));
            }
        }
    }

    fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        self.inner.lock().ok().and_then(|g| {
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

    fn set_visible(&self, visible: bool) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.set_visible(visible);
            }
        }
    }

    fn set_background_color(&self, r: u8, g: u8, b: u8, a: u8) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.set_background_color((r, g, b, a));
            }
        }
    }

    fn focus(&self) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.focus();
            }
        }
    }

    // ── Zoom ───────────────────────────────────────────────────────────────

    fn zoom(&self, scale: f64) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.zoom(scale);
            }
        }
    }

    // ── DevTools ───────────────────────────────────────────────────────────

    fn open_devtools(&self) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                wv.open_devtools();
            }
        }
    }

    fn close_devtools(&self) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                wv.close_devtools();
            }
        }
    }

    fn is_devtools_open(&self) -> bool {
        self.inner.lock().ok().map_or(false, |g| {
            g.as_ref().map_or(false, |w| w.is_devtools_open())
        })
    }

    // ── Cookies ────────────────────────────────────────────────────────────

    fn cookies(&self) -> PyResult<Vec<CookieDict>> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        let wv = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("destroyed"))?;
        let cookies = wv
            .cookies()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        Ok(cookies.into_iter().map(CookieDict::from_cookie).collect())
    }

    fn cookies_for_url(&self, url: &str) -> PyResult<Vec<CookieDict>> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        let wv = guard
            .as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("destroyed"))?;
        let cookies = wv
            .cookies_for_url(url)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        Ok(cookies.into_iter().map(CookieDict::from_cookie).collect())
    }

    fn set_cookie(&self, name: &str, value: &str, domain: Option<&str>, path: Option<&str>) {
        use wry::cookie::Cookie;
        let mut builder = Cookie::build((name, value));
        if let Some(d) = domain {
            builder = builder.domain(d);
        }
        if let Some(p) = path {
            builder = builder.path(p);
        }
        let cookie = builder.build();
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.set_cookie(&cookie);
            }
        }
    }

    fn delete_cookie(&self, name: &str, url: &str) {
        // Try to find matching cookies for the given URL first,
        // then delete each one by name.
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                if let Ok(cookies) = wv.cookies_for_url(url) {
                    for c in cookies {
                        if c.name() == name {
                            let _ = wv.delete_cookie(&c);
                        }
                    }
                }
            }
        }
    }

    // ── Misc ───────────────────────────────────────────────────────────────

    fn print(&self) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.print();
            }
        }
    }

    fn clear_all_browsing_data(&self) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(ref wv) = *guard {
                let _ = wv.clear_all_browsing_data();
            }
        }
    }
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

// ── Module ─────────────────────────────────────────────────────────────────

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<WebView>()?;
    m.add_class::<CookieDict>()?;
    Ok(())
}
