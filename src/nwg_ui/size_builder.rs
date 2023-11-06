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

use nwg::stretch::geometry::Size;
use nwg::stretch::style::Dimension;

#[derive(Default)]
pub struct SizeBuilder {
    width: Dimension,
    height: Dimension,
}

#[allow(dead_code)]
impl SizeBuilder {
    pub fn width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }

    pub fn width_percent(mut self, width: u32) -> Self {
        self.width = Dimension::Percent(width as f32);
        self
    }

    pub fn width_pt(mut self, width: u32) -> Self {
        self.width = Dimension::Points(width as f32);
        self
    }

    pub fn width_button_normal(mut self) -> Self {
        self.width = Dimension::Points(70 as f32);
        self
    }

    pub fn width_button_wide(mut self) -> Self {
        self.width = Dimension::Points(100 as f32);
        self
    }

    pub fn width_button_xwide(mut self) -> Self {
        self.width = Dimension::Points(130 as f32);
        self
    }
    pub fn width_input_default(mut self) -> Self {
        self.width = Dimension::Points(100 as f32);
        self
    }

    pub fn width_label_normal(mut self) -> Self {
        self.width = Dimension::Points(100 as f32);
        self
    }

    pub fn width_number_input_normal(mut self) -> Self {
        self.width = Dimension::Points(80 as f32);
        self
    }

    pub fn width_checkbox_normal(mut self) -> Self {
        self.width = Dimension::Points(30 as f32);
        self
    }

    pub fn width_auto(mut self) -> Self {
        self.width = Dimension::Auto;
        self
    }

    pub fn height(mut self, height: Dimension) -> Self {
        self.height = height;
        self
    }

    pub fn height_percent(mut self, height: u32) -> Self {
        self.height = Dimension::Percent(height as f32);
        self
    }

    pub fn height_pt(mut self, height: u32) -> Self {
        self.height = Dimension::Points(height as f32);
        self
    }

    pub fn height_button(mut self) -> Self {
        self.height = Dimension::Points(25 as f32);
        self
    }

    pub fn height_input_form_row(mut self) -> Self {
        self.height = Dimension::Points(20 as f32);
        self
    }

    pub fn height_auto(mut self) -> Self {
        self.height = Dimension::Auto;
        self
    }

    pub fn build(self) -> Size<Dimension> {
        Size {
            width: self.width,
            height: self.height
        }
    }
}
