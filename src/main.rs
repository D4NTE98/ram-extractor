use std::io::{self, Write};
use winapi::um::winnt::{HANDLE, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::psapi::{EnumProcesses, GetModuleBaseNameA};
use winapi::um::handleapi::CloseHandle;
use winapi::shared::minwindef::{DWORD, FALSE};

fn find_process_id(process_name: &str) -> Option<DWORD> {
    let mut processes = vec![0u32; 1024];
    let mut bytes_returned = 0;

    unsafe {
        if EnumProcesses(processes.as_mut_ptr(), (processes.len() * 4) as u32, &mut bytes_returned) == 0 {
            return None;
        }
    }

    let count = bytes_returned as usize / 4;

    for i in 0..count {
        let pid = processes[i];
        if pid == 0 { continue; }

        unsafe {
            let h_process = OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, pid);
            if h_process.is_null() { continue; }

            let mut name = [0i8; 256];
            if GetModuleBaseNameA(h_process, std::ptr::null_mut(), name.as_mut_ptr() as *mut i8, 256) > 0 {
                let proc_name = std::ffi::CStr::from_ptr(name.as_ptr())
                    .to_string_lossy()
                    .to_lowercase();

                if proc_name.contains(&process_name.to_lowercase()) {
                    CloseHandle(h_process);
                    return Some(pid);
                }
            }
            CloseHandle(h_process);
        }
    }
    None
}

fn read_memory<T: Copy>(handle: HANDLE, address: usize) -> Option<T> {
    let mut buffer: T = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<T>();

    let success = unsafe {
        ReadProcessMemory(
            handle,
            address as *const _,
            &mut buffer as *mut _ as *mut _,
            size,
            std::ptr::null_mut(),
        )
    };

    if success != 0 {
        Some(buffer)
    } else {
        None
    }
}

fn main() {
    println!("=== RAM Extractor (Memory Reader) ===");
    println!("External process memory reading tool written in Rust\n");

    let process_name = "cs2.exe"; // Change this to target different games

    println!("Searching for process: {}", process_name);

    let pid = match find_process_id(process_name) {
        Some(pid) => {
            println!("✅ Process found! PID: {}", pid);
            pid
        }
        None => {
            println!("❌ Process '{}' not found. Make sure the game is running.", process_name);
            return;
        }
    };

    let handle = unsafe {
        OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, FALSE, pid)
    };

    if handle.is_null() {
        println!("❌ Failed to open process. Run this program as Administrator.");
        return;
    }

    println!("✅ Process opened successfully!\n");

    // ====================== MAIN LOOP ======================
    loop {
        println!("--- Menu ---");
        println!("1. Read Integer (i32 - 4 bytes)");
        println!("2. Read Float (f32 - 4 bytes)");
        println!("3. Read String (max 50 characters)");
        println!("0. Exit");

        print!("Choose option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                print!("Enter memory address (hex, e.g. 0x7FF812345678): ");
                io::stdout().flush().unwrap();
                let mut addr_str = String::new();
                io::stdin().read_line(&mut addr_str).unwrap();

                if let Ok(addr) = usize::from_str_radix(
                    addr_str.trim().trim_start_matches("0x"), 16
                ) {
                    if let Some(value) = read_memory::<i32>(handle, addr) {
                        println!("Value (i32): {}", value);
                    } else {
                        println!("❌ Failed to read memory at this address.");
                    }
                } else {
                    println!("❌ Invalid address format.");
                }
            }
            "2" => {
                print!("Enter memory address (hex): ");
                io::stdout().flush().unwrap();
                let mut addr_str = String::new();
                io::stdin().read_line(&mut addr_str).unwrap();

                if let Ok(addr) = usize::from_str_radix(
                    addr_str.trim().trim_start_matches("0x"), 16
                ) {
                    if let Some(value) = read_memory::<f32>(handle, addr) {
                        println!("Value (float): {:.4}", value);
                    } else {
                        println!("❌ Failed to read memory at this address.");
                    }
                } else {
                    println!("❌ Invalid address format.");
                }
            }
            "3" => {
                print!("Enter memory address (hex): ");
                io::stdout().flush().unwrap();
                let mut addr_str = String::new();
                io::stdin().read_line(&mut addr_str).unwrap();

                if let Ok(addr) = usize::from_str_radix(
                    addr_str.trim().trim_start_matches("0x"), 16
                ) {
                    let mut buffer = [0u8; 50];
                    unsafe {
                        ReadProcessMemory(
                            handle,
                            addr as *const _,
                            buffer.as_mut_ptr() as *mut _,
                            50,
                            std::ptr::null_mut(),
                        );
                    }
                    if let Ok(text) = String::from_utf8(buffer.to_vec()) {
                        let clean_text = text.trim_end_matches('\0').trim();
                        println!("String: {}", clean_text);
                    } else {
                        println!("❌ Failed to read string.");
                    }
                } else {
                    println!("❌ Invalid address format.");
                }
            }
            "0" => {
                println!("Exiting...");
                break;
            }
            _ => println!("❌ Invalid option. Please try again."),
        }
    }

    unsafe { CloseHandle(handle); }
    println!("Program terminated.");
}
