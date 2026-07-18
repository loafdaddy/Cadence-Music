//! Right-click context menus for library items.

use std::rc::Rc;

use adw::prelude::*;
use gtk::gdk;

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

fn build_popover(on_action: Rc<dyn Fn(ContextAction)>) -> gtk::Popover {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    list.add_css_class("cadence-context-menu");

    for (label, action) in [
        ("Add to queue", ContextAction::AddToQueue),
        ("Add to playlist…", ContextAction::AddToPlaylist),
        ("Delete from library and disk", ContextAction::Delete),
    ] {
        let btn = gtk::Button::builder()
            .label(label)
            .halign(gtk::Align::Start)
            .css_classes(["flat", "cadence-context-item"])
            .build();
        if action == ContextAction::Delete {
            btn.add_css_class("destructive-action");
        }
        let on_action = Rc::clone(&on_action);
        btn.connect_clicked(move |btn| {
            if let Some(pop) = btn
                .ancestor(gtk::Popover::static_type())
                .and_then(|w| w.downcast::<gtk::Popover>().ok())
            {
                pop.popdown();
            }
            on_action(action);
        });
        list.append(&btn);
    }

    gtk::Popover::builder()
        .autohide(true)
        .has_arrow(false)
        .child(&list)
        .build()
}
