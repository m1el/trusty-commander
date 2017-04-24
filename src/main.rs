#![no_main]
extern crate winapi;
extern crate user32;

use winapi::*;
mod win_layer;
mod win_gdi;
mod messages;
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

fn our_main(instance: HINSTANCE) -> Result<(), u32>
{
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let hwnd = try!(win_gdi::MainCls::create(instance));
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

    Ok(())
}
