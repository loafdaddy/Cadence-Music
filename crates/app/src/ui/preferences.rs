use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;

use crate::services::LibraryService;

pub struct PreferencesWindow {
    pub window: adw::PreferencesWindow,
}

impl PreferencesWindow {
    #[must_use]
    pub fn new(parent: &impl IsA<gtk::Window>, library: LibraryService) -> Self {
        let window = adw::PreferencesWindow::builder()
            .transient_for(parent)
            .title("Preferences")
            .search_enabled(true)
            .build();

        let folders_group = adw::PreferencesGroup::builder()
            .title("Library Folders")
            .description("Music folders Cadence indexes. Organisation never runs unless you ask.")
            .build();

        let folder_list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();
        folders_group.add(&folder_list);

        let add_btn = gtk::Button::builder()
            .label("Add Folder")
            .css_classes(["pill"])
            .halign(gtk::Align::Start)
            .build();
        let add_row = adw::ActionRow::builder()
            .title("Add a music folder")
            .build();
        add_row.add_suffix(&add_btn);
        folders_group.add(&add_row);

        let page = adw::PreferencesPage::builder().title("Library").build();
        page.add(&folders_group);

        let playback = adw::PreferencesGroup::builder()
            .title("Playback")
            .description("Theme follows GNOME. Crossfade and ReplayGain will arrive later.")
            .build();
        playback.add(
            &adw::ActionRow::builder()
                .title("Colour scheme")
                .subtitle("System")
                .build(),
        );
        let playback_page = adw::PreferencesPage::builder().title("Playback").build();
        playback_page.add(&playback);

        window.add(&page);
        window.add(&playback_page);

        let folder_list = Rc::new(folder_list);
        reload_folders(&library, &folder_list);

        let lib = library.clone();
        let list = Rc::clone(&folder_list);
        let parent_win = parent.clone().upcast::<gtk::Window>();
        add_btn.connect_clicked(move |_| {
            let dialog = gtk::FileDialog::builder().title("Add Music Folder").build();
            let lib = lib.clone();
            let list = Rc::clone(&list);
            dialog.select_folder(Some(&parent_win), gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let list = Rc::clone(&list);
                        let lib2 = lib.clone();
                        lib.add_folder(path, move |_| {
                            reload_folders(&lib2, &list);
                        });
                    }
                }
            });
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn reload_folders(library: &LibraryService, list: &Rc<gtk::ListBox>) {
    let list = Rc::clone(list);
    library.list_folders(move |result| {
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }
        if let Ok(folders) = result {
            for folder in folders {
                let row = adw::ActionRow::builder()
                    .title(folder.display().to_string())
                    .build();
                list.append(&row);
            }
        }
    });
}
