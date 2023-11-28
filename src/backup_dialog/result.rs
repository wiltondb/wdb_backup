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
pub(super) struct BackupResult {
    pub(super) error: String
}

impl BackupResult {
    pub(super) fn success() -> Self {
        Self {
            error: Default::default()
        }
    }

    pub(super) fn failure(error: String) -> Self {
        Self {
            error
        }
    }
}

#[derive(Default, Clone)]
pub struct BackupDialogResult {
    pub success: bool,
}

impl BackupDialogResult {
    pub fn success() -> Self {
        Self {
            success: true,
        }
    }

    pub fn failure() -> Self {
        Self {
            success: false,
        }
    }
}
