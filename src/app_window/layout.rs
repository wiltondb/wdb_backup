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
    tabs_container_layout: nwg::FlexboxLayout,

    backup_tab_layout: nwg::FlexboxLayout,
    backup_dbname_layout: nwg::FlexboxLayout,
    backup_dest_dir_layout: nwg::FlexboxLayout,
    backup_filename_layout: nwg::FlexboxLayout,
    backup_spacer_layout: nwg::FlexboxLayout,
    backup_buttons_layout: nwg::FlexboxLayout,

    restore_tab_layout: nwg::FlexboxLayout,
    restore_src_dir_layout: nwg::FlexboxLayout,
    restore_bbf_db_layout: nwg::FlexboxLayout,
    restore_dbname_layout: nwg::FlexboxLayout,
    restore_spacer_layout: nwg::FlexboxLayout,
    restore_buttons_layout: nwg::FlexboxLayout,
}

impl ui::Layout<AppWindowControls> for AppWindowLayout {

    // backup

    fn build(&self, c: &AppWindowControls) -> Result<(), nwg::NwgError> {
        nwg::FlexboxLayout::builder()
            .parent(&c.backup_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.backup_dbname_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.backup_dbname_combo)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .build_partial(&self.backup_dbname_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.backup_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.backup_dest_dir_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.backup_dest_dir_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .child(&c.backup_dest_dir_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.backup_dest_dir_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.backup_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.backup_filename_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.backup_filename_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .build_partial(&self.backup_filename_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.backup_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .build_partial(&self.backup_spacer_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.backup_tab)
            .flex_direction(ui::FlexDirection::Row)
            .justify_content(ui::JustifyContent::FlexEnd)
            .auto_spacing(None)
            .child(&c.backup_run_button)
            .child_size(ui::size_builder()
                .width_button_wide()
                .height_button()
                .build())
            .child(&c.backup_close_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.backup_buttons_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.backup_tab)
            .flex_direction(ui::FlexDirection::Column)
            .child_layout(&self.backup_dbname_layout)
            .child_layout(&self.backup_dest_dir_layout)
            .child_layout(&self.backup_filename_layout)
            .child_layout(&self.backup_spacer_layout)
            .child_flex_grow(1.0)
            .child_layout(&self.backup_buttons_layout)
            .build(&self.backup_tab_layout)?;

        // restore

        nwg::FlexboxLayout::builder()
            .parent(&c.restore_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.restore_src_file_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.restore_src_file_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .child(&c.restore_src_file_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.restore_src_dir_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.restore_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.restore_bbf_db_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.restore_bbf_db_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .build_partial(&self.restore_bbf_db_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.restore_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .child(&c.restore_dbname_label)
            .child_size(ui::size_builder()
                .width_label_normal()
                .height_input_form_row()
                .build())
            .child(&c.restore_dbname_input)
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .child_flex_grow(1.0)
            .build_partial(&self.restore_dbname_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.restore_tab)
            .flex_direction(ui::FlexDirection::Row)
            .auto_spacing(None)
            .build_partial(&self.restore_spacer_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.restore_tab)
            .flex_direction(ui::FlexDirection::Row)
            .justify_content(ui::JustifyContent::FlexEnd)
            .auto_spacing(None)
            .child(&c.restore_run_button)
            .child_size(ui::size_builder()
                .width_button_wide()
                .height_button()
                .build())
            .child(&c.restore_close_button)
            .child_size(ui::size_builder()
                .width_button_normal()
                .height_button()
                .build())
            .child_margin(ui::margin_builder()
                .start_pt(5)
                .build())
            .build_partial(&self.restore_buttons_layout)?;

        nwg::FlexboxLayout::builder()
            .parent(&c.restore_tab)
            .flex_direction(ui::FlexDirection::Column)
            .child_layout(&self.restore_src_dir_layout)
            .child_layout(&self.restore_bbf_db_layout)
            .child_layout(&self.restore_dbname_layout)
            .child_layout(&self.restore_spacer_layout)
            .child_flex_grow(1.0)
            .child_layout(&self.restore_buttons_layout)
            .build(&self.restore_tab_layout)?;

        // tabs container

        nwg::FlexboxLayout::builder()
            .parent(&c.window)
            .flex_direction(ui::FlexDirection::Column)
            .child(&c.tabs_container)
            .child_margin(ui::margin_builder()
                .start_default()
                .top_default()
                .end_default()
                .bottom_pt(30)
                .build())
            .build(&self.tabs_container_layout)?;

        Ok(())
    }
}
