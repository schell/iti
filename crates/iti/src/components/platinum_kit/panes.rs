//! Platinum kit sandbox for [`Panes`] in retain mode.
use std::collections::HashMap;

use futures_lite::FutureExt;
use mogwai::{prelude::*, web::WebElement};

use crate::{
    components::{
        button::Button,
        icon::{Icon, IconGlyph, IconSize, IconStyle},
        pane::Panes,
        tab::{TabItemRemoval, TabList, TabListEvent},
    },
    id::Id,
};

/// Platinum kit sandbox demonstrating retained panes.
///
/// Three tabs with scrollable content and a live timer prove that both
/// scroll position and async state survive tab switches.
#[derive(ViewChild)]
pub struct PlatinumKitPanes<V: View> {
    #[child]
    div: V::Element,
    tabs: TabList<V, V::Element>,
    panes: Panes<V, V::Element>,
    tab_ids_to_pane_ids: HashMap<Id<V::Element>, Id<V::Element>>,
    new_item_input: V::Element,
    new_item_button: Button<V>,
    close_icons: Vec<(Id<V::Element>, Icon<V>)>,
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
                let list = {TabList::default()}
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
            let text = V::Text::new(format!("A — paragraph {i} of 20."));
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
            let text = V::Text::new(format!("B — paragraph {i} of 20."));
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
            let text = V::Text::new(format!("Timer — filler paragraph {i} of 15."));
            rsx! { let p = p() { {text} } }
            pane_timer.append_child(&p);
        }

        // -- Assemble ----------------------------------------------------
        rsx! {
            let default_pane = p(class = "text-muted mt-2") {
                "Select a tab above."
            }
        }

        let panes = Panes::new_retained(pane_wrapper, default_pane);

        let mut item = Self {
            div,
            tabs: list,
            panes,
            timer_text,
            seconds: 0,
            new_item_input,
            new_item_button,
            close_icons: vec![],
            tab_ids_to_pane_ids: Default::default(),
        };

        let (tab_a_id, _) = item.add(
            {
                rsx! { let s = span() { "Scrollable A" } }
                s
            },
            pane_a,
        );

        let _ = item.add(
            {
                rsx! { let s = span() { "Scrollable B" } }
                s
            },
            pane_b,
        );

        let _ = item.add(
            {
                rsx! { let s = span() { "Timer" } }
                s
            },
            pane_timer,
        );

        // Show the first pane by default.
        item.select(&tab_a_id);

        item
    }
}

impl<V: View> PlatinumKitPanes<V> {
    fn add(
        &mut self,
        tab_item: V::Element,
        pane_item: V::Element,
    ) -> (Id<V::Element>, Id<V::Element>) {
        let tab_id = self.tabs.push(tab_item);
        let pane_id = self.panes.add_pane(pane_item);
        self.tab_ids_to_pane_ids
            .insert(tab_id.clone(), pane_id.clone());
        (tab_id, pane_id)
    }

    fn select(&mut self, id: &Id<V::Element>) {
        self.tabs.select_by_id(id);
        if let Some(id) = self.tab_ids_to_pane_ids.get(id) {
            let _ = self.panes.select(id);
        }
    }
}

impl<V: View> StepMut for PlatinumKitPanes<V> {
    type Output = ();
    async fn step_mut(&mut self) {
        enum Ev<V: View, T> {
            Timer,
            Tab(TabListEvent<V, T>),
            NewItem(String),
            Remove(Id<V::Element>),
        }
        let timer_fut = async {
            mogwai::time::wait_millis(1000).await;
            Ev::Timer
        };
        let list_fut = async {
            let event = self.tabs.step().await;
            Ev::Tab(event)
        };
        let new_tab_fut = async {
            let _event = self.new_item_button.step().await;
            let s = self
                .new_item_input
                .dyn_el(|el: &web_sys::HtmlInputElement| el.value())
                .unwrap();
            Ev::NewItem(s)
        };
        let closes = self
            .close_icons
            .iter()
            .map(|(id, icon)| async {
                let _ = icon.listen("click").next().await;
                Ev::Remove::<V, V::Element>(id.clone())
            })
            .collect::<Vec<_>>();
        let close_tab_fut = mogwai::future::race_all(closes);
        let result = timer_fut
            .or(list_fut)
            .or(new_tab_fut)
            .or(close_tab_fut)
            .await;
        match result {
            Ev::Tab(TabListEvent::ItemClicked {
                id,
                index: _,
                event: _,
            }) => {
                self.select(&id);
            }
            Ev::Timer => {
                self.seconds += 1;
                self.timer_text
                    .set_text(format!("{} seconds elapsed", self.seconds));
            }
            Ev::NewItem(s) => {
                let close_icon =
                    Icon::with_style(IconGlyph::Xmark, IconSize::Regular, IconStyle::Solid);
                rsx! {
                    let item = div() {
                        {&close_icon}
                        {format!("Tab {}", self.close_icons.len()).into_text::<V>()}
                    }
                }
                rsx! {
                    let pane = div() {
                        span() {
                            {s.into_text::<V>()}
                        }
                    }
                }
                let (tab_id, pane_id) = self.add(item, pane);
                self.close_icons.push((tab_id.clone(), close_icon));

                self.tabs.select_by_id(&tab_id);
                let _ = self.panes.select(&pane_id);
            }
            Ev::Remove(id) => {
                if let Some(TabItemRemoval {
                    id: _,
                    index,
                    item: _,
                    was_selected: true,
                }) = self.tabs.remove_by_id(&id)
                {
                    if let Some(pane_id) = self.tab_ids_to_pane_ids.remove(&id) {
                        let _ = self.panes.remove_by_id(&pane_id);
                    }

                    let selected_index = index.min(self.tab_ids_to_pane_ids.len() - 1);
                    let id = self.tabs.get(selected_index).unwrap().id().clone();
                    self.select(&id);
                }
            }
        }
    }
}
