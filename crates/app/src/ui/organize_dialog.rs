//! Organise library window — preview stays open until Apply or Cancel.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use cadence_core::organization::{OrganizationPlan, PlanEntry, Preset, Template};

pub struct OrganizeDialog {
    pub window: adw::Window,
    pub preset: gtk::DropDown,
    list: gtk::ListBox,
    status: gtk::Label,
    apply_button: gtk::Button,
    preview_button: gtk::Button,
    plan: Rc<RefCell<Option<OrganizationPlan>>>,
}

impl OrganizeDialog {
    #[must_use]
    pub fn new(parent: &impl IsA<gtk::Window>) -> Self {
        let labels: Vec<&str> = Preset::all().iter().map(|p| p.label()).collect();
        let preset = gtk::DropDown::from_strings(&labels);

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();
        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .min_content_height(280)
            .child(&list)
            .build();

        let status = gtk::Label::builder()
            .label("Choose a template, then preview the proposed moves.")
            .xalign(0.0)
            .wrap(true)
            .css_classes(["dim-label"])
            .build();

        let preview_button = gtk::Button::builder()
            .label("Preview")
            .css_classes(["pill"])
            .build();
        let apply_button = gtk::Button::builder()
            .label("Apply")
            .sensitive(false)
            .css_classes(["suggested-action", "pill"])
            .build();
        let cancel_button = gtk::Button::builder()
            .label("Cancel")
            .css_classes(["pill"])
            .build();

        let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        buttons.set_halign(gtk::Align::End);
        buttons.append(&cancel_button);
        buttons.append(&preview_button);
        buttons.append(&apply_button);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content.set_margin_start(20);
        content.set_margin_end(20);
        content.set_margin_top(16);
        content.set_margin_bottom(16);
        content.append(
            &gtk::Label::builder()
                .label("Organise Library")
                .xalign(0.0)
                .css_classes(["title-2"])
                .build(),
        );
        content.append(
            &gtk::Label::builder()
                .label("Preview file moves before anything changes on disk. You can undo afterwards.")
                .xalign(0.0)
                .wrap(true)
                .css_classes(["dim-label"])
                .build(),
        );
        content.append(&gtk::Label::builder().label("Template").xalign(0.0).build());
        content.append(&preset);
        content.append(&status);
        content.append(&scrolled);
        content.append(&buttons);

        let window = adw::Window::builder()
            .transient_for(parent)
            .modal(true)
            .default_width(640)
            .default_height(520)
            .title("Organise Library")
            .content(&content)
            .build();

        {
            let window = window.clone();
            cancel_button.connect_clicked(move |_| window.close());
        }

        Self {
            window,
            preset,
            list,
            status,
            apply_button,
            preview_button,
            plan: Rc::new(RefCell::new(None)),
        }
    }

    #[must_use]
    pub fn selected_template(&self) -> Template {
        let idx = self.preset.selected() as usize;
        Template::Preset(
            Preset::all()
                .get(idx)
                .copied()
                .unwrap_or(Preset::ArtistAlbumTrack),
        )
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn connect_preview<F: Fn() + 'static>(&self, f: F) {
        self.preview_button.connect_clicked(move |_| f());
    }

    pub fn connect_apply<F: Fn(OrganizationPlan) + 'static>(&self, f: F) {
        let plan = Rc::clone(&self.plan);
        self.apply_button.connect_clicked(move |_| {
            if let Some(p) = plan.borrow().clone() {
                f(p);
            }
        });
    }

    pub fn set_busy(&self, busy: bool) {
        self.preview_button.set_sensitive(!busy);
        self.apply_button.set_sensitive(!busy && self.plan.borrow().is_some());
        if busy {
            self.status.set_label("Building preview…");
        }
    }

    pub fn show_plan(&self, plan: OrganizationPlan) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        let moves = plan.move_count();
        let conflicts = plan.conflict_count();
        let already = plan
            .entries
            .iter()
            .filter(|e| matches!(e, PlanEntry::AlreadyOrganized(_)))
            .count();
        if plan.entries.is_empty() {
            self.status.set_label(
                "No tracks found in the library. Add a music folder and scan first.",
            );
        } else {
            self.status.set_label(&format!(
                "{moves} move(s) ready · {already} already organised · {conflicts} conflict(s)"
            ));
        }
        // Show moves first so the preview feels purposeful.
        let mut ordered: Vec<&PlanEntry> = plan.entries.iter().collect();
        ordered.sort_by_key(|e| match e {
            PlanEntry::Move(_) => 0,
            PlanEntry::Conflict { .. } => 1,
            PlanEntry::AlreadyOrganized(_) => 2,
        });
        for entry in ordered {
            let (label, css) = match entry {
                PlanEntry::AlreadyOrganized(path) => {
                    (format!("Already organised — {}", path.display()), "dim-label")
                }
                PlanEntry::Move(m) => (
                    format!("{}  →  {}", m.from.display(), m.to.display()),
                    "",
                ),
                PlanEntry::Conflict { r#move } => (
                    format!("Conflict — {}", r#move.to.display()),
                    "error",
                ),
            };
            let row = gtk::Label::builder()
                .label(label)
                .xalign(0.0)
                .wrap(true)
                .margin_start(10)
                .margin_end(10)
                .margin_top(6)
                .margin_bottom(6)
                .css_classes(["caption", css])
                .build();
            self.list
                .append(&gtk::ListBoxRow::builder().child(&row).build());
        }
        let can_apply = moves > 0;
        self.apply_button.set_sensitive(can_apply);
        *self.plan.borrow_mut() = if can_apply { Some(plan) } else { None };
    }

    pub fn show_error(&self, message: &str) {
        self.status.set_label(message);
        self.apply_button.set_sensitive(false);
        *self.plan.borrow_mut() = None;
    }

    pub fn close(&self) {
        self.window.close();
    }
}

impl Default for OrganizeDialog {
    fn default() -> Self {
        // Parentless fallback for type completeness; prefer new(parent).
        Self::new(&gtk::Window::new())
    }
}
