use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::{Accessibility::*, WindowsAndMessaging::*},
        System::Threading::*,
    },
};

unsafe extern "system" fn event_callback(
    _h_win_event_hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    let title = get_window_title(hwnd).unwrap_or_else(|_| "Unknown".to_string());
    let pid = get_window_pid(hwnd).unwrap_or(0);
    let exe_path = get_window_exe_path(hwnd).unwrap_or_else(|_| "Unknown".to_string());

    println!("title: {}, pid: {}, exe_path: {}", title, pid, exe_path);
}

unsafe fn get_window_title(hwnd: HWND) -> Result<String> {
    let length = GetWindowTextLengthW(hwnd) + 1; // +1 for null terminator
    let mut title = vec![0u16; length as usize];
    GetWindowTextW(hwnd, &mut title);

    // Convert to String and trim null terminator if present
    if let Some(end) = title.iter().position(|&c| c == 0) {
        title.truncate(end);
    }
    Ok(String::from_utf16(&title).expect("Failed to decode UTF-16"))
}

unsafe fn get_window_pid(hwnd: HWND) -> Result<u32> {
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    Ok(pid)
}

unsafe fn get_window_exe_path(hwnd: HWND) -> Result<String> {
    let pid = get_window_pid(hwnd)?;

    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)?;
    let mut buffer = [0u16; 4096];
    let mut size = buffer.len() as u32;
    QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut size)?;
    CloseHandle(handle)?;

    // Convert to String and trim null terminator if present
    if let Some(end) = buffer.iter().position(|&c| c == 0) {
        buffer[end] = 0;
    }
    Ok(String::from_utf16(&buffer[..size as usize]).expect("Failed to decode UTF-16"))
}

fn main() -> Result<()> {
    unsafe {
        let event_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        let mut msg: MSG = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWinEvent(event_hook);
    }
    Ok(())
}
