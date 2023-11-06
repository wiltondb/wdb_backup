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

use nwg::stretch::geometry::Rect;
use nwg::stretch::style::Dimension;

#[derive(Default)]
pub struct MarginBuilder {
    start: u32,
    end: u32,
    top: u32,
    bottom: u32,
}

#[allow(dead_code)]
impl MarginBuilder {
    pub fn start_pt(mut self, start: u32) -> Self {
        self.start = start;
        self
    }

    pub fn start_default(mut self) -> Self {
        self.start = 5;
        self
    }

    pub fn start_no_label_normal(mut self) -> Self {
        self.start = 105;
        self
    }

    pub fn end_pt(mut self, end: u32) -> Self {
        self.end = end;
        self
    }

    pub fn end_default(mut self) -> Self {
        self.end = 5;
        self
    }

    pub fn top_pt(mut self, top: u32) -> Self {
        self.top = top;
        self
    }

    pub fn top_default(mut self) -> Self {
        self.top = 5;
        self
    }

    pub fn bottom_pt(mut self, bottom: u32) -> Self {
        self.bottom = bottom;
        self
    }

    pub fn bottom_default(mut self) -> Self {
        self.bottom = 5;
        self
    }

    pub fn build(self) -> Rect<Dimension> {
        Rect {
            start: Dimension::Points(self.start as f32),
            end: Dimension::Points(self.end as f32),
            top: Dimension::Points(self.top as f32),
            bottom: Dimension::Points(self.bottom as f32),
        }
    }
}
