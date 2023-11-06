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

#[derive(Default)]
pub(super) struct ConnectCheckResult {
    pub(super) success: bool,
    pub(super) message: String,
}

impl ConnectCheckResult {
    pub(super) fn success(message: String) -> Self {
        Self {
            success: true,
            message
        }
    }

    pub(super) fn failure(message: String) -> Self {
        Self {
            success: false,
            message
        }
    }
}

pub struct ConnectCheckDialogResult {
    pub value: Result<String, postgres::Error>
}

impl Default for ConnectCheckDialogResult {
    fn default() -> Self {
        Self {
            value: Ok(String::new())
        }
    }
}
