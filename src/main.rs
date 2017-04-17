#![no_main]
extern crate winapi;
extern crate user32;

use winapi::*;
mod win_layer;
use win_layer::*;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system"
fn wWinMain(
    instance: HINSTANCE,
    _prev_instance: HINSTANCE,
    _cmd_line: *const u16,
    _cmd_show: c_int)
    -> c_int
{
    match our_main(instance) { Ok(_) => 0, Err(x) => x as c_int }
}

#[no_mangle]
pub extern "system"
fn main(
    _argc: c_int,
    _argv: *const *const u8)
    -> c_int
{
    let instance = GetModuleHandleW(None).unwrap_or(0 as HINSTANCE);
    match our_main(instance) { Ok(_) => 0, Err(x) => x as c_int }
}

extern "system"
fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    param: WPARAM,
    para: LPARAM)
    -> LRESULT
{
    //println!("winmsg: 0x{:04x}", msg);
    match msg {
        WM_CLOSE => match DestroyWindow(hwnd) {
            Ok(_) => 0,
            Err(x) => x as LRESULT
        },
        WM_PAINT => {
            unsafe {
                let mut ps = std::mem::zeroed::<PAINTSTRUCT>();
                let hdc = user32::BeginPaint(hwnd, &mut ps as *mut PAINTSTRUCT);
                let mut r = std::mem::zeroed::<RECT>();
                user32::GetClientRect(hwnd, &mut r as *mut RECT);
                user32::FillRect(hdc, &r, (winapi::COLOR_WINDOW + 2) as HBRUSH);
                user32::EndPaint(hwnd, &ps);
            }
            0
        }
        WM_DESTROY => { PostQuitMessage(0); 0 },
        _ => DefWindowProcW(hwnd, msg, param, para)
    }
}

fn our_main(instance: HINSTANCE) -> Result<u32, u32>
{
    let wnd_cls = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: unsafe { user32::LoadIconW(0 as HINSTANCE, IDI_APPLICATION) },
        hCursor: unsafe { user32::LoadCursorW(0 as HINSTANCE, IDC_ARROW) },
        hbrBackground: (winapi::COLOR_WINDOW + 1) as HBRUSH,
        lpszMenuName: 0 as *const u16,
        lpszClassName: &[0x40u16, 0x40, 0x00] as *const u16,
        hIconSm: unsafe { user32::LoadIconW(0 as HINSTANCE, IDI_APPLICATION) },
    };

    let atom = try!(RegisterClassExW(&wnd_cls));
    let hwnd = try!(CreateWindowExW(
        winapi::WS_EX_CLIENTEDGE,
        WinClsW::Atom(atom),
        Some(&[0x44, 0x44, 0x00]),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, 640, 480,
        None, None, instance, None));

    let _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
    let _ = try!(UpdateWindow(hwnd));

    /*
    'outer: loop {
        for rmsg in peeking_msg_loop(None) {
            let msg = try!(rmsg);
            println!("msg, 0x{:04x}", msg.message);
            if msg.message == WM_QUIT { break 'outer; }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    */

    for rmsg in blocking_msg_loop(None) {
        let msg = try!(rmsg);
        //println!("msg, 0x{:04x}", msg.message);
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    Ok(0)
}
