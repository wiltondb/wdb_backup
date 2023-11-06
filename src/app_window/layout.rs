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

use super::*;

#[derive(Default)]
pub(super) struct AppWindowLayout {
    root_layout: nwg::FlexboxLayout,
    dbname_layout: nwg::FlexboxLayout,
    dest_dir_layout: nwg::FlexboxLayout,
    spacer_layout: nwg::FlexboxLayout,
    buttons_layout: nwg::FlexboxLayout,
}

impl ui::Layout<AppWindowControls> for AppWindowLayout {
    fn build(&self, c: &AppWindowControls) -> Result<(), nwg::NwgError> {
        nwg::FlexboxLayout::builder()
            .parent(&c.window)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.dbname_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.dbname_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .build_partial(&self.dbname_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.window)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.dest_dir_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.dest_dir_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .child(&c.dest_dir_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.dest_dir_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.window)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .build_partial(&self.spacer_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.window)
            .flex_direction(ui::FlexDirection::Row)
            .justify_content(ui::JustifyContent::FlexEnd)
            .auto_spacing(None)
            .child(&c.close_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .bottom_pt(22)
                .build())
            .build_partial(&self.buttons_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.window)
            .flex_direction(ui::FlexDirection::Column)
            .child_layout(&self.dbname_layout)
            .child_layout(&self.dest_dir_layout)
            .child_layout(&self.spacer_layout)
            .child_flex_grow(1.0)
            .child_layout(&self.buttons_layout)
            .build(&self.root_layout)?;

        Ok(())
    }
}
