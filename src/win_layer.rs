#![allow(non_snake_case)]
#![allow(dead_code)]
extern crate winapi;
extern crate kernel32;
extern crate user32;

use winapi::{HANDLE, OVERLAPPED, INVALID_HANDLE_VALUE};
use winapi::{HWND};
use winapi::{c_int, c_uint};

#[allow(dead_code)]
#[inline]
pub fn ExitProcess(code: c_uint) -> !
{ unsafe { kernel32::ExitProcess(code) }; loop {} }

#[inline]
pub fn GetLastError() -> u32
{ unsafe { kernel32::GetLastError() } }

#[inline]
pub fn GetConsoleWindow() -> Option<HWND>
{
    let handle = unsafe { kernel32::GetConsoleWindow() };

    if handle != 0 as HWND { Some(handle) }
    else { None }
}

#[inline]
pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> Result<(), u32>
{
    let result = unsafe { user32::ShowWindow(hWnd, nCmdShow) };

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
