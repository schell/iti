//! [`TabListItem`] — the clickable tab-header component used by [`TabPanel`].
//!
//! [`TabListItem`] implements [`Step`], [`StepWith`], and [`StepWithMut`],
//! yielding [`TabListItemEvent`] (`Click`, `Close`, or `User(Ev)`).
//!
//! - Use [`Step`] when you only need to detect clicks and close-button clicks.
//! - Use [`StepWith`] / [`StepWithMut`] to race those against a user-supplied
//!   future over the tab's inner content (`&T` / `&mut T`).
//!
//! # Event types
//!
//! - [`TabListItemEvent`]: public event enum (`Click`, `Close`, `User(Ev)`).
//! - [`TabListItemEventData`]: shared payload (id, index, DOM event).
//!
//! # Relation to `TabPanel`
//!
//! `TabPanel` stores `TabListItem` inside each `TabPanelEntry`. `TabPanel`'s
//! own `StepWithMut` closure receives `&mut TabOrSpacer` and can delegate
//! to `TabListItem`'s step impls or race the tab's listeners manually.

use std::future::Future;

use futures_lite::FutureExt;
use mogwai::{future::MogwaiFutureExt, prelude::*};

use crate::id::Id;

/// Default spacer content — renders nothing. Used as the `S` type parameter
/// default for [`TabListItem`], [`super::entry::TabSpacer`], and
/// [`super::TabPanel`] when no custom spacer content is needed.
pub struct EmptySpacer;

impl Default for EmptySpacer {
    fn default() -> Self {
        EmptySpacer
    }
}

impl<V: View> ViewChild<V> for EmptySpacer {
    fn as_append_arg(&self) -> AppendArg<V, impl Iterator<Item = std::borrow::Cow<'_, V::Node>>> {
        AppendArg::new(std::iter::empty())
    }
}

/// A single tab within a [`super::TabPanel`].
///
/// Generic over the view type `V`, the tab's inner content type `T`, and the
/// spacer content type `S` (shared with [`super::entry::TabSpacer`] for the id
/// pool). The `is_active` flag is a `Proxy<bool>` that reactively toggles the
/// `nav-link active` CSS class on the underlying `<a>` element.
///
/// Each tab may optionally show a close button on the right side of the tab
/// label. The close button uses the `title-bar-close` CSS class (the same
/// Platinum close-box used by [`crate::components::title_bar::TitleBar`]).
/// When clicked, it emits [`TabListItemEvent::Close`] from the owning
/// [`super::TabPanel`].
///
/// Constructed internally by [`super::TabPanel::push`] /
/// [`super::TabPanel::insert`]; users rarely call [`TabListItem::new`]
/// directly.
#[derive(ViewChild, ViewProperties)]
pub struct TabListItem<V: View, T = <V as View>::Element, S = EmptySpacer> {
    #[child]
    #[properties]
    li: V::Element,
    on_click: V::EventListener,
    close_click: V::EventListener,
    close_visible: Proxy<bool>,
    closable: bool,
    inner: T,
    is_active: Proxy<bool>,
    id: Id<(T, S)>,
    index: usize,
}

impl<V: View, T, S> TabListItem<V, T, S> {
    /// Returns a reference to the tab's inner content.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the tab's inner content.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Get a reference to this item's [`Id`].
    pub fn id(&self) -> &Id<(T, S)> {
        &self.id
    }

    /// Whether this tab is currently the active (selected) tab.
    pub fn is_active(&self) -> bool {
        *self.is_active
    }

    /// Get a reference to the tab's root `<li>` element.
    pub(in crate::components::tab) fn li(&self) -> &V::Element {
        &self.li
    }

    /// Set the tab's active state (toggles the `active` CSS class).
    pub(in crate::components::tab) fn set_is_active(&mut self, active: bool) {
        self.is_active.set(active);
    }

    /// Update the cached entry index.
    pub(in crate::components::tab) fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    /// Consume the tab item, returning its [`Id`] and inner content.
    pub(in crate::components::tab) fn into_parts(self) -> (Id<(T, S)>, T) {
        (self.id, self.inner)
    }

    /// Get a reference to this tab's click event listener.
    pub fn on_click(&self) -> &V::EventListener {
        &self.on_click
    }

    /// Get a reference to this tab's close-button click event listener.
    pub fn close_listener(&self) -> &V::EventListener {
        &self.close_click
    }

    /// Show or hide this tab's close button.
    pub fn set_closable(&mut self, closable: bool) {
        self.closable = closable;
        self.close_visible.set(closable);
    }

    /// Whether this tab's close button is visible.
    pub fn is_closable(&self) -> bool {
        self.closable
    }
}

impl<V: View, T: ViewChild<V> + 'static, S: 'static> TabListItem<V, T, S> {
    /// Create a new tab item with the given [`Id`], entry index, and inner content.
    ///
    /// The close button is hidden by default. Use
    /// [`TabListItem::set_closable`] to show it.
    ///
    /// This is called by [`super::TabPanel::push`] / [`super::TabPanel::insert`]; you
    /// usually don't construct `TabListItem` values yourself.
    ///
    /// The `index` is the entry's position in the parent `TabPanel`'s `Vec`; it
    /// is cached so the step impls can report it in [`TabListItemEventData`]
    /// without a lookup.
    pub fn new(id: Id<(T, S)>, index: usize, inner: T) -> Self {
        let mut is_active = Proxy::new(false);
        let mut close_visible = Proxy::new(false);
        rsx! {
            let li = li(class = "nav-item", style:cursor = "pointer") {
                let a = a(
                    class = is_active(active => if *active {
                        "nav-link active"
                    } else {
                        "nav-link"
                    }),
                    on:click = on_click,
                ) {
                    button(
                        type = "button",
                        class = "title-bar-close nav-tab-close",
                        style:display = close_visible(v => if *v {
                            "inline-block"
                        } else {
                            "none"
                        }),
                        on:click = close_click,
                    ) {}
                    {&inner}
                }
            }
        }

        Self {
            li,
            on_click,
            close_click,
            close_visible,
            closable: false,
            inner,
            is_active,
            id,
            index,
        }
    }
}

/// Shared payload for tab-item click and close events.
///
/// Carries the tab's [`Id`], its cached entry index, and the underlying DOM
/// event. Produced internally by the step impls; matched via
/// [`TabListItemEvent::Click`] / [`TabListItemEvent::Close`].
pub struct TabListItemEventData<V: View, T = <V as View>::Element, S = EmptySpacer> {
    pub id: Id<(T, S)>,
    pub index: usize,
    pub event: V::Event,
}

/// Event produced by stepping a [`TabListItem`].
///
/// - [`Click`](Self::Click): the tab was clicked.
/// - [`Close`](Self::Close): the tab's close button was clicked.
/// - [`User`](Self::User): a user-domain event from the closure passed to
///   [`StepWith`] / [`StepWithMut`]. Unreachable when using plain [`Step`].
pub enum TabListItemEvent<V: View, T = <V as View>::Element, S = EmptySpacer, Ev = ()> {
    Click(TabListItemEventData<V, T, S>),
    Close(TabListItemEventData<V, T, S>),
    User(Ev),
}

/// Race a tab's click and close-button click listeners, returning the first
/// to fire as a [`TabListItemEvent`].
async fn step_impl<V: View, T, S: 'static, Ev: 'static>(
    on_click: &V::EventListener,
    on_close: &V::EventListener,
    id: Id<(T, S)>,
    index: usize,
) -> TabListItemEvent<V, T, S, Ev> {
    let click_ev = on_click.next().map(Ok).boxed_local();
    let close_ev = on_close.next().map(Err).boxed_local();

    let mk_ev_data = |ev| TabListItemEventData {
        id: id.clone(),
        index,
        event: ev,
    };

    match click_ev.or(close_ev).await {
        Ok(ev) => TabListItemEvent::Click(mk_ev_data(ev)),
        Err(ev) => TabListItemEvent::Close(mk_ev_data(ev)),
    }
}

/// Plain [`Step`] for [`TabListItem`]: races the click and close-button
/// listeners. The [`User`](TabListItemEvent::User) variant is unreachable.
/// Use [`StepWith`] / [`StepWithMut`] to race a user-domain future alongside.
impl<V: View, T: 'static, S: 'static> Step for TabListItem<V, T, S> {
    type Output = TabListItemEvent<V, T, S>;

    async fn step(&self) -> Self::Output {
        step_impl(
            &self.on_click,
            &self.close_click,
            self.id.clone(),
            self.index,
        )
        .await
    }
}

/// [`StepWith<T>`] for [`TabListItem`]: races the tab's click/close event
/// against a user-supplied future over `&T` (the tab's inner content).
/// Suitable when the inner content's own step takes `&self`. Use
/// [`StepWithMut`] when `&mut self` is needed.
impl<V: View, T: 'static, S: 'static> StepWith<T> for TabListItem<V, T, S> {
    type Output<Ev: 'static> = TabListItemEvent<V, T, S, Ev>;

    async fn step_with<Ev>(
        &self,
        mut f: impl for<'a> FnMut(&'a T) -> std::pin::Pin<Box<dyn Future<Output = Ev> + 'a>>,
    ) -> Self::Output<Ev>
    where
        Ev: 'static,
    {
        let step = self.step().map(Ok);
        let user = f(&self.inner).map(Err);

        match step.or(user).await {
            Ok(ev) => match ev {
                TabListItemEvent::Click(d) => TabListItemEvent::Click(d),
                TabListItemEvent::Close(d) => TabListItemEvent::Close(d),
                TabListItemEvent::User(()) => {
                    unreachable!("step only returns click or close variants")
                }
            },
            Err(ev) => TabListItemEvent::User(ev),
        }
    }
}

/// [`StepWithMut<T>`] for [`TabListItem`]: same as [`StepWith`] but the user
/// closure receives `&mut T` (mutable access to the tab's inner content).
impl<V: View, T: 'static, S: 'static> StepWithMut<T> for TabListItem<V, T, S> {
    type Output<Ev: 'static> = TabListItemEvent<V, T, S, Ev>;

    async fn step_with_mut<Ev>(
        &mut self,
        mut f: impl for<'a> FnMut(&'a mut T) -> std::pin::Pin<Box<dyn Future<Output = Ev> + 'a>>,
    ) -> Self::Output<Ev>
    where
        Ev: 'static,
    {
        let step = step_impl(
            &self.on_click,
            &self.close_click,
            self.id.clone(),
            self.index,
        )
        .map(Ok);
        let user = f(&mut self.inner).map(Err);

        match step.or(user).await {
            Ok(ev) => match ev {
                TabListItemEvent::Click(d) => TabListItemEvent::Click(d),
                TabListItemEvent::Close(d) => TabListItemEvent::Close(d),
                TabListItemEvent::User(()) => {
                    unreachable!("step only returns click or close variants")
                }
            },
            Err(ev) => TabListItemEvent::User(ev),
        }
    }
}
