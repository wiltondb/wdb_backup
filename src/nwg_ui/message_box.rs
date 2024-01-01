/*
 * Copyright 2023, WiltonDB Software
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::ptr::null_mut as NULL;

use winapi::um::winuser;

pub fn message_box(title: &str, message: &str, flags: u32) -> i32 {
    let mut title_term = title.to_string();
    title_term.push('\0');
    let title_wide: Vec<u16> =  title_term.encode_utf16().collect();
    let mut message_term = message.to_string();
    message_term.push('\0');
    let message_wide: Vec<u16> = message_term.encode_utf16().collect();
    unsafe {
        winuser::MessageBoxW(
            NULL(),
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            flags
        ) as i32
    }
}

#[allow(dead_code)]
pub fn message_box_debug(message: &str) {
    message_box("Debug", message, winuser::MB_OK | winuser::MB_ICONINFORMATION);
}

pub fn message_box_warning_yn(message: &str) -> bool {
    let code = message_box("Warning", message, winuser::MB_YESNO | winuser::MB_ICONWARNING);
    if 6 == code {
        true
    } else {
        false
    }
}

