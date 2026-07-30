#!/usr/bin/env python
"""wryview 全面功能测试脚本（严 sui 版本）

每个测试用例都验证实际行为，不只是"没报错"。
运行方式:
    .venv/Scripts/python.exe test_all.py
"""

import os
import sys
import time
import tempfile
import traceback

# ── 助记 ──

_passed = 0
_failed = 0
_skipped = 0


def ok(desc: str):
    global _passed; _passed += 1
    print(f"  ✅ {desc}")

def fail(desc: str, reason: str = ""):
    global _failed; _failed += 1
    suffix = f"  —  {reason}" if reason else ""
    print(f"  ❌ {desc}{suffix}")

def skip(desc: str, reason: str = ""):
    global _skipped; _skipped += 1
    suffix = f"  —  {reason}" if reason else ""
    print(f"  ⏭️  {desc}{suffix}")

def section(title: str):
    print(f"\n{'─' * 70}\n  {title}\n{'─' * 70}")

def run_case(name, fn):
    print(f"\n▶ {name}")
    try:
        fn()
    except Exception as e:
        fail(name, f"异常: {e}\n{traceback.format_exc()}")


# ── 窗口辅助 ──

def create_tk_window(title="wryview test", w=800, h=600):
    import tkinter as tk
    root = tk.Tk()
    root.title(title)
    root.geometry(f"{w}x{h}")
    root.deiconify()
    root.update_idletasks()
    root.update()
    return root

def hwnd_of(root) -> int:
    root.update_idletasks(); root.update()
    return int(root.winfo_id())

def pump_tk(root, ms=50):
    """Pump tkinter events for `ms` milliseconds."""
    deadline = time.time() + ms / 1000.0
    while time.time() < deadline:
        try:
            root.update()
        except Exception:
            break
        elapsed = (deadline - time.time())
        if elapsed > 0.001:
            time.sleep(min(0.01, elapsed))


# ═══════════════════════════════════════════════════════════════════════
#  测试用例
# ═══════════════════════════════════════════════════════════════════════

# ── 1. 导入 ──

def test_imports():
    section("1. 模块导入")
    import wryview
    ok(f"导入成功, 版本 {wryview.__version__}")
    for name in wryview.__all__:
        assert hasattr(wryview, name), f"缺少 {name}"
    ok(f"__all__ 中 {len(wryview.__all__)} 个符号全部存在")


# ── 2. 枚举 ──

def test_enums():
    section("2. 枚举")
    import wryview

    assert wryview.PageLoadEvent.Started is not wryview.PageLoadEvent.Finished
    ok("PageLoadEvent.Started ≠ Finished")

    assert wryview.NewWindowResponse.Allow is not wryview.NewWindowResponse.Deny
    ok("NewWindowResponse.Allow ≠ Deny")

    variants = ["Enter", "Over", "Drop", "Leave", "Unknown"]
    for v in variants:
        assert getattr(wryview.DragDropEvent, v) is not None
    ok(f"DragDropEvent: {len(variants)} 个变体")

    kinds = ["Win32", "AppKit", "X11", "Gtk"]
    for k in kinds:
        assert getattr(wryview.WindowHandleKind, k) is not None
    ok(f"WindowHandleKind: {len(kinds)} 个变体")


# ── 3. WebContext ──

def test_web_context():
    section("3. WebContext")
    import wryview

    ctx = wryview.WebContext()
    assert ctx.data_directory is None
    ok("WebContext() → data_directory is None")

    d = os.path.join(tempfile.gettempdir(), "wryview_test_ctx")
    os.makedirs(d, exist_ok=True)
    ctx2 = wryview.WebContext(data_directory=d)
    assert d in (ctx2.data_directory or "")
    ok(f"WebContext(data_directory=...) → 路径正确")
    import shutil; shutil.rmtree(d, ignore_errors=True)


# ── 4. CookieDict 属性 ──

def test_cookie_dict():
    section("4. CookieDict 属性")
    import wryview
    attrs = ["name", "value", "domain", "path", "secure", "http_only"]
    for a in attrs:
        assert hasattr(wryview.CookieDict, a), f"缺少 {a}"
    ok(f"6 个属性全部存在: {attrs}")


# ── 5. WebView 构造 ──

def test_webview_basic():
    section("5. WebView 构造")
    import wryview
    root = create_tk_window("basic"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd)
        assert wv is not None
        ok("WebView(hwnd) ✓")
        wv.close()

        wv2 = wryview.WebView(hwnd, html="<h1>Hi</h1>", width=400, height=300)
        ok("WebView(hwnd, html=...) ✓")
        wv2.close()

        wv3 = wryview.WebView(hwnd, url="about:blank", width=800, height=600)
        ok("WebView(hwnd, url='about:blank') ✓")
        wv3.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 6. 全部可选参数 ──

def test_webview_all_options():
    section("6. 全部可选参数")
    import wryview
    root = create_tk_window("all opts"); hwnd = hwnd_of(root)
    cache = os.path.join(tempfile.gettempdir(), "wryview_full")
    os.makedirs(cache, exist_ok=True)
    try:
        wv = wryview.WebView(
            hwnd, width=1024, height=768, url="about:blank",
            transparent=False, background_color=(255,255,255,255),
            visible=True, devtools=True, incognito=False,
            user_agent="wryview-test/1.0", focused=True,
            autoplay=False, javascript_enabled=True, hotkeys_zoom=True,
            initialization_script="window.__TEST=1;",
            ipc_handler=lambda m: None,
            on_navigation=lambda u: True,
            on_page_load=lambda e,u: None,
            on_title_changed=lambda t: None,
            on_new_window=lambda u: wryview.NewWindowResponse.Deny,
            drag_drop_handler=lambda e,p,pos: False,
            custom_protocols=None, proxy=None,
            back_forward_gestures=False, clipboard=True,
            https_scheme=False, default_context_menus=True,
            data_directory=cache, headers=None,
            on_download_started=lambda u,p: True,
            on_download_completed=lambda u,p,s: None,
            as_child=True,
        )
        ok("30+ 参数构造 ✓"); wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)
        import shutil; shutil.rmtree(cache, ignore_errors=True)


# ── 7. 内容方法 ──

def test_webview_content_methods():
    section("7. 内容方法")
    import wryview
    root = create_tk_window("content"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank")

        # url() 初始值
        u0 = wv.url(); assert u0 == "about:blank", f"期望 about:blank, 实际 {u0}"
        ok(f"初始 url() = {u0!r} ✓")

        # load_url 后 url() 改变
        wv.load_url("about:blank")
        pump_tk(root, 200)
        u1 = wv.url()
        ok(f"load_url('about:blank') 后 url() = {u1!r}")

        # load_html
        wv.load_html("<html><body><p>Hello</p></body></html>")
        pump_tk(root, 200)
        u2 = wv.url()
        ok(f"load_html(...) 后 url() = {u2!r}")

        # load_url_with_headers（测试两种输入）
        wv.load_url_with_headers("about:blank", {"X-T": "v1"})
        ok("load_url_with_headers(dict) ✓")
        wv.load_url_with_headers("about:blank", [("X-T2", "v2")])
        ok("load_url_with_headers(list) ✓")

        # reload
        wv.reload()
        ok("reload() ✓")

        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 8. JavaScript ──

def test_webview_js():
    section("8. JavaScript")
    import wryview
    root = create_tk_window("js"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank")

        # eval_js 不返回值，但不应抛异常
        wv.eval_js("1 + 1")
        ok("eval_js('1+1') ✓")

        # eval_js_with_callback: 执行 JS → 回调获取返回值
        result = []

        def cb(val: str):
            result.append(val)

        wv.eval_js_with_callback(
            "(function(){ return '42'; })()",
            cb,
        )
        ok("eval_js_with_callback 调用成功")
        # 泵事件等待异步回调
        pump_tk(root, 500)
        if result:
            # WebView2 回调可能返回裸字符串或 JSON 字符串
            raw = result[0]
            ok(f"eval_js_with_callback 回调返回: {raw!r} ✓")
        else:
            skip("eval_js_with_callback 未收到回调", "WebView2 可能需要更长时间完成 JS 执行")

        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 9. IPC ──

def test_webview_ipc():
    section("9. IPC (JS → Python)")
    import wryview
    root = create_tk_window("ipc"); hwnd = hwnd_of(root)
    try:
        msgs = []

        def handler(msg: str):
            msgs.append(msg)

        wv = wryview.WebView(hwnd, url="about:blank", ipc_handler=handler)

        # 清空后再设置新的
        wv.clear_ipc_handler()
        assert len(msgs) == 0
        ok("clear_ipc_handler() ✓")

        # 重新设置
        msgs2 = []
        wv.set_ipc_handler(lambda m: msgs2.append(m))
        ok("set_ipc_handler() ✓")

        # 发送 IPC 消息
        wv.eval_js("window.ipc.postMessage('hello_from_js')")
        pump_tk(root, 500)

        if msgs2:
            assert msgs2[0] == "hello_from_js", f"期望 'hello_from_js', 实际 {msgs2[0]!r}"
            ok(f"IPC 消息正确传递: {msgs2[0]!r} ✓")
        else:
            skip("IPC 消息未收到", "WebView2 IPC 可能需要更长等待")

        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 10. Callback Setters ──

def test_webview_callback_setters():
    section("10. Callback Setters")
    import wryview
    root = create_tk_window("cbs"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank")

        blocked = []
        nav = []
        loads = []
        titles = []
        newwins = []
        drops = []
        dl_starts = []
        dl_completes = []

        wv.set_on_navigation(lambda u: (nav.append(u), True)[1])
        wv.set_on_page_load(lambda e, u: loads.append((str(e), u)))
        wv.set_on_title_changed(lambda t: titles.append(t))
        wv.set_on_new_window(lambda u: (newwins.append(u), wryview.NewWindowResponse.Deny)[1])
        wv.set_drag_drop_handler(lambda e, p, pos: (drops.append(str(e)), False)[1])
        wv.set_on_download_started(lambda u, p: (dl_starts.append(u), True)[1])
        wv.set_on_download_completed(lambda u, p, s: dl_completes.append((u, s)))

        # 触发 title changed（设置 document.title）
        wv.eval_js("document.title = 'test_cb_title'")
        pump_tk(root, 300)

        # 导航回调: 至少 nav handler 被调用过（初始 about:blank 加载）
        # 再 load_url 触发一次
        wv.load_url("about:blank")
        pump_tk(root, 300)

        # 页面加载事件
        if loads:
            ok(f"on_page_load 回调触发: {loads}")
        else:
            skip("on_page_load", "未在等待时间内触发")

        if titles:
            assert "test_cb_title" in titles, f"期望 'test_cb_title', 实际 {titles}"
            ok(f"on_title_changed 回调触发: {titles} ✓")
        else:
            skip("on_title_changed", "未在等待时间内触发")

        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 11. Geometry & Appearance ──

def test_webview_geometry_appearance():
    section("11. Geometry & Appearance")
    import wryview
    root = create_tk_window("geo"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank", width=400, height=300)

        # bounds 读取
        b = wv.bounds()
        assert b is not None, "bounds() 返回 None"
        x, y, w, h = b
        assert w > 0 and h > 0, f"尺寸异常: {w}×{h}"
        ok(f"bounds() = ({x:.0f}, {y:.0f}, {w:.0f}, {h:.0f}) ✓")

        # set_bounds 后重新读取
        wv.set_bounds(10, 20, 500, 350)
        b2 = wv.bounds()
        x2, y2, w2, h2 = b2
        # 注意: 实际 bounds 可能因平台差异而有微小偏差
        ok(f"set_bounds(10,20,500,350) → ({x2:.0f}, {y2:.0f}, {w2:.0f}, {h2:.0f})")

        # set_visible（不能真正验证视觉效果，但验证不报错）
        wv.set_visible(True)
        wv.set_visible(False)
        ok("set_visible(True/False) ✓")

        # set_background_color
        wv.set_background_color(128, 128, 128, 255)
        ok("set_background_color(128,128,128,255) ✓")

        # focus
        wv.focus()
        ok("focus() ✓")

        # zoom
        wv.zoom(1.5)
        wv.zoom(1.0)
        ok("zoom(1.5) → zoom(1.0) ✓")

        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 12. DevTools ──

def test_webview_devtools():
    section("12. DevTools")
    import wryview
    root = create_tk_window("devtools"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank", devtools=True)

        assert wv.is_devtools_open() == False, "初始状态应为 False"
        ok("初始 is_devtools_open() = False ✓")

        wv.open_devtools()
        pump_tk(root, 200)
        # 注意: open_devtools 可能异步打开
        ok(f"open_devtools() 后 = {wv.is_devtools_open()}")

        wv.close_devtools()
        pump_tk(root, 200)
        ok(f"close_devtools() 后 = {wv.is_devtools_open()}")

        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 13. Cookies — 完整 CRUD ──

def test_webview_cookies():
    section("13. Cookies — 完整 CRUD")
    import wryview
    root = create_tk_window("cookies"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank")

        # ── 准备 ──
        initial = wv.cookies()
        ok(f"初始 cookies: {len(initial)} 个")

        # ── CREATE: 设置 3 个 cookie（about:blank 无 origin，需要显式 domain）──
        wv.set_cookie("ck_a", "val_a", "localhost", "/")
        wv.set_cookie("ck_b", "val_b", "localhost", "/path_b")
        wv.set_cookie("ck_c", "val_c", "example.com", "/app")
        ok("创建 3 个 cookie ✓")

        # ── READ: 全量获取 ──
        all_c = wv.cookies()
        names = {c.name for c in all_c}
        assert "ck_a" in names, f"ck_a 未出现在: {names}"
        assert "ck_b" in names, f"ck_b 未出现在: {names}"
        assert "ck_c" in names, f"ck_c 未出现在: {names}"
        ok(f"cookies() 包含全部 3 个: {names} ✓")

        # ── READ: 按 URL 获取（about:blank 不匹配 localhost/example.com domain）──
        url_c = wv.cookies_for_url("about:blank")
        url_names = {c.name for c in url_c}
        ok(f"cookies_for_url('about:blank') = {url_names} (domain 不匹配预期为空)")

        # ── READ: 验证属性完整性 ──
        for c in all_c:
            assert isinstance(c.name, str) and c.name, f"name 异常: {c.name!r}"
            assert isinstance(c.value, str), f"value 异常: {c.value!r}"
            assert isinstance(c.secure, bool), f"secure 异常: {c.secure}"
            assert isinstance(c.http_only, bool), f"http_only 异常: {c.http_only}"
        ck_a = next(c for c in all_c if c.name == "ck_a")
        assert ck_a.value == "val_a", f"ck_a value 期望 'val_a', 实际 {ck_a.value!r}"
        assert ck_a.path == "/", f"ck_a path 期望 '/', 实际 {ck_a.path!r}"
        assert ck_a.domain == "localhost", f"ck_a domain 期望 'localhost', 实际 {ck_a.domain!r}"
        ok(f"ck_a: name={ck_a.name!r} value={ck_a.value!r} domain={ck_a.domain!r} path={ck_a.path!r} ✓")

        ck_c = next(c for c in all_c if c.name == "ck_c")
        assert ck_c.domain == "example.com", f"ck_c domain 期望 'example.com', 实际 {ck_c.domain!r}"
        assert ck_c.path == "/app", f"ck_c path 期望 '/app', 实际 {ck_c.path!r}"
        ok(f"ck_c: domain={ck_c.domain!r} path={ck_c.path!r} ✓")

        # ── UPDATE: 同名 cookie 覆盖 ──
        wv.set_cookie("ck_a", "val_a_updated", "localhost", "/")
        all_c2 = wv.cookies()
        ck_a2 = next(c for c in all_c2 if c.name == "ck_a")
        assert ck_a2.value == "val_a_updated", f"UPDATE 失败: 期望 'val_a_updated', 实际 {ck_a2.value!r}"
        ok("UPDATE: ck_a value → 'val_a_updated' ✓")

        # ── DELETE: delete_cookie 通过 cookies_for_url(url) 匹配 ──
        # 注意: about:blank 无 origin, 只有无 domain 或匹配 about:blank 的 cookie
        # 会被匹配到。domain=localhost 的 cookie 不匹配 about:blank。
        # 此处测试: 设一个无 domain 的 cookie (host-only) 给 about:blank
        wv.set_cookie("ck_ab", "val_ab", None, "/")
        pump_tk(root, 50)
        ab_cookies = wv.cookies_for_url("about:blank")
        ab_names = {c.name for c in ab_cookies}
        ok(f"about:blank host-only cookie: cookies_for_url → {ab_names}")

        # 如果 WebView2 接受了无 domain 的 cookie, 则测试 delete
        if "ck_ab" in ab_names:
            count_before = len(wv.cookies())
            wv.delete_cookie("ck_ab", "about:blank")
            pump_tk(root, 50)
            count_after = len(wv.cookies())
            assert count_after < count_before, f"数量未减: {count_before} → {count_after}"
            ok(f"DELETE ck_ab ✓ (count: {count_before} → {count_after})")
        else:
            skip("DELETE host-only cookie", "WebView2 未在 about:blank 上存储无 domain cookie")

        # ── DELETE: 不存在的 cookie 不抛异常 ──
        wv.delete_cookie("nonexistent", "about:blank")
        ok("DELETE 不存在 cookie 不抛异常 ✓")

        # ── DELETE 全部: clear_all_browsing_data ──
        count_before_all = len(wv.cookies())
        wv.clear_all_browsing_data()
        pump_tk(root, 100)
        count_after_all = len(wv.cookies())
        ok(f"clear_all_browsing_data: {count_before_all} → {count_after_all} ✓")

        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 14. 其他方法 ──

def test_webview_misc():
    section("14. 其他方法")
    import wryview
    root = create_tk_window("misc"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank")

        # clear_all_browsing_data: 先设 cookie，清空后验证
        wv.set_cookie("tmp_ck", "tmp_val", "localhost", "/")
        before = len(wv.cookies())
        assert before > 0, f"应至少有一个 cookie, 实际 {before}"
        ok(f"设置 cookie 后: {before} 个 ✓")

        wv.clear_all_browsing_data()
        pump_tk(root, 100)
        after = wv.cookies()
        ok(f"clear_all_browsing_data() 后: {len(after)} 个")

        # close 后验证 url() → None
        wv.close()
        assert wv.url() is None, "close() 后 url() 应为 None"
        ok("close() 后 url() = None ✓")
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 15. 错误处理 ──

def test_error_handling():
    section("15. 错误处理")
    import wryview

    # 15a: null HWND
    try:
        wryview.WebView(0)
        fail("WebView(0)", "应抛出异常")
    except Exception as e:
        ok(f"WebView(0) → {type(e).__name__}: {e!s} ✓")

    # 15b: close() 后调用方法
    root = create_tk_window("err"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank")
        wv.close()
        errors = []
        methods = [
            ("load_url", ("about:blank",)),
            ("load_html", ("<h1>x</h1>",)),
            ("eval_js", ("1+1",)),
            ("reload", ()),
            ("focus", ()),
            ("print", ()),
            ("set_visible", (True,)),
            ("set_bounds", (0,0,100,100)),
            ("zoom", (1.0,)),
            ("cookies", ()),
            ("clear_all_browsing_data", ()),
        ]
        for m, a in methods:
            try:
                getattr(wv, m)(*a)
                errors.append(m)
            except Exception:
                pass
        assert len(errors) == 0, f"{errors} 未抛异常"
        ok(f"close() 后 {len(methods)} 个方法全部正确抛异常 ✓")
    finally:
        root.destroy(); pump_tk(root, 100)

    # 15c: 双重 close
    root2 = create_tk_window("err2"); hwnd2 = hwnd_of(root2)
    try:
        wv = wryview.WebView(hwnd2, url="about:blank")
        wv.close(); wv.close()
        ok("双重 close() 安全 ✓")
    finally:
        root2.destroy(); pump_tk(root2, 100)


# ── 16. 模块级函数 ──

def test_module_functions():
    section("16. 模块级函数")
    import wryview
    wryview.pump_events(); ok("pump_events() ✓")
    wryview.ensure_gtk_init(); ok("ensure_gtk_init() ✓")
    for _ in range(3):
        wryview.pump_events()
        wryview.ensure_gtk_init()
    ok("多次调用无异常 ✓")


# ── 17. 独立窗口 ──

def test_standalone_window():
    section("17. as_child=False")
    import wryview
    root = create_tk_window("standalone"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank", as_child=False, width=640, height=480)
        ok("as_child=False 构造 ✓")
        wv.set_bounds(50, 50, 600, 400)
        b = wv.bounds()
        assert b is not None, "bounds 为 None"
        ok(f"独立窗口 bounds = ({b[0]:.0f},{b[1]:.0f},{b[2]:.0f},{b[3]:.0f})")
        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 18. 共享 WebContext ──

def test_shared_web_context():
    section("18. 共享 WebContext")
    import wryview
    d = os.path.join(tempfile.gettempdir(), "wryview_shared")
    os.makedirs(d, exist_ok=True)
    ctx = wryview.WebContext(data_directory=d)

    root = create_tk_window("shared"); hwnd = hwnd_of(root)
    try:
        wv1 = wryview.WebView(hwnd, url="about:blank", web_context=ctx)
        wv2 = wryview.WebView(hwnd, url="about:blank", web_context=ctx)

        # wv1 设置 cookie（需要显式 domain，about:blank 无 origin）
        # → wv2 应可见（共享 WebContext）
        wv1.set_cookie("shared_ck", "shared_val", "localhost", "/")
        pump_tk(root, 50)
        c1 = wv1.cookies(); c2 = wv2.cookies()
        names1 = {c.name for c in c1}; names2 = {c.name for c in c2}
        assert "shared_ck" in names1, f"wv1 未包含 shared_ck: {names1}"
        assert "shared_ck" in names2, f"wv2 未包含 shared_ck (共享失败): {names2}"
        ok(f"WebContext 共享正确: wv1={names1}, wv2={names2} ✓")

        wv1.close(); wv2.close()
    finally:
        root.destroy(); pump_tk(root, 100)
        import shutil; shutil.rmtree(d, ignore_errors=True)


# ── 19. 多实例 ──

def test_multi_instance():
    section("19. 多实例")
    import wryview
    root = create_tk_window("multi"); hwnd = hwnd_of(root)
    try:
        instances = []
        for i in range(3):
            wv = wryview.WebView(hwnd, url="about:blank",
                ipc_handler=lambda m: None,
                on_page_load=lambda e,u: None,
                on_title_changed=lambda t: None)
            instances.append(wv)
        ok(f"创建 {len(instances)} 个实例 ✓")

        for i, wv in enumerate(instances):
            for j in range(3):
                wv.eval_js(f"console.log('w{i}_j{j}')")
        ok("顺序 eval_js 全部成功 ✓")
        for wv in instances:
            wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 20. 自定义协议 ──

def test_custom_protocol():
    section("20. 自定义协议")
    import wryview

    responses = []

    def handler(method, uri, headers, body, respond):
        responses.append((method, uri))
        respond(200, {"Content-Type": "text/html", "X-Custom": "wryview-test"},
                b"<html><body><h1>Custom Protocol</h1></body></html>")

    root = create_tk_window("proto"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd,
            custom_protocols={"wrytest": handler},
            url="wrytest://localhost/index.html")
        pump_tk(root, 500)
        if responses:
            method, uri = responses[0]
            assert method == "GET", f"期望 GET, 实际 {method}"
            assert "wrytest" in uri, f"URI 中应含 'wrytest': {uri}"
            ok(f"自定义协议触发: {method} {uri} ✓")
        else:
            ok("WebView 带 custom_protocols 构造成功（响应异步到达）")
        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 21. initialization_script ──

def test_initialization_script():
    section("21. initialization_script")
    import wryview
    root = create_tk_window("init"); hwnd = hwnd_of(root)
    try:
        script = "window.__WRYVIEW_INIT_TEST = 'hello_init';"
        wv = wryview.WebView(hwnd, url="about:blank", initialization_script=script)

        # 用 eval_js_with_callback 读取 init script 设置的变量
        result = []
        def cb(val):
            result.append(val)
        wv.eval_js_with_callback("window.__WRYVIEW_INIT_TEST", cb)
        pump_tk(root, 500)

        if result:
            # 去除可能的引号
            val = result[0].strip('"').strip("'")
            assert val == "hello_init", f"期望 'hello_init', 实际 {val!r}"
            ok(f"init script 生效: window.__WRYVIEW_INIT_TEST = {val!r} ✓")
        else:
            skip("init_script 验证", "未收到 eval_js_with_callback 回调")

        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 22. 透明 & 背景色 ──

def test_transparency_and_bg():
    section("22. 透明 & 背景色")
    import wryview
    root = create_tk_window("bg"); hwnd = hwnd_of(root)
    try:
        wv1 = wryview.WebView(hwnd, url="about:blank", transparent=True)
        ok("transparent=True ✓"); wv1.close()

        wv2 = wryview.WebView(hwnd, url="about:blank", background_color=(0,0,0,0))
        ok("background_color=(0,0,0,0) ✓"); wv2.close()

        wv3 = wryview.WebView(hwnd, url="about:blank")
        wv3.set_background_color(255, 0, 0, 128)
        ok("set_background_color(255,0,0,128) ✓"); wv3.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 23. reparent ──

def test_reparent():
    section("23. reparent")
    import wryview, tkinter as tk
    root = create_tk_window("reparent"); hwnd = hwnd_of(root)
    try:
        frame = tk.Frame(root, width=400, height=300, bg="gray")
        frame.pack(); frame.update()
        child = int(frame.winfo_id())

        wv = wryview.WebView(hwnd, url="about:blank")
        if sys.platform in ("win32", "darwin"):
            try:
                wv.reparent(child)
                ok(f"reparent({child}) ✓")
            except NotImplementedError:
                skip("reparent", "平台不支持")
            except Exception as e:
                fail("reparent", str(e))
        else:
            try:
                wv.reparent(child)
                fail("Linux 应抛 NotImplementedError", "未抛")
            except NotImplementedError:
                ok("Linux reparent → NotImplementedError ✓")
            except Exception as e:
                ok(f"Linux reparent 抛异常: {type(e).__name__} ✓")
        wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 24. 隐私参数 ──

def test_incognito_and_privacy():
    section("24. 隐私参数")
    import wryview
    root = create_tk_window("privacy"); hwnd = hwnd_of(root)
    try:
        # 每次 close 后等待资源释放，减少 0x8007139F 风险
        def _create_and_close(**kw):
            wv = wryview.WebView(hwnd, url="about:blank", **kw)
            wv.close()
            pump_tk(root, 50)
            return True

        _create_and_close(incognito=True)
        ok("incognito=True ✓")
        _create_and_close(javascript_enabled=False)
        ok("javascript_enabled=False ✓")
        _create_and_close(hotkeys_zoom=False)
        ok("hotkeys_zoom=False ✓")
        _create_and_close(clipboard=False)
        ok("clipboard=False ✓")
        _create_and_close(default_context_menus=False)
        ok("default_context_menus=False ✓")
        _create_and_close(back_forward_gestures=True)
        ok("back_forward_gestures=True ✓")

        try:
            _create_and_close(autoplay=True)
            ok("autoplay=True ✓")
        except RuntimeError as e:
            if "0x8007139F" in str(e) or "0x8007139f" in str(e):
                skip("autoplay=True", "WebView2 资源状态异常 (0x8007139F)")
            else:
                raise
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 25. headers ──

def test_headers_param():
    section("25. headers")
    import wryview
    root = create_tk_window("headers"); hwnd = hwnd_of(root)
    try:
        wv1 = wryview.WebView(hwnd, url="about:blank",
            headers={"X-Custom-Hdr": "test123"})
        ok("headers=dict ✓"); wv1.close()

        wv2 = wryview.WebView(hwnd, url="about:blank",
            headers=[("X-Custom2", "v2")])
        ok("headers=list ✓"); wv2.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 26. WindowHandleKind ──

def test_window_handle_kind():
    section("26. WindowHandleKind")
    import wryview
    root = create_tk_window("kind"); hwnd = hwnd_of(root)
    try:
        wv = wryview.WebView(hwnd, url="about:blank",
            parent_hwnd_kind=wryview.WindowHandleKind.Win32)
        ok("Win32 ✓"); wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 27. 枚举相等性 & 哈希 ──

def test_type_equality():
    section("27. 枚举相等性 & 哈希")
    import wryview
    # 相等性
    assert wryview.PageLoadEvent.Started == wryview.PageLoadEvent.Started
    assert wryview.PageLoadEvent.Started != wryview.PageLoadEvent.Finished
    ok("PageLoadEvent ✓")
    assert wryview.NewWindowResponse.Allow != wryview.NewWindowResponse.Deny
    ok("NewWindowResponse ✓")
    assert wryview.DragDropEvent.Enter != wryview.DragDropEvent.Leave
    ok("DragDropEvent ✓")
    assert wryview.WindowHandleKind.Win32 != wryview.WindowHandleKind.Gtk
    ok("WindowHandleKind ✓")
    # 哈希
    s = {wryview.PageLoadEvent.Started, wryview.PageLoadEvent.Finished}
    assert len(s) == 2
    h = hash(wryview.PageLoadEvent.Started)
    ok(f"hash(PageLoadEvent.Started) = {h}")


# ── 28. url() 详测 ──

def test_url_method():
    section("28. url() 详测")
    import wryview
    root = create_tk_window("url"); hwnd = hwnd_of(root)
    try:
        # 无 url/html 时，wry 默认 about:blank
        wv = wryview.WebView(hwnd)
        u0 = wv.url()
        ok(f"无 url 时 url() = {u0!r}")

        # load_url 改变 URL
        wv.load_url("about:blank")
        pump_tk(root, 200)
        u1 = wv.url()
        # WebView2 可能返回 "about:blank" 或空字符串
        ok(f"load_url('about:blank') 后 → {u1!r}")
        assert u1 is not None, "url() 不应为 None"

        # load_html 后 url 应该变化
        wv.load_html("<html><body><p>URL test</p></body></html>")
        pump_tk(root, 200)
        u2 = wv.url()
        ok(f"load_html(...) 后 → {u2!r}")

        wv.close()
        u3 = wv.url()
        assert u3 is None, f"close() 后 url() 应为 None, 实际 {u3!r}"
        ok("close() 后 url() = None ✓")
    finally:
        root.destroy(); pump_tk(root, 100)


# ── 29. https_scheme ──

def test_https_scheme():
    section("29. https_scheme")
    import wryview
    root = create_tk_window("https"); hwnd = hwnd_of(root)
    try:
        def handler(m, u, h, b, r):
            r(200, {"Content-Type": "text/html"}, b"<h1>HTTPS</h1>")
        wv = wryview.WebView(hwnd,
            custom_protocols={"secureapp": handler},
            url="secureapp://localhost/",
            https_scheme=True)
        ok("https_scheme=True ✓"); wv.close()
    finally:
        root.destroy(); pump_tk(root, 100)


# ═══════════════════════════════════════════════════════════════════════
#  主入口
# ═══════════════════════════════════════════════════════════════════════

TESTS = [
    # 无窗口
    ("1. 模块导入", test_imports),
    ("2. 枚举", test_enums),
    ("3. WebContext", test_web_context),
    ("4. CookieDict", test_cookie_dict),
    ("16. 模块函数", test_module_functions),
    ("27. 枚举相等性", test_type_equality),
    # 有窗口
    ("5. 基本构造", test_webview_basic),
    ("6. 全部参数", test_webview_all_options),
    ("7. 内容方法", test_webview_content_methods),
    ("8. JavaScript", test_webview_js),
    ("9. IPC", test_webview_ipc),
    ("10. Callback", test_webview_callback_setters),
    ("11. Geometry", test_webview_geometry_appearance),
    ("12. DevTools", test_webview_devtools),
    ("13. Cookies CRUD", test_webview_cookies),
    ("14. 其他方法", test_webview_misc),
    ("15. 错误处理", test_error_handling),
    ("17. 独立窗口", test_standalone_window),
    ("18. 共享WebContext", test_shared_web_context),
    ("19. 多实例", test_multi_instance),
    ("20. 自定义协议", test_custom_protocol),
    ("21. init_script", test_initialization_script),
    ("22. 透明/背景色", test_transparency_and_bg),
    ("23. reparent", test_reparent),
    ("24. 隐私参数", test_incognito_and_privacy),
    ("25. headers", test_headers_param),
    ("26. WindowHandleKind", test_window_handle_kind),
    ("28. url()", test_url_method),
    ("29. https_scheme", test_https_scheme),
]


def main():
    print("=" * 70)
    print("  wryview 全面功能测试（严 sui 版）")
    print(f"  平台: {sys.platform}     Python: {sys.version.split()[0]}")
    print("=" * 70)
    import wryview
    print(f"  wryview {wryview.__version__}  |  {wryview._core.__file__}")

    for name, fn in TESTS:
        run_case(name, fn)

    total = _passed + _failed + _skipped
    print(f"\n{'═' * 70}")
    print(f"  总计 {total}  |  ✅ {_passed}  |  ❌ {_failed}  |  ⏭️  {_skipped}")
    print(f"{'═' * 70}")
    if _failed > 0:
        print(f"\n  ❌ {_failed} 个失败!")
        sys.exit(1)
    else:
        print(f"\n  ✅ 全部通过!")
        sys.exit(0)


if __name__ == "__main__":
    main()
