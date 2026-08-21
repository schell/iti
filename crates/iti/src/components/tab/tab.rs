//! `TabPanel` individual tab or individual spacer.

use std::future::Future;

use mogwai::prelude::*;

use super::item::TabListItem;
use crate::id::Id;

/// A flexible spacer element within a [`TabPanel`].
///
/// Spacers are `flex-grow: 1` elements that absorb available space in the tab
/// bar. Insert them before, after, or between tabs to control alignment.
///
/// `T` is the type of the custom tab view. If no custom view, this is `V::Element`.
/// `S` is the type of the custom spacer view. If none, this defaults to `()`.
///
/// Each spacer has a unique [`Id<(T, S)>`] (allocated from the same pool as tabs)
/// so it can be individually identified and removed via [`TabPanel::remove_by_id`].
#[derive(ViewChild)]
pub struct TabSpacer<V: View, T = <V as View>::Element, S = ()> {
    #[child]
    li: V::Element,
    id: Id<(T, S)>,
    inner: S,
}

impl<V: View, T, S: ViewChild<V>> TabSpacer<V, T, S> {
    fn new(id: Id<(T, S)>, inner: S) -> Self {
        rsx! {
            let li = li(class = "nav-tab-spacer") {
                {&inner}
            }
        }
        Self { li, id, inner }
    }

    /// Get a reference to this spacer's [`Id`].
    pub fn id(&self) -> &Id<(T, S)> {
        &self.id
    }
}

/// Steps the inner custom spacer view witha user-domain specific function.
impl<V: View, T, S> StepWith<S> for TabSpacer<V, T, S> {
    type Output<Ev: 'static> = Ev;

    async fn step_with<Ev>(
        &self,
        mut f: impl for<'a> FnMut(&'a S) -> std::pin::Pin<Box<dyn Future<Output = Ev> + 'a>>,
    ) -> Self::Output<Ev>
    where
        Ev: 'static,
    {
        f(&self.inner).await
    }
}

/// Steps the inner custom spacer view witha user-domain specific function.
impl<V: View, T, S> StepWithMut<S> for TabSpacer<V, T, S> {
    type Output<Ev: 'static> = Ev;

    async fn step_with_mut<Ev>(
        &mut self,
        mut f: impl for<'a> FnMut(&'a mut S) -> std::pin::Pin<Box<dyn Future<Output = Ev> + 'a>>,
    ) -> Self::Output<Ev>
    where
        Ev: 'static,
    {
        f(&mut self.inner).await
    }
}

/// A pane content element and its wrapper slot `<div>`.
///
/// The slot is the DOM element appended to [`TabPanel`]'s content area.
/// Show/hide is controlled by toggling `display: none` on the slot via
/// [`TabbedPane::show`] / [`TabbedPane::hide`].
pub struct TabbedPane<V: View, P> {
    /// The pane content.
    pane: P,
    /// Wrapper `<div>` holding the pane in the DOM.
    /// Toggled `display: none` to show/hide the pane.
    slot: V::Element,
}

impl<V: View, P: ViewChild<V>> TabbedPane<V, P> {
    /// Create a new `TabbedPane` wrapping the given pane content.
    ///
    /// The slot `<div>` is created with `display: none` (hidden), `flex: 1`,
    /// and `min-height: 0` so it fills the content area when shown.
    pub fn new(pane: P) -> Self {
        let slot = V::Element::new("div");
        slot.set_style("display", "none");
        slot.set_style("flex", "1");
        slot.set_style("min-height", "0");
        slot.append_child(&pane);
        Self { pane, slot }
    }

    /// Returns a reference to the pane content.
    pub fn pane(&self) -> &P {
        &self.pane
    }

    /// Returns a mutable reference to the pane content.
    pub fn pane_mut(&mut self) -> &mut P {
        &mut self.pane
    }

    /// Returns a reference to the wrapper slot element (for DOM operations
    /// like `parent.remove_child(&slot)`).
    pub fn slot(&self) -> &V::Element {
        &self.slot
    }

    /// Show this pane (removes `display: none` from the slot).
    pub fn show(&self) {
        self.slot.remove_style("display");
    }

    /// Hide this pane (sets `display: none` on the slot).
    pub fn hide(&self) {
        self.slot.set_style("display", "none");
    }

    /// Consume this `TabbedPane`, returning the pane content and slot element.
    pub fn into_parts(self) -> (P, V::Element) {
        (self.pane, self.slot)
    }
}

/// A tab + pane pair stored in a [`TabPanel`].
///
/// This is the type the [`StepWithMut`] closure receives a mutable reference
/// to, allowing callers to step both the tab `T` and the pane `P` together in
/// a single race. Use [`TabPanelEntry::tab`] / [`TabPanelEntry::tab_mut`] and
/// [`TabPanelEntry::pane`] / [`TabPanelEntry::pane_mut`] to access the
/// components.
///
/// ```ignore
/// panel.step_with_mut(|entry| {
///     let on_click = entry.tab().on_click();
///     let pane = entry.pane_mut();
///     async { /* race on_click.next() vs pane.step_mut() */ }.boxed_local()
/// })
/// ```
pub struct TabPanelEntry<V: View, T, P> {
    tab: TabListItem<V, T>,
    pane: TabbedPane<V, P>,
}

impl<V: View, T, P: ViewChild<V>> TabPanelEntry<V, T, P> {
    /// Returns a reference to the tab list item (the clickable tab header).
    pub fn tab(&self) -> &TabListItem<V, T> {
        &self.tab
    }

    /// Returns a mutable reference to the tab list item.
    pub fn tab_mut(&mut self) -> &mut TabListItem<V, T> {
        &mut self.tab
    }

    /// Returns a reference to the pane content.
    pub fn pane(&self) -> &P {
        self.pane.pane()
    }

    /// Returns a mutable reference to the pane content.
    pub fn pane_mut(&mut self) -> &mut P {
        self.pane.pane_mut()
    }

    /// Borrow the tab and pane simultaneously.
    ///
    /// Returns a shared reference to the tab and a mutable reference to the
    /// pane, allowing both to be used in the same scope (e.g. racing a
    /// tab-click future against a pane-step future in a `step_with_mut`
    /// closure).
    pub fn split_tab_pane(&mut self) -> (&TabListItem<V, T>, &mut P) {
        (&self.tab, self.pane.pane_mut())
    }
}

/// An entry in a [`TabPanel`] — either a tab+pane pair or a spacer.
pub enum TabOrSpacer<V: View, T, P> {
    /// A tab and its associated pane.
    Item(TabPanelEntry<V, T, P>),
    /// A flexible spacer element (no tab, no pane).
    Spacer(TabSpacer<V, T>),
}

impl<V: View, T, P> TabOrSpacer<V, T, P> {
    /// Get the underlying tab `li` element (for Item) or spacer `li` (for
    /// Spacer), for DOM operations.
    fn element(&self) -> &V::Element {
        match self {
            TabOrSpacer::Item(entry) => &entry.tab.li,
            TabOrSpacer::Spacer(spacer) => &spacer.li,
        }
    }

    /// Returns `true` if this entry is a spacer.
    pub fn is_spacer(&self) -> bool {
        matches!(self, TabOrSpacer::Spacer(_))
    }

    /// Try to get the entry as a tab+pane pair reference.
    pub fn as_item(&self) -> Option<&TabPanelEntry<V, T, P>> {
        match self {
            TabOrSpacer::Item(entry) => Some(entry),
            TabOrSpacer::Spacer(_) => None,
        }
    }

    /// Try to get the entry as a mutable tab+pane pair reference.
    pub fn as_item_mut(&mut self) -> Option<&mut TabPanelEntry<V, T, P>> {
        match self {
            TabOrSpacer::Item(entry) => Some(entry),
            TabOrSpacer::Spacer(_) => None,
        }
    }
}

impl<V: View, T: ViewChild<V>, P: ViewChild<V>> ViewChild<V> for TabOrSpacer<V, T, P> {
    fn as_append_arg(&self) -> AppendArg<V, impl Iterator<Item = std::borrow::Cow<'_, V::Node>>> {
        match self {
            TabOrSpacer::Item(entry) => entry.tab.as_boxed_append_arg(),
            TabOrSpacer::Spacer(spacer) => spacer.as_boxed_append_arg(),
        }
    }
}
