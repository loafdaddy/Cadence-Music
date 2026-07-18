//! Right-click context menus for library items.
//!
//! Uses `gtk::PopoverMenu` + `gio::Menu` so items pick up native Adwaita
//! menu styling (no custom button borders / focus rings).

use std::rc::Rc;

use adw::prelude::*;
use gtk::gdk;
use gtk::gio;

/// Actions offered on tracks / albums.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextAction {
    AddToQueue,
    AddToPlaylist,
    Delete,
}

/// Attach a secondary-click menu to `widget`.
pub fn attach_context_menu<W, F>(widget: &W, on_action: F)
where
    W: IsA<gtk::Widget>,
    F: Fn(ContextAction) + 'static,
{
    let on_action: Rc<dyn Fn(ContextAction)> = Rc::new(on_action);
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gdk::BUTTON_SECONDARY);
    let widget_weak = widget.downgrade();
    gesture.connect_pressed(move |gesture, _n, x, y| {
        let Some(widget) = widget_weak.upgrade() else {
            return;
        };
        gesture.set_state(gtk::EventSequenceState::Claimed);

        let popover = build_popover(Rc::clone(&on_action));
        popover.set_parent(&widget);
        let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
        popover.set_pointing_to(Some(&rect));
        // Unparent on close so ListBox rows can be finalized cleanly.
        popover.connect_closed(|pop| {
            pop.unparent();
        });
        popover.popup();
    });
    widget.add_controller(gesture);
}

fn build_popover(on_action: Rc<dyn Fn(ContextAction)>) -> gtk::PopoverMenu {
    let menu = gio::Menu::new();
    menu.append(Some("Add to Queue"), Some("ctx.add-to-queue"));
    menu.append(Some("Add to Playlist…"), Some("ctx.add-to-playlist"));
    menu.append(Some("Delete from Library and Disk"), Some("ctx.delete"));

    let group = gio::SimpleActionGroup::new();
    add_ctx_action(
        &group,
        "add-to-queue",
        ContextAction::AddToQueue,
        &on_action,
    );
    add_ctx_action(
        &group,
        "add-to-playlist",
        ContextAction::AddToPlaylist,
        &on_action,
    );
    add_ctx_action(&group, "delete", ContextAction::Delete, &on_action);

    let popover = gtk::PopoverMenu::from_model(Some(&menu));
    popover.set_has_arrow(false);
    popover.set_autohide(true);
    popover.insert_action_group("ctx", Some(&group));
    popover
}

fn add_ctx_action(
    group: &gio::SimpleActionGroup,
    name: &str,
    action: ContextAction,
    on_action: &Rc<dyn Fn(ContextAction)>,
) {
    let simple = gio::SimpleAction::new(name, None);
    let on_action = Rc::clone(on_action);
    simple.connect_activate(move |_, _| {
        on_action(action);
    });
    group.add_action(&simple);
}
