//! Platinum kit sandbox for retained panes with closable tabs.
use futures_lite::FutureExt;
use mogwai::{prelude::*, web::WebElement};

use crate::{
    components::{button::Button, tab::TabListEvent, tab::TabPanel},
    id::Id,
};

/// Platinum kit sandbox demonstrating retained panes.
///
/// Three tabs with scrollable content and a live timer prove that both
/// scroll position and async state survive tab switches. All tabs are
/// closable via the built-in close button.
#[derive(ViewChild)]
pub struct PlatinumKitPanes<V: View> {
    #[child]
    div: V::Element,
    tab_panel: TabPanel<V, V::Element, V::Element>,
    new_item_input: V::Element,
    new_item_button: Button<V>,
    timer_text: V::Text,
    seconds: u32,
}

impl<V: View> Default for PlatinumKitPanes<V> {
    fn default() -> Self {
        let new_item_button = {
            let mut b = Button::new("", None);
            b.set_has_icon(true);
            b.get_icon_mut()
                .set_glyph(crate::components::icon::IconGlyph::Plus);
            b
        };
        rsx! {
            let div = div() {
                let pane_wrapper = div() {}

                // TODO: use forms here when they are ready
                div(class = "row container-fluid border-top") {
                    fieldset() {
                        legend() {
                            "Add a new pane"
                        }
                        let new_item_input = input(type = "text") {}
                        {&new_item_button}
                    }
                }
            }
        }

        // -- Scrollable A ------------------------------------------------
        rsx! {
            let pane_a = div(
                style:overflow_y = "auto",
                style:max_height = "200px",
                style:border = "1px solid #dee2e6",
                style:padding = "1rem",
                style:margin_top = "0.5rem",
            ) {
                h5() { "Pane A" }
                p(class = "text-muted") {
                    "Scroll down, switch tabs, then come back."
                    br{}
                    "Your scroll position will be preserved."
                }
            }
        }
        for i in 1..=20 {
            let text = V::Text::new(format!("A - paragraph {i} of 20."));
            rsx! { let p = p() { {text} } }
            pane_a.append_child(&p);
        }

        // -- Scrollable B ------------------------------------------------
        rsx! {
            let pane_b = div(
                style:overflow_y = "auto",
                style:max_height = "200px",
                style:border = "1px solid #dee2e6",
                style:padding = "1rem",
                style:margin_top = "0.5rem",
            ) {
                h5() { "Pane B" }
                p(class = "text-muted") {
                    "This is a different pane with its own scroll state."
                }
            }
        }
        for i in 1..=20 {
            let text = V::Text::new(format!("B - paragraph {i} of 20."));
            rsx! { let p = p() { {text} } }
            pane_b.append_child(&p);
        }

        // -- Timer -------------------------------------------------------
        let timer_text = V::Text::new("0 seconds elapsed");
        rsx! {
            let pane_timer = div(
                style:overflow_y = "auto",
                style:max_height = "200px",
                style:border = "1px solid #dee2e6",
                style:padding = "1rem",
                style:margin_top = "0.5rem",
            ) {
                h5() { "Timer Pane" }
                p(class = "text-muted") {
                    "This timer keeps running even when this tab is hidden."
                    br{}
                    "Scroll down, switch away, then come back."
                }
                p(class = "fw-bold") { {&timer_text} }
            }
        }
        for i in 1..=15 {
            let text = V::Text::new(format!("Timer - filler paragraph {i} of 15."));
            rsx! { let p = p() { {text} } }
            pane_timer.append_child(&p);
        }

        // -- Assemble ----------------------------------------------------
        rsx! {
            let default_pane = p(class = "text-muted mt-2") {
                "Select a tab above."
            }
        }

        let mut tab_panel = TabPanel::new(default_pane);
        tab_panel.set_default_closable(true);

        // Move the tab_panel's window into our div by appending it to
        // pane_wrapper. The TabPanel was constructed with its own window div,
        // so we need to append it as a child.
        pane_wrapper.append_child(&tab_panel);

        let mut item = Self {
            div,
            tab_panel,
            timer_text,
            seconds: 0,
            new_item_input,
            new_item_button,
        };

        rsx! { let tab_a = span() { "Scrollable A" } }
        let tab_a_id = item.tab_panel.push(tab_a, pane_a);

        rsx! { let tab_b = span() { "Scrollable B" } }
        let _ = item.tab_panel.push(tab_b, pane_b);

        rsx! { let tab_timer = span() { "Timer" } }
        let _ = item.tab_panel.push(tab_timer, pane_timer);

        // Show the first pane by default.
        item.tab_panel.select(&tab_a_id);

        item
    }
}

impl<V: View> PlatinumKitPanes<V> {
    fn select(&mut self, id: &Id<V::Element>) {
        self.tab_panel.select(id);
    }
}

impl<V: View> StepMut for PlatinumKitPanes<V> {
    type Output = ();
    async fn step_mut(&mut self) {
        enum Ev<V: View, T, P> {
            Timer,
            Tab(TabListEvent<V, T, P>),
            NewItem(String),
        }

        let timer_fut = async {
            mogwai::time::wait_millis(1000).await;
            Ev::Timer::<V, V::Element, V::Element>
        };
        let list_fut = async {
            let event = self.tab_panel.step_mut().await;
            Ev::Tab(event)
        };
        let new_tab_fut = async {
            let _event = self.new_item_button.step().await;
            let s = self
                .new_item_input
                .dyn_el(|el: &web_sys::HtmlInputElement| el.value())
                .unwrap();
            Ev::NewItem::<V, V::Element, V::Element>(s)
        };

        let result = timer_fut.or(list_fut).or(new_tab_fut).await;
        match result {
            Ev::Tab(TabListEvent::ItemClicked {
                id,
                index: _,
                event: _,
            }) => {
                self.select(&id);
            }
            Ev::Tab(TabListEvent::CloseClicked {
                id: _,
                index: _,
                item: _,
                pane: _,
            }) => {
                // Tab already removed by StepMut; nothing else to do.
            }
            Ev::Timer => {
                self.seconds += 1;
                self.timer_text
                    .set_text(format!("{} seconds elapsed", self.seconds));
            }
            Ev::NewItem(s) => {
                rsx! {
                    let tab = span() {
                        {format!("Tab {}", self.seconds).into_text::<V>()}
                    }
                }
                rsx! {
                    let pane = div() {
                        span() {
                            {s.into_text::<V>()}
                        }
                    }
                }
                let tab_id = self.tab_panel.push(tab, pane);
                self.tab_panel.select(&tab_id);
            }
        }
    }
}
