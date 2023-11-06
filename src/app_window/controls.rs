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
pub(super) struct AppWindowControls {
    layout: AppWindowLayout,

    pub(super) font_normal: nwg::Font,
    pub(super) font_small: nwg::Font,

    pub(super) icon: nwg::Icon,
    pub(super) window: nwg::Window,

    pub(super) file_menu: nwg::Menu,
    pub(super) file_connect_menu_item: nwg::MenuItem,
    pub(super) file_exit_menu_item: nwg::MenuItem,
    pub(super) help_menu: nwg::Menu,
    pub(super) help_about_menu_item: nwg::MenuItem,
    pub(super) help_website_menu_item: nwg::MenuItem,

    pub(super) dbname_label: nwg::Label,
    pub(super) dbname_input: nwg::TextInput,
    pub(super) dest_dir_label: nwg::Label,
    pub(super) dest_dir_input: nwg::TextInput,
    pub(super) dest_dir_button: nwg::Button,
    pub(super) dest_dir_chooser: nwg::FileDialog,

    pub(super) close_button: nwg::Button,
    pub(super) status_bar: nwg::StatusBar,

    pub(super) about_notice: ui::SyncNotice,
    pub(super) connect_notice: ui::SyncNotice,
}

impl ui::Controls for AppWindowControls {
    fn build(&mut self) -> Result<(), nwg::NwgError> {
        // fonts
        nwg::Font::builder()
            .size(ui::font_size_builder()
                .normal()
                .build())
            .build(&mut self.font_normal)?;
        nwg::Font::builder()
            .size(ui::font_size_builder()
                .small()
                .build())
            .build(&mut self.font_small)?;

        // window

        nwg::Icon::builder()
            .source_embed(Some(&nwg::EmbedResource::load(None)
                .expect("Error loading embedded resource")))
            .source_embed_id(2)
            .build(&mut self.icon)?;

        nwg::Window::builder()
            .size((640, 320))
            .icon(Some(&self.icon))
            .center(true)
            .title("WiltonDB Backup Tool")
            .build(&mut self.window)?;

        // menu

        nwg::Menu::builder()
            .parent(&self.window)
            .text("File")
            .build(&mut self.file_menu)?;
        nwg::MenuItem::builder()
            .parent(&self.file_menu)
            .text("DB Connection")
            .build(&mut self.file_connect_menu_item)?;
        nwg::MenuItem::builder()
            .parent(&self.file_menu)
            .text("Exit")
            .build(&mut self.file_exit_menu_item)?;

        nwg::Menu::builder()
            .parent(&self.window)
            .text("Help")
            .build(&mut self.help_menu)?;
        nwg::MenuItem::builder()
            .parent(&self.help_menu)
            .text("About")
            .build(&mut self.help_about_menu_item)?;
        nwg::MenuItem::builder()
            .parent(&self.help_menu)
            .text("Website")
            .build(&mut self.help_website_menu_item)?;

        // form

        nwg::Label::builder()
            .text("Logical DB name:")
            .font(Some(&self.font_normal))
            .h_align(nwg::HTextAlign::Left)
            .parent(&self.window)
            .build(&mut self.dbname_label)?;
        nwg::TextInput::builder()
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.dbname_input)?;

        nwg::Label::builder()
            .text("Destination directory:")
            .font(Some(&self.font_normal))
            .h_align(nwg::HTextAlign::Left)
            .parent(&self.window)
            .build(&mut self.dest_dir_label)?;
        nwg::TextInput::builder()
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.dest_dir_input)?;
        nwg::Button::builder()
            .text("Choose")
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.dest_dir_button)?;
        nwg::FileDialog::builder()
            .title("Choose destination directory")
            .action(nwg::FileDialogAction::OpenDirectory)
            .build(&mut self.dest_dir_chooser)?;

        // buttons

        nwg::Button::builder()
            .text("Close")
            .font(Some(&self.font_normal))
            .parent(&self.window)
            .build(&mut self.close_button)?;

        // other

        nwg::StatusBar::builder()
            .parent(&self.window)
            .font(Some(&self.font_small))
            .build(&mut self.status_bar)?;

        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.about_notice)?;
        ui::notice_builder()
            .parent(&self.window)
            .build(&mut self.connect_notice)?;

        self.layout.build(&self)?;

        Ok(())
    }

    fn update_tab_order(&self) {
        ui::tab_order_builder()
            .control(&self.close_button)
            .build();
    }
}
