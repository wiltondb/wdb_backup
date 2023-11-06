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

use winapi::shared::windef::HWND;
use winapi::um::winuser::SetWindowPos;
use winapi::um::winuser::SWP_NOMOVE;
use winapi::um::winuser::SWP_NOSIZE;

#[derive(Default)]
pub struct TabOrderBuilder {
    control_handles: Vec<nwg::ControlHandle>,
}

impl TabOrderBuilder {
    pub(super) fn new() -> Self {
        Default::default()
    }

    pub fn control<C: Into<nwg::ControlHandle>>(mut self, control: C) -> Self {
        self.control_handles.push(control.into());
        self
    }

    pub fn build(self) {
        let handles: Vec<HWND> = self.control_handles
            .iter()
            .filter_map(|h: &nwg::ControlHandle| h.hwnd())
            .collect();
        for i in 0..handles.len() - 1 {
            unsafe {
                SetWindowPos(
                    handles[i + 1],
                    handles[i],
                    0, 0, 0, 0,
                    SWP_NOSIZE | SWP_NOMOVE);
            }
        }
    }
}
