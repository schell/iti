//! [`TabOrSpacer`], [`TabPanelEntry`], [`TabbedPane`], and [`TabSpacer`] —
//! the entry types stored in a [`TabPanel`]'s `Vec`.
//!
//! [`TabPanel`]: super::TabPanel

use std::future::Future;

use futures_lite::FutureExt;
use mogwai::{prelude::*, step::StepWithMut};

use super::item::{EmptySpacer, TabListItem, TabListItemEvent};
use crate::id::Id;

/// A flexible spacer element within a [`TabPanel`].
///
/// Spacers are `flex-grow: 1` elements that absorb available space in the tab
/// bar. Insert them before, after, or between tabs to control alignment.
///
/// `S` is the type of the spacer's inner content. When the spacer is just a
/// flexible gap, `S` defaults to `()`. For interactive spacers (e.g. a "+"
/// button that creates a new tab), `S` is the button component and
/// [`StepWith`] / [`StepWithMut`] drive its event loop.
///
/// Each spacer has a unique [`Id<(T, S)>`] (allocated from the same pool as
/// tabs) so it can be individually identified and removed via
/// [`TabPanel::remove_by_id`].
///
/// [`TabPanel`]: super::TabPanel
/// [`TabPanel::remove_by_id`]: super::TabPanel::remove_by_id
#[derive(ViewChild)]
pub struct TabSpacer<V: View, T = <V as View>::Element, S = EmptySpacer> {
    #[child]
    li: V::Element,
    id: Id<(T, S)>,
    index: usize,
    inner: S,
}

impl<V: View, T, S: ViewChild<V>> TabSpacer<V, T, S> {
    pub(in crate::components::tab) fn new(id: Id<(T, S)>, index: usize, inner: S) -> Self {
        rsx! {
            let li = li(class = "nav-tab-spacer") {
                {&inner}
            }
        }
        Self {
            li,
            id,
            index,
            inner,
        }
    }
}

impl<V: View, T, S> TabSpacer<V, T, S> {
    /// Get a reference to this spacer's [`Id`].
    pub fn id(&self) -> &Id<(T, S)> {
        &self.id
    }

    /// Returns a reference to the spacer's root `<li>` element.
    pub(in crate::components::tab) fn li(&self) -> &V::Element {
        &self.li
    }

    /// Update the cached entry index.
    pub(in crate::components::tab) fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    /// Returns a reference to the spacer's inner content.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Returns a mutable reference to the spacer's inner content.
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

/// [`StepWith<S>`] for [`TabSpacer`]: delegates entirely to the user closure
/// over `&S` (the spacer's inner content). Spacers have no intrinsic events.
impl<V: View, T, S: 'static> StepWith<S> for TabSpacer<V, T, S> {
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

/// [`StepWithMut<S>`] for [`TabSpacer`]: same as [`StepWith`] but with mutable
/// access to the spacer's inner content.
impl<V: View, T, S: 'static> StepWithMut<S> for TabSpacer<V, T, S> {
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
///
/// [`TabPanel`]: super::TabPanel
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
/// [`TabPanelEntry`] implements [`StepWithMut<P>`], which races the tab's
/// own click/close event (via [`TabListItem::step`]) against a user-supplied
/// future over `&mut P` (the pane content). The result is a
/// [`TabPanelEntryEvent`] discriminating tab vs pane events.
///
/// [`TabPanel`]: super::TabPanel
pub struct TabPanelEntry<V: View, P, T = <V as View>::Element, S = EmptySpacer> {
    tab: TabListItem<V, T, S>,
    pane: TabbedPane<V, P>,
}

/// Event produced by stepping a [`TabPanelEntry`].
///
/// - [`Tab`](Self::Tab): the tab's own click/close event (from
///   [`TabListItemEvent`]).
/// - [`Pane`](Self::Pane): a user-domain event from the closure passed to
///   [`StepWithMut`] on [`TabPanelEntry`].
pub enum TabPanelEntryEvent<V: View, T = <V as View>::Element, S = EmptySpacer, Ev = ()> {
    /// The tab's own event (click or close-button click).
    Tab(TabListItemEvent<V, T, S>),
    /// A pane event from the user closure.
    Pane(Ev),
}

impl<V: View, P, T, S> TabPanelEntry<V, P, T, S> {
    /// Create a new `TabPanelEntry` from a tab and pane.
    pub(in crate::components::tab) fn new(
        tab: TabListItem<V, T, S>,
        pane: TabbedPane<V, P>,
    ) -> Self {
        Self { tab, pane }
    }

    /// Returns a reference to the tab list item (the clickable tab header).
    pub fn tab(&self) -> &TabListItem<V, T, S> {
        &self.tab
    }

    /// Returns a mutable reference to the tab list item.
    pub fn tab_mut(&mut self) -> &mut TabListItem<V, T, S> {
        &mut self.tab
    }

    /// Consume the entry, returning the tab and pane.
    pub(in crate::components::tab) fn into_parts(self) -> (TabListItem<V, T, S>, TabbedPane<V, P>) {
        (self.tab, self.pane)
    }
}

impl<V: View, P: ViewChild<V>, T, S> TabPanelEntry<V, P, T, S> {
    /// Returns a reference to the pane content.
    pub fn pane(&self) -> &P {
        self.pane.pane()
    }

    /// Returns a mutable reference to the pane content.
    pub fn pane_mut(&mut self) -> &mut P {
        self.pane.pane_mut()
    }

    /// Show the pane (removes `display: none` from the slot).
    pub(in crate::components::tab) fn show_pane(&self) {
        self.pane.show();
    }

    /// Hide the pane (sets `display: none` on the slot).
    pub(in crate::components::tab) fn hide_pane(&self) {
        self.pane.hide();
    }
}

/// [`StepWithMut<P>`] for [`TabPanelEntry`]: races the tab's click/close event
/// (via [`TabListItem::step`]) against a user-supplied future over `&mut P`
/// (the pane content).
///
/// The tab event is tagged [`TabPanelEntryEvent::Tab`], the pane future is
/// tagged [`TabPanelEntryEvent::Pane`]. The first to resolve wins.
impl<V: View, P: ViewChild<V>, T: 'static, S: 'static> StepWithMut<P>
    for TabPanelEntry<V, P, T, S>
{
    type Output<Ev: 'static> = TabPanelEntryEvent<V, T, S, Ev>;

    async fn step_with_mut<Ev>(
        &mut self,
        mut f: impl for<'a> FnMut(&'a mut P) -> std::pin::Pin<Box<dyn Future<Output = Ev> + 'a>>,
    ) -> Self::Output<Ev>
    where
        Ev: 'static,
    {
        let tab_fut = async { TabPanelEntryEvent::Tab(self.tab.step().await) }.boxed_local();
        let pane_fut =
            async { TabPanelEntryEvent::Pane(f(self.pane.pane_mut()).await) }.boxed_local();
        tab_fut.or(pane_fut).await
    }
}

/// An entry in a [`TabPanel`] — either a tab+pane pair or a spacer.
///
/// [`TabPanel`]: super::TabPanel
pub enum TabOrSpacer<V: View, P, T = <V as View>::Element, S = EmptySpacer> {
    /// A tab and its associated pane.
    Item(TabPanelEntry<V, P, T, S>),
    /// A flexible spacer element (optionally with inner content).
    Spacer(TabSpacer<V, T, S>),
}

impl<V: View, P, T, S> TabOrSpacer<V, P, T, S> {
    /// Get the underlying element (for DOM operations).
    pub(in crate::components::tab) fn element(&self) -> &V::Element {
        match self {
            TabOrSpacer::Item(entry) => entry.tab().li(),
            TabOrSpacer::Spacer(spacer) => spacer.li(),
        }
    }

    /// Returns `true` if this entry is a spacer.
    pub fn is_spacer(&self) -> bool {
        matches!(self, TabOrSpacer::Spacer(_))
    }

    /// Try to get the entry as a tab+pane pair reference.
    pub fn as_item(&self) -> Option<&TabPanelEntry<V, P, T, S>> {
        match self {
            TabOrSpacer::Item(entry) => Some(entry),
            TabOrSpacer::Spacer(_) => None,
        }
    }

    /// Try to get the entry as a mutable tab+pane pair reference.
    pub fn as_item_mut(&mut self) -> Option<&mut TabPanelEntry<V, P, T, S>> {
        match self {
            TabOrSpacer::Item(entry) => Some(entry),
            TabOrSpacer::Spacer(_) => None,
        }
    }
}

impl<V: View, P, T: ViewChild<V>, S: ViewChild<V>> ViewChild<V> for TabOrSpacer<V, P, T, S> {
    fn as_append_arg(&self) -> AppendArg<V, impl Iterator<Item = std::borrow::Cow<'_, V::Node>>> {
        match self {
            TabOrSpacer::Item(entry) => entry.tab.as_boxed_append_arg(),
            TabOrSpacer::Spacer(spacer) => spacer.as_boxed_append_arg(),
        }
    }
}
