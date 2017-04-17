#![allow(non_snake_case)]
#![allow(dead_code)]
extern crate winapi;
extern crate kernel32;
extern crate user32;
extern crate std;

use winapi::*;

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

    if handle != 0 as HMODULE { Ok(handle) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn GetModuleHandleA(name: Option<&[u8]>) -> Result<HMODULE, u32>
{
    let handle = unsafe { kernel32::GetModuleHandleA(name.map_or(0 as LPCSTR, |x|x.as_ptr() as LPCSTR)) };
    if handle != 0 as HMODULE { Ok(handle) }
    else { Err(GetLastError()) }
}

#[inline]
pub fn GetStdHandle(handle_id: u32) -> Result<Option<HANDLE>, u32>
{
    let handle = unsafe { kernel32::GetStdHandle(handle_id) };

    if handle == 0 as HANDLE { Ok(None) }
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
        let buffer_ptr = buffer.as_ptr() as *mut winapi::c_void;
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

    if handle != 0 as HWND { Some(handle) }
    else { None }
}

#[inline]
pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> bool
{
    unsafe { user32::ShowWindow(hWnd, nCmdShow) != 0 }
}

#[inline]
pub fn UpdateWindow(hWnd: HWND) -> Result<(), u32>
{
    let result = unsafe { user32::UpdateWindow(hWnd) };

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
pub fn RegisterClassExW(wnd_cls: &WNDCLASSEXW)
    -> Result<ATOM, u32>
{
    let result = unsafe { user32::RegisterClassExW(wnd_cls as *const WNDCLASSEXW) };
    if result != 0 { Ok(result) }
    else { Err(GetLastError()) }
}

pub enum WinClsW<'a> {
    Atom(ATOM),
    ClsName(&'a [u16]),
}

#[inline]
pub fn CreateWindowExW(
    ex_style: u32,
    cls_name: WinClsW,
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
            WinClsW::Atom(x) => x as LPCWSTR,
            WinClsW::ClsName(n) => (*n).as_ptr() as LPCWSTR,
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

    if result != 0 as HWND { Ok(result) }
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
        let mut msg = unsafe { std::mem::zeroed::<MSG>() };
        let hwnd = self.hwnd.unwrap_or(0 as HWND);
        let result = unsafe { user32::PeekMessageW(&mut msg as *mut MSG, hwnd, 0, 0, PM_REMOVE) };

        if result == 0 { None }
        else if result == -1 { Some(Err(GetLastError())) }
        else { Some(Ok(msg)) }
    }
}

pub fn peeking_msg_loop(hwnd: Option<HWND>) -> PeekingMsgLoop
{ PeekingMsgLoop { hwnd: hwnd } }


#[inline]
pub fn DestroyWindow(hwnd: HWND) -> Result<(), u32>
{
    let result = unsafe { user32::DestroyWindow(hwnd) };

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
