//! Window / page panes.
//!
//! A type for storing panes of HTML and their logic.
//!
//! A "pane" is a spot of HTML suitable for storing in a collection, where only
//! one item in that collection is visible at a time.
//!
//! Think of the content represented by a tab.
use mogwai::prelude::*;

/// Static panes container.
///
/// Stores panes as concrete values. [`Panes::select`] swaps the visible child
/// via [`ProxyChild::replace`].
#[derive(ViewChild)]
pub struct Panes<V: View, T> {
    #[child]
    wrapper: V::Element,
    index: Option<usize>,
    child: ProxyChild<V>,
    default_pane: T,
    panes: Vec<T>,
}

impl<V: View, T: ViewChild<V>> Panes<V, T> {
    pub fn new(wrapper: V::Element, pane: T) -> Self {
        let child = ProxyChild::new(&pane);
        wrapper.append_child(&child);
        Self {
            wrapper,
            index: None,
            child,
            default_pane: pane,
            panes: vec![],
        }
    }

    pub fn add_pane(&mut self, pane: T) {
        self.panes.push(pane);
    }

    pub fn select(&mut self, index: usize) {
        if Some(index) != self.index {
            if let Some(pane) = self.panes.get(index) {
                self.index = Some(index);
                self.child.replace(&self.wrapper, pane);
            }
        }
    }

    pub fn get_pane(&self) -> &T {
        match self.index {
            Some(n) => self.panes.get(n).unwrap_or(&self.default_pane),
            None => &self.default_pane,
        }
    }

    pub fn get_pane_mut(&mut self) -> &mut T {
        match self.index {
            Some(n) => {
                if let Some(pane) = self.panes.get_mut(n) {
                    pane
                } else {
                    &mut self.default_pane
                }
            }
            None => &mut self.default_pane,
        }
    }

    pub fn get_pane_at(&self, index: usize) -> Option<&T> {
        self.panes.get(index)
    }

    pub fn get_pane_at_mut(&mut self, index: usize) -> Option<&mut T> {
        self.panes.get_mut(index)
    }
}

/// Factory-based panes container.
///
/// Stores pane *factories* (`Box<dyn FnMut() -> T>`). Each time a pane is
/// selected it is re-created from the factory, ensuring a fresh state.
#[derive(ViewChild)]
pub struct RestartPanes<V: View, T> {
    #[child]
    wrapper: V::Element,
    index: Option<usize>,
    child: ProxyChild<V>,
    pane: T,
    panes: Vec<Box<dyn FnMut() -> T>>,
}

impl<V: View, T: ViewChild<V>> RestartPanes<V, T> {
    pub fn new(wrapper: V::Element, default_pane: T) -> Self {
        let child = ProxyChild::new(&default_pane);
        wrapper.append_child(&child);
        Self {
            wrapper,
            index: None,
            child,
            pane: default_pane,
            panes: vec![],
        }
    }

    pub fn add_pane(&mut self, create: impl FnMut() -> T + 'static) {
        self.panes.push(Box::new(create));
        if self.panes.len() == 1 {
            log::info!("selecting tab 0");
            self.select(0);
        }
    }

    pub fn select(&mut self, index: usize) {
        if Some(index) != self.index {
            if let Some(f) = self.panes.get_mut(index) {
                let pane = f();
                self.pane = pane;
                self.child.replace(&self.wrapper, &self.pane);
                self.index = Some(index);
            }
        }
    }

    pub fn get_pane(&self) -> &T {
        &self.pane
    }

    pub fn get_pane_mut(&mut self) -> &mut T {
        &mut self.pane
    }
}
