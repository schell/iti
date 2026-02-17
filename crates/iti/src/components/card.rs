//! Card component.
//!
//! A Bootstrap card container with optional header, body, and footer sections.
use mogwai::prelude::*;

/// A Bootstrap card.
///
/// Provides a structured container with optional header, body, and footer
/// sections. Each section can hold arbitrary content via [`ProxyChild`].
#[derive(ViewChild)]
pub struct Card<V: View> {
    #[child]
    div: V::Element,
    header: V::Element,
    body: V::Element,
    footer: V::Element,
    header_child: ProxyChild<V>,
    body_child: ProxyChild<V>,
    footer_child: ProxyChild<V>,
}

impl<V: View> Card<V> {
    pub fn new() -> Self {
        rsx! {
            let header_placeholder = span() {}
        }
        rsx! {
            let body_placeholder = span() {}
        }
        rsx! {
            let footer_placeholder = span() {}
        }

        let header_child = ProxyChild::new(&header_placeholder);
        let body_child = ProxyChild::new(&body_placeholder);
        let footer_child = ProxyChild::new(&footer_placeholder);

        rsx! {
            let div = div(class = "card") {
                let header = div(class = "card-header") {
                    {&header_child}
                }
                let body = div(class = "card-body") {
                    {&body_child}
                }
                let footer = div(class = "card-footer") {
                    {&footer_child}
                }
            }
        }

        Self {
            div,
            header,
            body,
            footer,
            header_child,
            body_child,
            footer_child,
        }
    }

    /// Replace the header content.
    pub fn set_header(&mut self, content: &impl ViewChild<V>) {
        self.header_child.replace(&self.header, content);
    }

    /// Replace the body content.
    pub fn set_body(&mut self, content: &impl ViewChild<V>) {
        self.body_child.replace(&self.body, content);
    }

    /// Replace the footer content.
    pub fn set_footer(&mut self, content: &impl ViewChild<V>) {
        self.footer_child.replace(&self.footer, content);
    }

    /// Hide the header section.
    pub fn hide_header(&self) {
        self.header.set_style("display", "none");
    }

    /// Show the header section.
    pub fn show_header(&self) {
        self.header.remove_style("display");
    }

    /// Hide the footer section.
    pub fn hide_footer(&self) {
        self.footer.set_style("display", "none");
    }

    /// Show the footer section.
    pub fn show_footer(&self) {
        self.footer.remove_style("display");
    }
}

impl<V: View> Default for Card<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "library")]
pub mod library {
    use mogwai::prelude::*;

    use super::*;

    #[derive(ViewChild)]
    pub struct CardLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
    }

    impl<V: View> Default for CardLibraryItem<V> {
        fn default() -> Self {
            let mut card = Card::new();

            rsx! {
                let header_text = strong() { "Card Header" }
            }
            card.set_header(&header_text);

            rsx! {
                let body_content = div() {
                    h5(class = "card-title") { "Card Title" }
                    p(class = "card-text") {
                        "Some quick example text to build on the card title and \
                         make up the bulk of the card\u{2019}s content."
                    }
                }
            }
            card.set_body(&body_content);

            rsx! {
                let footer_text = small(class = "text-body-secondary") {
                    "Last updated 3 mins ago"
                }
            }
            card.set_footer(&footer_text);

            rsx! {
                let wrapper = div(style:max_width = "24rem") {
                    {&card}
                }
            }

            Self { wrapper }
        }
    }
}
