//! Window / page panes.
//!
//! A type for storing panes of HTML and their logic.
//!
//! A "pane" is a spot of HTML suitable for storing in a collection, where only
//! one item in that collection is visible at a time.
//!
//! Think of the content represented by a tab.
use std::collections::HashMap;

use mogwai::prelude::*;

use crate::id::{Id, IdPool};

/// Controls how [`Panes`] shows and hides pane content.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneMode {
    /// Swap DOM nodes via [`ProxyChild::replace`] (default).
    ///
    /// The previously visible pane's DOM subtree is removed from the document,
    /// so transient state (scroll position, iframe content, form input, etc.)
    /// is lost on every switch.
    #[default]
    Replace,
    /// Keep all panes in the DOM, toggling `display: none` on wrapper `div`
    /// slots.
    ///
    /// Every pane is appended to the container once (inside its own wrapper
    /// `div`) and never removed. Switching panes only changes which slot is
    /// visible, preserving scroll position, iframe state, and any other
    /// transient DOM state.
    Retain,
}

/// Result of removing an item from [`Panes`].
pub struct PaneItemRemoval<T> {
    /// [`Id`] of the item removed.
    pub id: Id<T>,
    /// The item that was removed.
    pub item: T,
    /// Whether or not the pane was active when it was removed.
    pub was_selected: bool,
}

/// Static panes container.
///
/// Stores panes as concrete values. Visibility is controlled by the
/// [`PaneMode`] chosen at construction time:
///
/// * [`PaneMode::Replace`] (the default, via [`Panes::new`]) — swaps DOM nodes
///   with [`ProxyChild::replace`].
/// * [`PaneMode::Retain`] (via [`Panes::new_retained`]) — keeps every pane in
///   the DOM inside a wrapper `div` and toggles `display: none`.
#[derive(ViewChild, ViewProperties)]
pub struct Panes<V: View, T> {
    #[child]
    #[properties]
    wrapper: V::Element,
    mode: PaneMode,
    id_pool: IdPool<T>,
    current_id: Option<Id<T>>,
    child: ProxyChild<V>,
    slots: HashMap<Id<T>, V::Element>,
    default_slot: Option<V::Element>,
    default_pane: T,
    panes: HashMap<Id<T>, T>,
}

impl<V: View, T: ViewChild<V>> Panes<V, T> {
    /// Create a new panes container using [`PaneMode::Replace`].
    ///
    /// The given `pane` is shown as the default content. When [`select`] is
    /// called the default content is replaced with the selected pane's DOM
    /// nodes via [`ProxyChild::replace`].
    ///
    /// [`select`]: Panes::select
    pub fn new(wrapper: V::Element, pane: T) -> Self {
        let child = ProxyChild::new(&pane);
        wrapper.append_child(&child);
        Self {
            wrapper,
            mode: PaneMode::Replace,
            id_pool: IdPool::default(),
            current_id: None,
            child,
            slots: HashMap::new(),
            default_slot: None,
            default_pane: pane,
            panes: HashMap::new(),
        }
    }

    /// Create a new panes container using [`PaneMode::Retain`].
    ///
    /// Every pane (including the default) is wrapped in a `div` slot element
    /// and appended to the container once. Switching panes toggles
    /// `display: none` on the slot wrappers so that DOM state (scroll position,
    /// iframe content, etc.) is preserved across switches.
    pub fn new_retained(wrapper: V::Element, pane: T) -> Self {
        let default_slot = V::Element::new("div");
        default_slot.append_child(&pane);
        wrapper.set_style("display", "flex");
        wrapper.set_style("flex-direction", "column");
        wrapper.append_child(&default_slot);

        // ProxyChild is unused in Retain mode but we need a value for the
        // field. Create it from an empty text node so it holds no meaningful
        // DOM state.
        let placeholder = V::Text::new("");
        let child = ProxyChild::new(&placeholder);

        Self {
            wrapper,
            mode: PaneMode::Retain,
            id_pool: IdPool::default(),
            current_id: None,
            child,
            slots: HashMap::new(),
            default_slot: Some(default_slot),
            default_pane: pane,
            panes: HashMap::new(),
        }
    }

    /// Returns the [`PaneMode`] this container was created with.
    pub fn mode(&self) -> PaneMode {
        self.mode
    }

    /// Add a pane to the container.
    ///
    /// In [`PaneMode::Retain`], the pane is immediately appended to the DOM
    /// inside a hidden wrapper `div`.
    ///
    /// Returns the [`Id`] allocated for this pane, which can be used with
    /// [`select`] to show or access this pane later.
    ///
    /// [`select`]: Panes::select
    pub fn add_pane(&mut self, pane: T) -> Id<T> {
        let id = self.id_pool.get_id();
        if self.mode == PaneMode::Retain {
            let slot = V::Element::new("div");
            slot.set_style("display", "none");
            slot.set_style("flex", "1");
            slot.set_style("min-height", "0");
            slot.append_child(&pane);
            self.wrapper.append_child(&slot);
            self.slots.insert(id.clone(), slot);
        }
        self.panes.insert(id.clone(), pane);
        id
    }

    /// Show the pane with the given `id`, hiding the previously active pane.
    ///
    /// In [`PaneMode::Replace`], the selected pane's DOM nodes replace the
    /// current content via [`ProxyChild::replace`].
    ///
    /// In [`PaneMode::Retain`], the previously active slot gets
    /// `display: none` and the newly active slot has that style removed.
    ///
    /// Returns `true` if the pane was found and selection changed, `false` otherwise.
    pub fn select(&mut self, id: &Id<T>) -> bool {
        if Some(id) != self.current_id.as_ref() {
            match self.mode {
                PaneMode::Replace => {
                    if let Some(pane) = self.panes.get(id) {
                        self.current_id = Some(id.clone());
                        self.child.replace(&self.wrapper, pane);
                        return true;
                    }
                }
                PaneMode::Retain => {
                    if self.panes.contains_key(id) {
                        // Hide the currently active slot.
                        if let Some(old_id) = &self.current_id {
                            if let Some(slot) = self.slots.get(old_id) {
                                slot.set_style("display", "none");
                            }
                        } else if let Some(default_slot) = &self.default_slot {
                            default_slot.set_style("display", "none");
                        }

                        // Show the newly selected slot.
                        if let Some(slot) = self.slots.get(id) {
                            slot.remove_style("display");
                        }
                        self.current_id = Some(id.clone());
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Returns a reference to the currently visible pane.
    pub fn current_pane(&self) -> Option<&T> {
        match &self.current_id {
            Some(id) => self.panes.get(id).or(Some(&self.default_pane)),
            None => Some(&self.default_pane),
        }
    }

    /// Returns a mutable reference to the currently visible pane.
    pub fn current_pane_mut(&mut self) -> Option<&mut T> {
        match &self.current_id {
            Some(id) => {
                if self.panes.contains_key(id) {
                    Some(self.panes.get_mut(id).unwrap())
                } else {
                    Some(&mut self.default_pane)
                }
            }
            None => Some(&mut self.default_pane),
        }
    }

    /// Returns a reference to the pane with the given `id`, if it exists.
    pub fn get_pane(&self, id: &Id<T>) -> Option<&T> {
        self.panes.get(id)
    }

    /// Returns a mutable reference to the pane with the given `id`, if it exists.
    pub fn get_pane_mut(&mut self, id: &Id<T>) -> Option<&mut T> {
        self.panes.get_mut(id)
    }

    /// Remove the pane with the given [`Id`], if any.
    pub fn remove_by_id(&mut self, id: &Id<T>) -> Option<PaneItemRemoval<T>> {
        let pane = self.panes.remove(id)?;
        let was_selected = self.current_id.as_ref() == Some(id);
        Some(PaneItemRemoval {
            id: id.clone(),
            item: pane,
            was_selected,
        })
    }

    /// Returns an iterator over all panes.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.panes.values()
    }

    /// Returns a mutable iterator over all panes.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.panes.values_mut()
    }
}

/// selected it is re-created from the factory, ensuring a fresh state.
#[derive(ViewChild, ViewProperties)]
pub struct RestartPanes<V: View, T> {
    #[child]
    #[properties]
    wrapper: V::Element,
    id_pool: IdPool<T>,
    current_id: Option<Id<T>>,
    child: ProxyChild<V>,
    pane: T,
    panes: HashMap<Id<T>, Box<dyn FnMut() -> T>>,
}

impl<V: View, T: ViewChild<V>> RestartPanes<V, T> {
    /// Create a new factory-based panes container.
    ///
    /// The given `default_pane` is shown initially. Use [`add_pane`] to add
    /// pane factories that will be recreated each time they are selected.
    ///
    /// [`add_pane`]: RestartPanes::add_pane
    pub fn new(wrapper: V::Element, default_pane: T) -> Self {
        let child = ProxyChild::new(&default_pane);
        wrapper.append_child(&child);
        Self {
            wrapper,
            id_pool: IdPool::default(),
            current_id: None,
            child,
            pane: default_pane,
            panes: HashMap::new(),
        }
    }

    /// Add a pane factory to the container.
    ///
    /// The factory is a closure that creates a new pane each time this pane is
    /// selected. If this is the first pane added, it is automatically selected.
    ///
    /// Returns the [`Id`] allocated for this pane factory, which can be used
    /// with [`select`] to show this pane.
    ///
    /// [`select`]: RestartPanes::select
    pub fn add_pane(&mut self, create: impl FnMut() -> T + 'static) -> Id<T> {
        let id = self.id_pool.get_id();
        let was_empty = self.panes.is_empty();
        self.panes.insert(id.clone(), Box::new(create));
        if was_empty {
            log::info!("selecting first pane");
            let _ = self.select(&id);
        }
        id
    }

    /// Show the pane with the given `id`, hiding the previously active pane.
    ///
    /// The pane is recreated fresh from its factory. Returns `true` if the
    /// pane was found and selected, `false` otherwise.
    pub fn select(&mut self, id: &Id<T>) -> bool {
        if Some(id) != self.current_id.as_ref() {
            if let Some(f) = self.panes.get_mut(id) {
                let pane = f();
                self.pane = pane;
                self.child.replace(&self.wrapper, &self.pane);
                self.current_id = Some(id.clone());
                return true;
            }
        }
        false
    }

    /// Returns a reference to the currently displayed pane.
    pub fn current_pane(&self) -> &T {
        &self.pane
    }

    /// Returns a mutable reference to the currently displayed pane.
    pub fn current_pane_mut(&mut self) -> &mut T {
        &mut self.pane
    }
}
