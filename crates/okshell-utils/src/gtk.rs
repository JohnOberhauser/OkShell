use relm4::{
    gtk::prelude::*,
    gtk::gdk,
    gtk::gio,
};

pub fn list_model_to_monitors(model: &gio::ListModel) -> Vec<gdk::Monitor> {
    let mut out = Vec::new();

    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i) {
            if let Ok(mon) = obj.downcast::<gdk::Monitor>() {
                out.push(mon);
            }
        }
    }

    out
}

pub fn monitor_at_position(
    model: &gio::ListModel,
    position: u32,
) -> Option<gdk::Monitor> {
    model
        .item(position)
        .and_then(|obj| obj.downcast::<gdk::Monitor>().ok())
}
