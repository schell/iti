//! Section container with a dashed border and configurable title.
//!
//! A reusable section component that wraps content in a dashed-bordered
//! fieldset with a title rendered as a `<legend>`. Two style variants are
//! provided:
//!
//! - [`SectionStyle::Titled`] — the legend sits above the dashed border.
//! - [`SectionStyle::Fieldset`] — the legend sits embedded in the top border,
//!   like a native fieldset.
//!
//! The section is generic over the title type `T` and the content type `C`,
//! both of which must implement [`ViewChild`] and [`StepMut`]. The title's
//! [`StepMut::step_mut`] returns a `bool` indicating whether the content
//! should be visible.
use std::future::Future;
use std::pin::Pin;

use futures_lite::FutureExt;
use mogwai::future::MogwaiFutureExt;
use mogwai::prelude::*;

/// How the section title is positioned relative to the dashed border.
pub enum SectionStyle {
    /// Title sits above the dashed border, as a standalone heading.
    Titled,
    /// Title sits embedded in the top border, like a native fieldset legend.
    Fieldset,
}

/// A dashed section container with a title and content.
///
/// Generic over the title type `T` (which must produce a `bool` from
/// [`StepMut`] to toggle content visibility) and the content type `C`.
#[derive(ViewChild)]
pub struct Section<
    V: View,
    T: ViewChild<V> + StepMut<Output = bool>,
    C: ViewChild<V> + StepMut<Output = ()>,
> {
    #[child]
    fieldset: V::Element,
    title: T,
    content: C,
    enabled: Proxy<bool>,
}

impl<V, T, C> Section<V, T, C>
where
    V: View,
    T: ViewChild<V> + StepMut<Output = bool>,
    C: ViewChild<V> + StepMut<Output = ()>,
{
    /// Create a new section.
    ///
    /// # Arguments
    ///
    /// * `style` — whether the title sits above or embedded in the border.
    /// * `color` — CSS color value for the border and title text.
    /// * `title` — the title component (e.g. a checkbox + label).
    /// * `content` — the body content.
    pub fn new(style: SectionStyle, color: impl AsRef<str>, title: T, content: C) -> Self {
        let color = color.as_ref().to_string();
        let class = match style {
            SectionStyle::Titled => "section-titled",
            SectionStyle::Fieldset => "section-fieldset",
        };
        let mut enabled = Proxy::new(true);

        rsx! {
            let fieldset = fieldset(
                class = class,
            ) {
                legend(class = "section-legend") {
                    {&title}
                }
                let body = div(
                    class = "section-body",
                    style:display = enabled(is_enabled => if *is_enabled {
                        "block"
                    } else {
                        "none"
                    })
                ) {
                    {&content}
                }
            }
        }

        fieldset.set_style("--section-color", &color);

        Self {
            fieldset,
            title,
            content,
            enabled,
        }
    }
}

impl<V, T, C> StepMut for Section<V, T, C>
where
    V: View,
    T: ViewChild<V> + StepMut<Output = bool>,
    C: ViewChild<V> + StepMut<Output = ()>,
{
    type Output = ();
    async fn step_mut(&mut self) {
        enum StepEv {
            Top(bool),
            Content,
        }

        let top = self.title.step_mut().map(StepEv::Top);
        let content = self.content.step_mut().map(|_| StepEv::Content);

        match top.or(content).await {
            StepEv::Content => {}
            StepEv::Top(enabled) => {
                self.enabled.set(enabled);
            }
        }
    }
}

/// Trait for type-erased section entries, so a collection can hold sections
/// with different title/content types.
pub trait SectionEntry<V: View> {
    /// Returns the root DOM element.
    fn element(&self) -> &V::Element;
    /// Advance the section by one event.
    fn step(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<V, T, C> SectionEntry<V> for Section<V, T, C>
where
    V: View,
    T: ViewChild<V> + StepMut<Output = bool> + 'static,
    C: ViewChild<V> + StepMut<Output = ()> + 'static,
{
    fn element(&self) -> &V::Element {
        &self.fieldset
    }

    fn step(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(self.step_mut())
    }
}

/// Static (non-interactive) content wrapper for use as the content type `C`.
///
/// Wraps a `V::Element` and implements [`StepMut`] by pending forever
/// (no events to drive).
#[derive(ViewChild)]
pub struct StaticContent<V: View> {
    #[child]
    element: V::Element,
}

impl<V: View> StaticContent<V> {
    pub fn new(element: V::Element) -> Self {
        Self { element }
    }
}

impl<V: View> StepMut for StaticContent<V> {
    type Output = ();
    async fn step_mut(&mut self) {
        futures_lite::future::pending().await
    }
}

/// Plain text title with no toggle.
///
/// Implements [`StepMut`] by pending forever (no clicks to handle),
/// always returning `true` (content visible).
#[derive(ViewChild)]
pub struct PlainTextTitle<V: View> {
    #[child]
    element: V::Element,
}

impl<V: View> PlainTextTitle<V> {
    pub fn new(text: impl AsRef<str>) -> Self {
        let t = text.as_ref().to_string();
        rsx! {
            let element = span() { {V::Text::new(t)} }
        }
        Self { element }
    }
}

impl<V: View> StepMut for PlainTextTitle<V> {
    type Output = bool;
    async fn step_mut(&mut self) -> bool {
        futures_lite::future::pending().await
    }
}
