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
pub struct FontSizeBuilder {
    size: u32
}

impl FontSizeBuilder {
    pub fn normal(mut self) -> Self {
        self.size = 15;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = 14;
        self
    }

    pub fn build(self) -> u32 {
        self.size
    }
}
