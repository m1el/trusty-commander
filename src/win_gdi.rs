extern crate winapi;
extern crate std;
extern crate user32;

use ::messages;
use winapi::*;
use win_layer::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[allow(dead_code)]
struct DebugBlock {
    message: &'static str,
}

macro_rules! debug { ( $( $x:expr ),* ) => {} }

impl DebugBlock {
    #[allow(dead_code)]
    pub fn start(msg: &'static str) -> DebugBlock {
        debug!("--> {}", msg);
        DebugBlock { message: msg }
    }
}
impl Drop for DebugBlock {
    fn drop(&mut self) {
        debug!("<-- {}", self.message);
    }
}
macro_rules! dbg_block {
    ($x:expr) => { let _guard = DebugBlock::start($x); }
}


type RcRc<T> = Rc<RefCell<T>>;
#[inline]
fn rcrc<T>(x: T) -> RcRc<T>
{ Rc::new(RefCell::new(x)) }

type HwndMap = HashMap<HWND, Box<Any>>;
thread_local! {
    static HWND_TABLE: RefCell<HwndMap> = RefCell::new(HashMap::new());
    static ATOM_TABLE: RefCell<HashMap<TypeId, ATOM>> = RefCell::new(HashMap::new());
}

fn lookup_hwnd<T>(hwnd: HWND)
    -> Option<RcRc<T>>
    where T: WinCls
{
    HWND_TABLE.with(|t| {
        let t = t.borrow();
        t.get(&hwnd)
            .and_then(|box_any| box_any.downcast_ref::<RcRc<T>>().map(|x|x.clone()))
    })
}

fn insert_hwnd<T>(hwnd: HWND, para: LPARAM)
    -> Option<RcRc<T>>
    where T: WinCls
{
    let rv_box;
    let box_any: Box<Any>;
    unsafe {
        let create_ptr = para as *const CREATESTRUCTW;
        let create_param = (*create_ptr).lpCreateParams;
        if create_param as usize == 0 { return None; }
        rv_box = Box::from_raw(create_param as *mut RcRc<T>);
        box_any = rv_box.clone();
    };
    HWND_TABLE.with(|t| t.borrow_mut().insert(hwnd, box_any));
    Some(*rv_box)
}

fn remove_hwnd<T>(hwnd: HWND)
    -> Option<RcRc<T>>
    where T: WinCls
{
    HWND_TABLE.with(|t| {
        t.borrow_mut().remove(&hwnd)
            .and_then(|box_any| box_any.downcast_ref::<RcRc<T>>().map(|x|x.clone()))
    })
}

pub trait WinCls: Sized + 'static {
    fn wnd_proc(
        &self,
        _hwnd: HWND, _msg: UINT,
        _param: WPARAM, _para: LPARAM)
        -> Option<LRESULT>
    { None }

    fn wnd_proc_static(
        _hwnd: HWND,
        _msg: UINT,
        _param: WPARAM,
        _para: LPARAM)
        -> Option<LRESULT>
    { None }

    extern "system"
    fn wnd_proc_raw(
        hwnd: HWND,
        msg: UINT,
        param: WPARAM,
        para: LPARAM)
        -> LRESULT
    {
        debug!("wnd_proc_raw {}", messages::msg_name(msg));
        if msg == WM_NCDESTROY {
            let orv = remove_hwnd::<Self>(hwnd)
                        .and_then(|removed| removed.borrow().wnd_proc(hwnd, msg, param, para));
            if let Some(rv) = orv {
                return rv;
            }
        }

        if let Some(rv) = Self::wnd_proc_static(hwnd, msg, param, para) {
            return rv;
        }

        let mut target = lookup_hwnd::<Self>(hwnd);
        if target.is_none() && msg == WM_CREATE {
            target = insert_hwnd::<Self>(hwnd, para);
        }

        target.and_then(|inst| inst.borrow().wnd_proc(hwnd, msg, param, para))
              .unwrap_or_else(|| DefWindowProcW(hwnd, msg, param, para))
    }

    fn register() -> Result<ATOM, u32>
    { Err(0xffffffff) }

    fn get_cls_id() -> Result<WinClsIdW, u32>
    {
        ATOM_TABLE.with(|t| {
            let mut t = t.borrow_mut();
            let type_id = TypeId::of::<Self>();
            if let Some(atom) = t.get(&type_id).map(|a|*a) {
                Ok(WinClsIdW::Atom(atom))
            } else {
                let atom = try!(Self::register());
                t.insert(type_id, atom);
                Ok(WinClsIdW::Atom(atom))
            }
        })
    }
}

pub struct MainCls {
    panel1: HWND,
    panel2: HWND,
}

impl MainCls {
    pub fn create(instance: HINSTANCE) -> Result<HWND, u32>
    {
        let inst = MainCls { panel1: 0 as HWND, panel2: 0 as HWND };
        let cls_id = try!(Self::get_cls_id());
        let inst_rc = rcrc(inst);
        let inst_ptr = Box::into_raw(Box::new(inst_rc.clone()) as Box<Any>);
        let hwnd = try!(CreateWindowExW(
            WS_EX_CLIENTEDGE,
            cls_id,
            Some(&wstr("Trusty Commander")),
            WS_TILEDWINDOW,
            CW_USEDEFAULT, CW_USEDEFAULT, 1024, 768,
            None, None, instance, Some(inst_ptr as LPVOID)));

        let panel1 = try!(FilePanelCls::create(instance, hwnd));
        let panel2 = try!(FilePanelCls::create(instance, hwnd));

        let mut inst = inst_rc.borrow_mut();
        inst.panel1 = panel1;
        inst.panel2 = panel2;
        Ok(hwnd)
    }
}

impl WinCls for MainCls {
    fn register () -> Result<ATOM, u32>
    {
        let wnd_cls = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc_raw),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as HINSTANCE,
            hIcon: try!(LoadIconW(0 as HINSTANCE, RC_IDI_APPLICATION)),
            hCursor: try!(LoadCursorW(0 as HINSTANCE, RC_IDC_ARROW)),
            hbrBackground: (winapi::COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: 0 as *const u16,
            lpszClassName: wstr("MainCls").as_ptr(),
            hIconSm: try!(LoadIconW(0 as HINSTANCE, RC_IDI_APPLICATION)),
        };

        RegisterClassExW(&wnd_cls)
    }

    fn wnd_proc_static(
        _hwnd: HWND, msg: UINT,
        _param: WPARAM, para: LPARAM)
        -> Option<LRESULT>
    {
        match msg {
            WM_GETMINMAXINFO => {
                let mmi = unsafe { &mut*(para as *mut MINMAXINFO) };
                mmi.ptMinTrackSize.x = 400;
                mmi.ptMinTrackSize.y = 200;
                Some(0)
            },
            _ => None,
        }
    }

    fn wnd_proc(
        &self,
        hwnd: HWND, msg: UINT,
        _param: WPARAM, para: LPARAM)
        -> Option<LRESULT>
    {
        match msg {
            WM_SIZE => {
                let rv = GetClientRect(hwnd).and_then(|rect| {
                    let width = rect.right - rect.left;
                    let height = rect.bottom - rect.top;
                    let panel_width = width / 2 - 2;
                    let panel2x = width - panel_width;
                    debug!("hwnd: {}, w: {}, h: {}", hwnd as usize, width, height);
                    let flags = SWP_NOACTIVATE | SWP_NOZORDER;
                    let mut hdwp = try!(BeginDeferWindowPos(2));
                    hdwp = try!(DeferWindowPos(hdwp, self.panel1, None, 0, 0, panel_width, height, flags));
                    hdwp = try!(DeferWindowPos(hdwp, self.panel2, None, panel2x, 0, panel_width, height, flags));
                    try!(EndDeferWindowPos(hdwp));
                    try!(InvalidateRect(hwnd, &rect, true));
                    Ok(())
                });
                match rv {
                    Ok(_) => Some(0),
                    Err(x) => Some(x as LRESULT),
                }
            },
            WM_PAINT => {
                let rv = BeginPaint(hwnd).and_then(|(ps, hdc)| {
                    let rect = try!(GetClientRect(hwnd));
                    try!(FillRect(hdc, &rect, (COLOR_WINDOW + 2) as HBRUSH));
                    EndPaint(hwnd, &ps)
                });
                match rv {
                    Ok(_) => Some(1),
                    Err(x) => Some(x as LRESULT),
                }
            },
            WM_DRAWITEM => {
                debug!("draw? :(");
                let draw_struct = unsafe { &*(para as *const DRAWITEMSTRUCT) };
                debug!("btnid: {}", draw_struct.hwndItem as usize);
                let rect = GetClientRect(draw_struct.hwndItem).unwrap();
                FillRect(draw_struct.hDC, &rect, (COLOR_WINDOW + 0) as HBRUSH).unwrap();
                Some(1)
            },
            WM_DESTROY => { PostQuitMessage(0); Some(0) },
            _ => None,
        }
    }
}

/*
pub struct ButtonCls {
}

impl ButtonCls {
    pub fn create(instance: HINSTANCE, parent: HWND) -> Result<HWND, u32>
    {
        CreateWindowExW(
            0,
            try!(Self::get_cls_id()),
            Some(&wstr("boop")),
            WS_VISIBLE | WS_CHILD | BS_FLAT,
            10, 10, 100, 100,
            Some(parent), None, instance, None)
    }
}

impl WinCls for ButtonCls {
    fn get_cls_id() -> Result<WinClsIdW, u32>
    {
        Ok(WinClsIdW::ClsName(wstr("BUTTON")))
    }
}
*/

pub struct FilePanelCls {
}
impl FilePanelCls {
    pub fn create(instance: HINSTANCE, parent: HWND) -> Result<HWND, u32>
    {
        let inst = FilePanelCls { };
        let cls_id = try!(Self::get_cls_id());
        let inst_rc = rcrc(inst);
        let inst_ptr = Box::into_raw(Box::new(inst_rc) as Box<Any>);
        let hwnd = try!(CreateWindowExW(
            0,
            cls_id,
            Some(&wstr("boop")),
            WS_VISIBLE | WS_CHILD | WS_BORDER,
            10, 10, 100, 100,
            Some(parent), None, instance, Some(inst_ptr as LPVOID)));

        Ok(hwnd)
    }
}
impl WinCls for FilePanelCls {
    fn wnd_proc(
        &self,
        hwnd: HWND, msg: UINT,
        _param: WPARAM, _para: LPARAM)
        -> Option<LRESULT>
    {
        match msg {
            WM_PAINT => {
                let rv = BeginPaint(hwnd).and_then(|(ps, hdc)| {
                    let rect = try!(GetClientRect(hwnd));
                    try!(FillRect(hdc, &rect, (COLOR_WINDOW + 0) as HBRUSH));
                    try!(SetBkMode(hdc, TRANSPARENT));
                    try!(TextOutW(hdc, 0, 0, &wstr("test")));
                    let rect = RECT { left: 0, right: 200, top: 0, bottom: 20 };
                    try!(DrawFocusRect(hdc, &rect));
                    EndPaint(hwnd, &ps)
                });
                match rv {
                    Ok(_) => Some(1),
                    Err(x) => Some(x as LRESULT),
                }
            },
            _ => None,
        }
    }
    fn register () -> Result<ATOM, u32>
    {
        let wnd_cls = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc_raw),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as HINSTANCE,
            hIcon: try!(LoadIconW(0 as HINSTANCE, RC_IDI_APPLICATION)),
            hCursor: try!(LoadCursorW(0 as HINSTANCE, RC_IDC_ARROW)),
            hbrBackground: (winapi::COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: 0 as *const u16,
            lpszClassName: wstr("FilePanelCls").as_ptr(),
            hIconSm: try!(LoadIconW(0 as HINSTANCE, RC_IDI_APPLICATION)),
        };

        RegisterClassExW(&wnd_cls)
    }
}
