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

use std::mem;
use std::thread::JoinHandle;

#[derive(Default)]
pub struct PopupJoinHandle<T> {
    handle: Option<JoinHandle<T>>
}

impl<T> PopupJoinHandle<T> {
    pub fn join(&mut self) -> T {
        mem::take(&mut self.handle)
            .expect("Join handle not set")
            .join()
            .expect("Join error")
    }
}

impl<T> From<JoinHandle<T>> for PopupJoinHandle<T> {
    fn from(value: JoinHandle<T>) -> Self {
        Self {
            handle: Some(value)
        }
    }
}
