#![no_main]
extern crate winapi;
use winapi::{HINSTANCE, c_int, STD_OUTPUT_HANDLE};

mod win_layer;
use win_layer::*;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C"
fn wWinMain(
    _instance: HINSTANCE,
    _prev_instance: HINSTANCE,
    _cmd_line: *const u16,
    _cmd_show: c_int)
    -> c_int
{
    match our_main() { Ok(_) => 0, Err(x) => x as c_int }
}

#[no_mangle]
pub extern "C"
fn main(
    _argc: c_int,
    _argv: *const *const u8)
    -> c_int
{
    match our_main() { Ok(_) => 0, Err(x) => x as c_int }
}

fn our_main() -> Result<u32, u32>
{
    Ok(0)
}
