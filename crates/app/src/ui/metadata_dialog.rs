use adw::prelude::*;
use cadence_core::models::TrackMetadata;

/// Simple metadata editor. Returns confirmed metadata via callback.
pub struct MetadataDialog {
    pub dialog: adw::AlertDialog,
    pub title: adw::EntryRow,
    pub artist: adw::EntryRow,
    pub album: adw::EntryRow,
    pub album_artist: adw::EntryRow,
    pub genre: adw::EntryRow,
    pub year: adw::EntryRow,
    pub track_number: adw::EntryRow,
}

impl MetadataDialog {
    #[must_use]
    pub fn new(initial: &TrackMetadata) -> Self {
        let title = adw::EntryRow::builder().title("Title").build();
        title.set_text(initial.title.as_deref().unwrap_or(""));
        let artist = adw::EntryRow::builder().title("Artist").build();
        artist.set_text(initial.artist.as_deref().unwrap_or(""));
        let album = adw::EntryRow::builder().title("Album").build();
        album.set_text(initial.album.as_deref().unwrap_or(""));
        let album_artist = adw::EntryRow::builder().title("Album Artist").build();
        album_artist.set_text(initial.album_artist.as_deref().unwrap_or(""));
        let genre = adw::EntryRow::builder().title("Genre").build();
        genre.set_text(initial.genre.as_deref().unwrap_or(""));
        let year = adw::EntryRow::builder().title("Year").build();
        if let Some(y) = initial.year {
            year.set_text(&y.to_string());
        }
        let track_number = adw::EntryRow::builder().title("Track Number").build();
        if let Some(n) = initial.track_number {
            track_number.set_text(&n.to_string());
        }

        let group = adw::PreferencesGroup::new();
        group.add(&title);
        group.add(&artist);
        group.add(&album);
        group.add(&album_artist);
        group.add(&genre);
        group.add(&year);
        group.add(&track_number);

        let dialog = adw::AlertDialog::builder()
            .heading("Edit Metadata")
            .body("Changes are written to the file after you confirm.")
            .extra_child(&group)
            .build();
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("save", "Save");
        dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("save"));
        dialog.set_close_response("cancel");

        Self {
            dialog,
            title,
            artist,
            album,
            album_artist,
            genre,
            year,
            track_number,
        }
    }

    #[must_use]
    pub fn into_metadata(&self) -> TrackMetadata {
        let text = |row: &adw::EntryRow| {
            let t = row.text().to_string();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        };
        TrackMetadata {
            title: text(&self.title),
            artist: text(&self.artist),
            album: text(&self.album),
            album_artist: text(&self.album_artist),
            genre: text(&self.genre),
            year: self.year.text().parse().ok(),
            track_number: self.track_number.text().parse().ok(),
            ..Default::default()
        }
    }
}
