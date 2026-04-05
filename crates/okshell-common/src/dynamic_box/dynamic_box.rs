use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    hash::Hash,
    marker::PhantomData,
    time::Duration,
};
use relm4::gtk::glib;
use gtk::prelude::*;
use relm4::{
    gtk,
    Component, ComponentParts, ComponentSender,
};
use crate::dynamic_box::generic_widget_controller::GenericWidgetController;
// ---- DynamicBox -------------------------------------------------------------

/// Entry stored per key: a Revealer inserted into the box, and the child controller.
pub struct Entry {
    pub revealer: gtk::Revealer,
    pub controller: Box<dyn GenericWidgetController>,
}

/// How to build/update children.
pub struct DynamicBoxFactory<Item, Key> {
    /// Compute stable key for item.
    pub id: Box<dyn Fn(&Item) -> Key + 'static>,
    /// Create a new child controller for this item.
    pub create: Box<dyn Fn(&Item) -> Box<dyn GenericWidgetController> + 'static>,
    /// Optional: update an existing child when the item changes.
    pub update: Option<Box<dyn Fn(&Box<dyn GenericWidgetController>, &Item) + 'static>>,
}

/// Input messages to drive reconciliation.
pub enum DynamicBoxInput<Item, Key> {
    SetItems(Vec<Item>),
    FinalizeRemoval(Key),
}

impl<Item, Key> Debug for DynamicBoxInput<Item, Key> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetItems(items) => f.debug_tuple("SetItems").field(&items.len()).finish(),
            Self::FinalizeRemoval(_) => write!(f, "FinalizeRemoval(..)"),
        }
    }
}

struct ExitAnchor<Key> {
    prev: Option<Key>, // nearest still-present key before it
    next: Option<Key>, // nearest still-present key after it
}

/// Init parameters.
pub struct DynamicBoxInit<Item, Key> {
    pub factory: DynamicBoxFactory<Item, Key>,
    pub orientation: gtk::Orientation,
    pub spacing: i32,

    pub transition_type: gtk::RevealerTransitionType,
    pub transition_duration_ms: u32,

    /// If true, reverse the order of the provided list.
    pub reverse: bool,
    pub retain_entries: bool,
}

/// A keyed container that:
/// - keeps matching-id children
/// - reorders to match new list
/// - animates removal with GtkRevealer and delays final unmount
pub struct DynamicBoxModel<Item, Key>
where
    Key: Eq + Hash + Clone + 'static,
    Item: 'static,
{
    factory: DynamicBoxFactory<Item, Key>,

    orientation: gtk::Orientation,
    spacing: i32,
    transition_type: gtk::RevealerTransitionType,
    transition_duration_ms: u32,
    reverse: bool,

    pub entries: HashMap<Key, Entry>,
    exiting: HashSet<Key>,

    // Used to send FinalizeRemoval(Key) back into the component from a GLib timeout.
    finalize_tx: relm4::Sender<DynamicBoxInput<Item, Key>>,

    _phantom: PhantomData<Item>,

    pub order: Vec<Key>, // current visual order (includes exiting)
    exit_anchors: HashMap<Key, ExitAnchor<Key>>,
    retain_entries: bool,
    hidden: HashSet<Key>,
}

#[relm4::component(pub)]
impl<Item, Key> Component for DynamicBoxModel<Item, Key>
where
    Key: Eq + Hash + Clone + 'static,
    Item: 'static,
{
    type CommandOutput = ();
    type Init = DynamicBoxInit<Item, Key>;
    type Input = DynamicBoxInput<Item, Key>;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: model.orientation,
            set_spacing: model.spacing,
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DynamicBoxModel {
            factory: init.factory,
            orientation: init.orientation,
            spacing: init.spacing,
            transition_type: init.transition_type,
            transition_duration_ms: init.transition_duration_ms,
            reverse: init.reverse,
            entries: HashMap::new(),
            exiting: HashSet::new(),
            finalize_tx: sender.input_sender().clone(),
            _phantom: PhantomData,
            order: Vec::new(),
            exit_anchors: HashMap::new(),
            retain_entries: init.retain_entries,
            hidden: HashSet::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            DynamicBoxInput::SetItems(mut items) => {
                if self.reverse {
                    items.reverse();
                }
                self.reconcile(root, items);
            }
            DynamicBoxInput::FinalizeRemoval(key) => {
                self.finalize_removal(root, key);
            }
        }
    }
}

impl<Item, Key> DynamicBoxModel<Item, Key>
where
    Key: Eq + Hash + Clone + 'static,
    Item: 'static,
{
    pub fn reconcile(&mut self, root: &gtk::Box, items: Vec<Item>) {
        // 1) Compute desired order and set.
        let mut desired: Vec<(Key, Item)> = Vec::with_capacity(items.len());
        for item in items {
            desired.push(((self.factory.id)(&item), item));
        }

        let desired_keys: Vec<Key> = desired.iter().map(|(k, _)| k.clone()).collect();

        let mut desired_set: HashSet<Key> = HashSet::with_capacity(desired.len());
        for (k, _) in desired.iter() {
            desired_set.insert(k.clone());
        }

        // 2) Ensure all desired entries exist (create new if missing) and update existing.
        for (key, item) in desired.iter() {
            if let Some(entry) = self.entries.get_mut(key) {
                // Cancel exit if it was exiting.
                if self.exiting.remove(key) {
                    self.hidden.remove(key);
                    self.exit_anchors.remove(key);
                    entry.revealer.set_visible(true);
                    entry.revealer.set_reveal_child(true);
                    entry.revealer.add_css_class("visible");
                } else if self.hidden.remove(key) {
                    // Was fully hidden (retained), bring it back
                    entry.revealer.set_visible(true);
                    entry.revealer.set_reveal_child(true);
                    entry.revealer.add_css_class("visible");
                    if !self.order.iter().any(|k| k == key) {
                        self.order.push(key.clone());
                    }
                }

                if let Some(update) = &self.factory.update {
                    update(&entry.controller, item);
                }
            } else {
                let controller = (self.factory.create)(item);
                let child_widget = controller.root_widget();

                let revealer = gtk::Revealer::new();
                revealer.set_transition_type(self.transition_type);
                revealer.set_transition_duration(self.transition_duration_ms);
                revealer.set_child(Some(&child_widget));
                revealer.set_reveal_child(false);
                revealer.add_css_class("dynamic_box_revealer");

                root.append(&revealer);

                // Enter: instant if duration is 0, otherwise animate on next idle.
                if self.transition_duration_ms == 0 {
                    revealer.set_reveal_child(true);
                    revealer.add_css_class("visible");
                } else {
                    let revealer_clone = revealer.clone();
                    glib::idle_add_local_once(move || {
                        revealer_clone.set_reveal_child(true);
                        revealer_clone.add_css_class("visible");
                    });
                }

                self.entries.insert(
                    key.clone(),
                    Entry {
                        revealer,
                        controller,
                    },
                );

                // If this key is brand new, ensure it's in order (we rebuild order below anyway,
                // but this helps anchor correctness if something exits in same reconcile).
                if !self.order.iter().any(|k| k == key) {
                    self.order.push(key.clone());
                }
            }
        }

        // 3) Start exit animation for missing keys: record anchors first, then animate out.
        let missing_keys: Vec<Key> = self
            .entries
            .keys()
            .filter(|k| !desired_set.contains(*k) && !self.hidden.contains(*k))
            .cloned()
            .collect();

        for key in missing_keys {
            if self.exiting.contains(&key) {
                continue;
            }

            // Record anchor so it stays positioned while desired changes.
            self.record_exit_anchor(&key, &desired_set);

            if let Some(entry) = self.entries.get(&key) {
                self.exiting.insert(key.clone());

                if self.transition_duration_ms == 0 {
                    // Skip animation entirely — finalize immediately via zero-delay timeout.
                    let delay = Duration::from_millis(0);
                    self.schedule_finalize_removal(key, delay);
                } else {
                    entry.revealer.remove_css_class("visible");
                    entry.revealer.set_reveal_child(false);
                    let delay = Duration::from_millis(self.transition_duration_ms as u64);
                    self.schedule_finalize_removal(key, delay);
                }
            }
        }

        // 4) Rebuild visual order: desired keys in desired order, plus exiting keys inserted by anchors.
        self.rebuild_order(&desired_keys, &desired_set);

        // 5) Reorder GTK children to match `self.order` (includes exiting).
        let mut prev: Option<gtk::Widget> = None;
        for key in self.order.iter() {
            if let Some(entry) = self.entries.get(key) {
                let w: gtk::Widget = entry.revealer.clone().upcast();
                root.reorder_child_after(&w, prev.as_ref());
                prev = Some(w);
            }
        }
    }

    fn record_exit_anchor(&mut self, key: &Key, desired_set: &HashSet<Key>) {
        // If we already have one, keep it (stable).
        if self.exit_anchors.contains_key(key) {
            return;
        }

        let Some(pos) = self.order.iter().position(|k| k == key) else {
            self.exit_anchors.insert(
                key.clone(),
                ExitAnchor { prev: None, next: None },
            );
            return;
        };

        let prev = self.order[..pos]
            .iter()
            .rev()
            .find(|k| desired_set.contains(*k))
            .cloned();

        let next = self.order[pos + 1..]
            .iter()
            .find(|k| desired_set.contains(*k))
            .cloned();

        self.exit_anchors.insert(key.clone(), ExitAnchor { prev, next });
    }

    fn rebuild_order(&mut self, desired_keys: &[Key], desired_set: &HashSet<Key>) {
        // Base: desired order (allows new keys in top/middle naturally)
        let mut new_order: Vec<Key> = desired_keys.to_vec();

        // Exiting keys in current visual order (stable when multiple exits)
        let exiting_in_visual_order: Vec<Key> = self
            .order
            .iter()
            .filter(|k| self.exiting.contains(*k) && !desired_set.contains(*k))
            .cloned()
            .collect();

        for key in exiting_in_visual_order {
            if new_order.iter().any(|k| k == &key) {
                continue;
            }

            let insert_at = match self.exit_anchors.get(&key) {
                Some(a) => {
                    if let Some(prev) = &a.prev {
                        if let Some(i) = new_order.iter().position(|k| k == prev) {
                            i + 1
                        } else if let Some(next) = &a.next {
                            new_order.iter().position(|k| k == next).unwrap_or(new_order.len())
                        } else {
                            new_order.len()
                        }
                    } else if let Some(next) = &a.next {
                        new_order.iter().position(|k| k == next).unwrap_or(0)
                    } else {
                        new_order.len()
                    }
                }
                None => new_order.len(),
            };

            new_order.insert(insert_at.min(new_order.len()), key);
        }

        self.order = new_order;
    }

    fn schedule_finalize_removal(&self, key: Key, delay: Duration) {
        let tx = self.finalize_tx.clone();
        glib::timeout_add_local_once(delay, move || {
            // Ignore error if component is gone.
            let _ = tx.send(DynamicBoxInput::FinalizeRemoval(key));
        });
    }

    fn finalize_removal(&mut self, root: &gtk::Box, key: Key) {
        // Only remove if it is still exiting. (It might have been re-added meanwhile.)
        if !self.exiting.remove(&key) {
            return;
        }

        self.exit_anchors.remove(&key);
        self.order.retain(|k| k != &key);

        if self.retain_entries {
            // Keep the entry alive but hide it from layout
            if let Some(entry) = self.entries.get(&key) {
                entry.revealer.set_visible(false);
                self.hidden.insert(key);
            }
        } else {
            if let Some(entry) = self.entries.remove(&key) {
                root.remove(&entry.revealer);
                entry.revealer.unparent();
            }
        }
    }

    pub fn for_each_entry(&self, mut f: impl FnMut(&Key, &Entry)) {
        for (k, e) in &self.entries {
            f(k, e);
        }
    }
}

// ---- Example usage -----------------------------------------------------------
//
// Suppose you have Item = MyRowData and you want a child component MyRowComp.
//
// let factory = DynamicBoxFactory::<MyRowData, u64> {
//     id: Box::new(|item| item.id),
//     create: Box::new(|item| {
//         let ctrl: Controller<MyRowComp> = MyRowComp::builder().launch(item.clone().into());
//         Box::new(ctrl) as Box<dyn GenericWidgetController>
//     }),
//     update: None, // Prefer typed updates; see note below.
// };
//
// let dynamic: Controller<DynamicBoxModel<MyRowData, u64>> =
//     DynamicBoxModel::builder().launch(DynamicBoxInit {
//         factory,
//         orientation: gtk::Orientation::Vertical,
//         spacing: 8,
//         transition_type: gtk::RevealerTransitionType::Crossfade,
//         transition_duration_ms: 220,
//         reverse: false,
//     });
//
// // Later:
// dynamic.sender().send(DynamicBoxInput::SetItems(new_items)).unwrap();