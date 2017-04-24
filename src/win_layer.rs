#![allow(non_snake_case)]
#![allow(dead_code)]
extern crate winapi;
extern crate kernel32;
extern crate user32;
extern crate gdi32;
extern crate std;

use winapi::*;

extern "system"
{
    #[link_name = "BeginDeferWindowPos"]
    fn user32_BeginDeferWindowPos(nNumWindows: c_int) -> HDWP;
}

#[allow(dead_code)]
#[inline]
pub fn ExitProcess(code: c_uint) -> !
{ unsafe { kernel32::ExitProcess(code) }; loop {} }

#[inline]
pub fn GetLastError() -> u32
{ unsafe { kernel32::GetLastError() } }

#[inline]
pub fn GetModuleHandleW(name: Option<&[u16]>) -> Result<HMODULE, u32>
{
    let handle = unsafe { kernel32::GetModuleHandleW(name.map_or(0 as LPCWSTR, |x|x.as_ptr() as LPCWSTR)) };

    if handle as usize != 0 { Ok(handle) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn GetModuleHandleA(name: Option<&[u8]>) -> Result<HMODULE, u32>
{
    let handle = unsafe { kernel32::GetModuleHandleA(name.map_or(0 as LPCSTR, |x|x.as_ptr() as LPCSTR)) };
    if handle as usize != 0 { Ok(handle) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn GetStdHandle(handle_id: u32) -> Result<Option<HANDLE>, u32>
{
    let handle = unsafe { kernel32::GetStdHandle(handle_id) };

    if handle as usize == 0 { Ok(None) }
    else if handle != INVALID_HANDLE_VALUE { Ok(Some(handle)) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn WriteFile(
    handle: HANDLE,
    buffer: &[u8],
    overlapped: Option<&mut OVERLAPPED>)
    -> Result<u32, u32>
{
    //assert!(buffer.len() <= u32::max_value() as usize, "buffer length has to fit into u32");
    let buffer_len = buffer.len() as u32;
    let mut written: u32 = 0u32;
    let result = unsafe {
        let buffer_ptr = buffer.as_ptr() as *const winapi::c_void;
        let written_ptr = &mut written as *mut u32;
        let overlapped_ptr = overlapped.map_or(0 as *mut OVERLAPPED, |x|x as *mut OVERLAPPED);
        kernel32::WriteFile(handle, buffer_ptr, buffer_len, written_ptr, overlapped_ptr)
    };

    if result != 0 { Ok(written) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn GetConsoleWindow() -> Option<HWND>
{
    let handle = unsafe { kernel32::GetConsoleWindow() };

    if handle as usize != 0 { Some(handle) }
    else { None }
}

#[inline]
pub fn wstr(s: &str) -> Vec<u16>
{
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    let mut rv: Vec<_> = OsStr::new(s).encode_wide().collect();
    // zero terminator
    if rv.last().map_or(true, |c|*c != 0) {
        rv.push(0);
    }
    rv
}

#[inline]
pub fn from_wstr(w: &[u16]) -> String
{
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(w).to_string_lossy().to_string()
}

#[inline]
pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> bool
{
    unsafe { user32::ShowWindow(hWnd, nCmdShow) != 0 }
}

#[inline]
pub fn UpdateWindow(hwnd: HWND) -> Result<(), u32>
{
    let result = unsafe { user32::UpdateWindow(hwnd) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn MessageBoxA(
    hwnd: Option<HWND>,
    message: &[u8],
    title: &[u8],
    flags: c_uint)
    -> Result<c_uint, u32>
{
    let hwnd_ptr = hwnd.unwrap_or(0 as HWND);
    let result = unsafe { user32::MessageBoxA(
            hwnd_ptr,
            message.as_ptr() as *const i8,
            title.as_ptr() as *const i8,
            flags) };

    if result != 0 { Ok(result as c_uint) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn MessageBoxW(
    hwnd: Option<HWND>,
    message: &[u16],
    title: &[u16],
    flags: c_uint)
    -> Result<c_uint, u32>
{
    let hwnd_ptr = hwnd.unwrap_or(0 as HWND);
    let result = unsafe { user32::MessageBoxW(
            hwnd_ptr,
            message.as_ptr() as *const u16,
            title.as_ptr() as *const u16,
            flags) };

    if result != 0 { Ok(result as c_uint) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn FindAtomW(wnd_cls: &[u16])
    -> Result<ATOM, u32>
{
    let result = unsafe { kernel32::FindAtomW(wnd_cls.as_ptr() as LPCWSTR) };

    if result != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn GlobalFindAtomW(wnd_cls: &[u16])
    -> Result<ATOM, u32>
{
    let result = unsafe { kernel32::GlobalFindAtomW(wnd_cls.as_ptr() as LPCWSTR) };

    if result != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

pub enum ResourceIdW<'a> {
    Int(WORD),
    Str(&'a[u16]),
}
pub const RC_IDI_APPLICATION: ResourceIdW<'static> = ResourceIdW::Int(32512);
pub const RC_IDC_ARROW: ResourceIdW<'static> = ResourceIdW::Int(32512);

#[inline]
pub fn LoadIconW(
    instance: HINSTANCE,
    resource_id: ResourceIdW)
    -> Result<HICON, u32>
{
    let result = unsafe {
        let rc_ptr = match resource_id {
            ResourceIdW::Int(id) => id as usize as LPWSTR,
            ResourceIdW::Str(s) => s.as_ptr() as LPWSTR,
        };
        user32::LoadIconW(instance, rc_ptr)
    };

    if result as usize != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn LoadCursorW(
    instance: HINSTANCE,
    resource_id: ResourceIdW)
    -> Result<HCURSOR, u32>
{
    let result = unsafe {
        let rc_ptr = match resource_id {
            ResourceIdW::Int(id) => id as usize as LPWSTR,
            ResourceIdW::Str(s) => s.as_ptr() as LPWSTR,
        };
        user32::LoadCursorW(instance, rc_ptr)
    };

    if result as usize != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn BeginDeferWindowPos(num_windows: c_int) -> Result<HDWP, u32>
{
    let result = unsafe { user32_BeginDeferWindowPos(num_windows) };

    if result as usize != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn DeferWindowPos(
    win_pos_info: HDWP,
    hwnd: HWND,
    insert_after: Option<HWND>,
    x: c_int,
    y: c_int,
    cx: c_int,
    cy: c_int,
    flags: UINT)
    -> Result<HDWP, u32>
{
    let insert_after_ptr = insert_after.unwrap_or(0 as HWND);
    let result = unsafe { user32::DeferWindowPos(
            win_pos_info, hwnd, insert_after_ptr,
            x, y, cx, cy, flags) };

    if result as usize != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn EndDeferWindowPos(hdwp: HDWP) -> Result<(), u32>
{
    let result = unsafe { user32::EndDeferWindowPos(hdwp) };

    if result as usize != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn InvalidateRect(
    hwnd: HWND,
    rect: &RECT,
    erase: bool)
    -> Result<(), u32>
{
    let erase_int = if erase { 1 } else { 0 };
    let result = unsafe { user32::InvalidateRect(hwnd, rect as *const RECT, erase_int) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn TextOutW(
    hdc: HDC,
    x_start: c_int,
    y_start: c_int,
    string: &[u16])
    -> Result<(), u32>
{
    let result = unsafe { gdi32::TextOutW(
            hdc, x_start, y_start, string.as_ptr(), string.len() as c_int) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn SetBkMode(hdc: HDC, mode: c_int) -> Result<(), u32>
{
    let result = unsafe { gdi32::SetBkMode(hdc, mode) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn DrawFocusRect(hdc: HDC, rect: &RECT) -> Result<(), u32>
{
    let result = unsafe { user32::DrawFocusRect(hdc, rect as *const RECT) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn SendMessageW(
    hwnd: HWND,
    msg: UINT,
    param: WPARAM,
    para: LPARAM)
    -> LRESULT
{
    unsafe { user32::SendMessageW(hwnd, msg, param, para) }
}

#[inline]
pub fn GetWindowLongW(hwnd: HWND, index: c_int) -> Result<LONG, u32>
{
    let result = unsafe { user32::GetWindowLongW(hwnd, index) };

    if result != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn SetWindowLongW(hwnd: HWND, index: c_int, value: LONG) -> Result<LONG, u32>
{
    let result = unsafe { user32::SetWindowLongW(hwnd, index, value) };

    if result != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn RegisterClassExW(wnd_cls: &WNDCLASSEXW)
    -> Result<ATOM, u32>
{
    let result = unsafe { user32::RegisterClassExW(wnd_cls as *const WNDCLASSEXW) };

    if result != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

pub enum WinClsIdW {
    Atom(ATOM),
    ClsName(Vec<u16>),
}

#[inline]
pub fn CreateWindowExW(
    ex_style: u32,
    cls_name: WinClsIdW,
    window_name: Option<&[u16]>,
    style: u32,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    wnd_parent: Option<HWND>,
    menu: Option<HMENU>,
    instance: HINSTANCE,
    param: Option<LPVOID>)
    -> Result<HWND, u32>
{
    let result = unsafe {
        let cls_ptr = match cls_name {
            WinClsIdW::Atom(x) => x as LPCWSTR,
            WinClsIdW::ClsName(n) => n.as_ptr() as LPCWSTR,
        };

        user32::CreateWindowExW(
            ex_style, cls_ptr,
            window_name.map_or(0 as LPCWSTR, |x|x.as_ptr() as LPCWSTR),
            style, x, y, width, height,
            wnd_parent.unwrap_or(0 as HWND),
            menu.unwrap_or(0 as HMENU),
            instance,
            param.unwrap_or(0 as LPVOID))
    };

    if result as usize != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

pub struct BlockingMsgLoop {
    hwnd: Option<HWND>,
}

impl Iterator for BlockingMsgLoop {
    type Item = Result<MSG, u32>;

    #[inline]
    fn next(&mut self) -> Option<Result<MSG, u32>> {
        let mut msg = MSG {
            hwnd: 0 as HWND,
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT {x: 0, y: 0},
        };
        let hwnd = self.hwnd.unwrap_or(0 as HWND);
        let result = unsafe { user32::GetMessageW(&mut msg as *mut MSG, hwnd, 0, 0) };

        if result == 0 { None }
        else if result == -1 { Some(Err(GetLastError())) }
        else { Some(Ok(msg)) }
    }
}

pub fn blocking_msg_loop(hwnd: Option<HWND>) -> BlockingMsgLoop
{ BlockingMsgLoop { hwnd: hwnd } }

pub struct PeekingMsgLoop {
    hwnd: Option<HWND>,
}

impl Iterator for PeekingMsgLoop {
    type Item = Result<MSG, u32>;

    #[inline]
    fn next(&mut self) -> Option<Result<MSG, u32>> {
        let mut msg;
        let hwnd = self.hwnd.unwrap_or(0 as HWND);
        let result = unsafe {
            msg = std::mem::zeroed::<MSG>();
            user32::PeekMessageW(&mut msg as *mut MSG, hwnd, 0, 0, PM_REMOVE)
        };

        if result == 0 { None }
        else if result == -1 { Some(Err(GetLastError())) }
        else { Some(Ok(msg)) }
    }
}

pub fn peeking_msg_loop(hwnd: Option<HWND>) -> PeekingMsgLoop
{ PeekingMsgLoop { hwnd: hwnd } }

#[inline]
pub fn IsDialogMessage(hwnd: HWND, msg: &mut MSG) -> bool {
    unsafe { user32::IsDialogMessageW(hwnd, msg as *mut MSG) == 0 }
}

#[inline]
pub fn DestroyWindow(hwnd: HWND) -> Result<(), u32>
{
    let result = unsafe { user32::DestroyWindow(hwnd) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn SetWindowPos(
    hwnd: HWND,
    insert_after: Option<HWND>,
    x: c_int, y: c_int,
    cx: c_int, cy: c_int,
    flags: UINT)
    -> Result<(), u32>
{
    let result = unsafe {
        user32::SetWindowPos(
            hwnd, insert_after.unwrap_or(0 as HWND),
            x, y, cx, cy, flags) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn MoveWindow(
    hwnd: HWND,
    x: c_int, y: c_int,
    cx: c_int, cy: c_int,
    repaint: bool)
    -> Result<(), u32>
{
    let repaint = if repaint { 1 } else { 0 };
    let result = unsafe { user32::MoveWindow(hwnd, x, y, cx, cy, repaint) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn DefWindowProcW(
    hwnd: HWND,
    msg: UINT,
    param: WPARAM,
    para: LPARAM)
    -> LRESULT
{ unsafe { user32::DefWindowProcW(hwnd, msg, param, para)} }

#[inline]
pub fn PostQuitMessage(exit_code: c_int)
{ unsafe { user32::PostQuitMessage(exit_code)} }

#[inline]
pub fn TranslateMessage(msg: &MSG) -> bool
{ unsafe { user32::TranslateMessage(msg as *const MSG) != 0 } }

#[inline]
pub fn DispatchMessageW(msg: &MSG) -> isize
{ unsafe { user32::DispatchMessageW(msg as *const MSG) as isize } }

#[inline]
pub fn GetClientRect(hwnd: HWND) -> Result<RECT, u32>
{
    let mut rect;
    let result = unsafe {
        rect = std::mem::zeroed::<RECT>();
        user32::GetClientRect(hwnd, &mut rect as *mut RECT)
    };

    if result != 0 { Ok(rect) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn BeginPaint(hwnd: HWND) -> Result<(PAINTSTRUCT, HDC), u32>
{
    let mut ps;
    let hdc = unsafe {
        ps = std::mem::zeroed::<PAINTSTRUCT>();
        user32::BeginPaint(hwnd, &mut ps as *mut PAINTSTRUCT)
    };

    if hdc as usize != 0 { Ok((ps, hdc)) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn EndPaint(hwnd: HWND, ps: &PAINTSTRUCT) -> Result<(), u32>
{
    let result = unsafe { user32::EndPaint(hwnd, ps) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn FillRect(hdc: HDC, rect: &RECT, brush: HBRUSH)
    -> Result<(), u32>
{
    let result = unsafe { user32::FillRect(hdc, rect, brush) };

    if result != 0 { Ok(()) }
    else { Err(GetLastError()) }
}
