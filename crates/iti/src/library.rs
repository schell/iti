//! Sandboxed component gallery for browsing and testing components in isolation.
use futures_lite::FutureExt;
use js_sys::wasm_bindgen::UnwrapThrowExt;
use mogwai::prelude::*;

use crate::components::{
    alert::library::AlertLibraryItem,
    badge::library::BadgeLibraryItem,
    button::library::ButtonLibraryItem,
    button_group::library::ButtonGroupLibraryItem,
    card::library::CardLibraryItem,
    dropdown::library::DropdownLibraryItem,
    icon::library::IconLibraryItem,
    list::{library::ListLibraryItem, List, ListEvent},
    modal::library::ModalLibraryItem,
    pane::{library::PaneRetainLibraryItem, RestartPanes},
    progress::library::ProgressLibraryItem,
    tab::library::TabListLibraryItem,
    toast::library::ToastLibraryItem,
};

#[derive(ViewChild)]
pub struct LibraryListItem<V: View> {
    #[child]
    label: V::Element,
}

impl<V: View> LibraryListItem<V> {
    pub fn new(title: impl AsRef<str>) -> Self {
        let text = V::Text::new(title);
        rsx! {
            let label = label(
                class = "stretched-link",
                style:cursor = "pointer"
            ) {
                {text}
            }
        }
        Self { label }
    }
}

pub enum LibraryListPane<V: View> {
    Default(V::Element),
    Alert(AlertLibraryItem<V>),
    Badge(BadgeLibraryItem<V>),
    Button(ButtonLibraryItem<V>),
    ButtonGroup(ButtonGroupLibraryItem<V>),
    Card(CardLibraryItem<V>),
    Dropdown(DropdownLibraryItem<V>),
    Icon(IconLibraryItem<V>),
    List(ListLibraryItem<V>),
    Modal(ModalLibraryItem<V>),
    PaneRetain(PaneRetainLibraryItem<V>),
    Progress(ProgressLibraryItem<V>),
    TabList(TabListLibraryItem<V>),
    Toast(ToastLibraryItem<V>),
}

impl<V: View> Default for LibraryListPane<V> {
    fn default() -> Self {
        rsx! {
            let html = p() { "Select a component on the left" }
        }
        LibraryListPane::Default(html)
    }
}

impl<V: View> ViewChild<V> for LibraryListPane<V> {
    fn as_append_arg(
        &self,
    ) -> AppendArg<V, impl Iterator<Item = std::borrow::Cow<'_, <V as View>::Node>>> {
        match self {
            LibraryListPane::Default(el) => el.as_boxed_append_arg(),
            LibraryListPane::Alert(item) => item.as_boxed_append_arg(),
            LibraryListPane::Badge(item) => item.as_boxed_append_arg(),
            LibraryListPane::Button(item) => item.as_boxed_append_arg(),
            LibraryListPane::ButtonGroup(item) => item.as_boxed_append_arg(),
            LibraryListPane::Card(item) => item.as_boxed_append_arg(),
            LibraryListPane::Dropdown(item) => item.as_boxed_append_arg(),
            LibraryListPane::Icon(item) => item.as_boxed_append_arg(),
            LibraryListPane::List(item) => item.as_boxed_append_arg(),
            LibraryListPane::Modal(item) => item.as_boxed_append_arg(),
            LibraryListPane::PaneRetain(item) => item.as_boxed_append_arg(),
            LibraryListPane::Progress(item) => item.as_boxed_append_arg(),
            LibraryListPane::TabList(item) => item.as_boxed_append_arg(),
            LibraryListPane::Toast(item) => item.as_boxed_append_arg(),
        }
    }
}

impl<V: View> LibraryListPane<V> {
    pub async fn step(&mut self) {
        match self {
            LibraryListPane::Alert(item) => item.step().await,
            LibraryListPane::Badge(item) => item.step().await,
            LibraryListPane::Button(item) => item.step().await,
            LibraryListPane::ButtonGroup(item) => item.step().await,
            LibraryListPane::Dropdown(item) => item.step().await,
            LibraryListPane::List(item) => item.step().await,
            LibraryListPane::Modal(item) => item.step().await,
            LibraryListPane::PaneRetain(item) => item.step().await,
            LibraryListPane::Progress(item) => item.step().await,
            LibraryListPane::TabList(item) => item.step().await,
            LibraryListPane::Toast(item) => item.step().await,
            LibraryListPane::Icon(item) => item.step().await,
            LibraryListPane::Default(_) | LibraryListPane::Card(_) => std::future::pending().await,
        }
    }
}

/// The component library gallery.
///
/// Presents a list of all components on the left and the selected component's
/// sandbox on the right. Uses [`RestartPanes`] so each component is freshly
/// recreated when selected.
#[derive(ViewChild)]
pub struct Library<V: View> {
    #[child]
    pub main: V::Element,
    library_list: List<V, LibraryListItem<V>>,
    right_column: RestartPanes<V, LibraryListPane<V>>,
    #[cfg(feature = "system9")]
    theme_toggle_click: V::EventListener,
    #[cfg(feature = "system9")]
    #[allow(dead_code)]
    theme_checkbox: V::Element,
    #[cfg(feature = "system9")]
    theme_enabled: bool,
}

impl<V: View> Default for Library<V> {
    fn default() -> Self {
        rsx! {
            let right_column_wrapper = div(class = "col") {}
        }

        let right_column = RestartPanes::new(right_column_wrapper, LibraryListPane::default());

        #[cfg(feature = "system9")]
        rsx! {
            let theme_checkbox = input(
                type = "checkbox",
                id = "system-9-toggle",
                on:click = theme_toggle_click
            ) {}
        }

        #[cfg(feature = "system9")]
        rsx! {
            let main = main(class = "container-fluid mt-3") {
                div(class = "row") {
                    div(class = "col-auto") {
                        label(class = "system-9-toggle") {
                            {&theme_checkbox}
                            "System 9 Theme"
                        }
                        let library_list = {List::default()}
                    }
                    {&right_column}
                }
            }
        }

        #[cfg(not(feature = "system9"))]
        rsx! {
            let main = main(class = "container-fluid mt-3") {
                div(class = "row") {
                    div(class = "col-auto") {
                        let library_list = {List::default()}
                    }
                    {&right_column}
                }
            }
        }

        #[cfg(feature = "system9")]
        let mut lib = Self {
            main,
            library_list,
            right_column,
            theme_toggle_click,
            theme_checkbox,
            theme_enabled: false,
        };

        #[cfg(not(feature = "system9"))]
        let mut lib = Self {
            main,
            library_list,
            right_column,
        };

        lib.add_item("components::Alert", || {
            LibraryListPane::Alert(Default::default())
        });

        lib.add_item("components::Badge", || {
            LibraryListPane::Badge(Default::default())
        });

        lib.add_item("components::Button", || {
            LibraryListPane::Button(Default::default())
        });

        lib.add_item("components::ButtonGroup<T>", || {
            LibraryListPane::ButtonGroup(Default::default())
        });

        lib.add_item("components::Card", || {
            LibraryListPane::Card(Default::default())
        });

        lib.add_item("components::Dropdown", || {
            LibraryListPane::Dropdown(Default::default())
        });

        lib.add_item("components::Icon", || {
            LibraryListPane::Icon(Default::default())
        });

        lib.add_item("components::List<T>", || {
            LibraryListPane::List(Default::default())
        });

        lib.add_item("components::Modal", || {
            LibraryListPane::Modal(Default::default())
        });

        lib.add_item("components::Progress", || {
            LibraryListPane::Progress(Default::default())
        });

        lib.add_item("components::Panes<T> (Retain)", || {
            LibraryListPane::PaneRetain(Default::default())
        });

        lib.add_item("components::TabList<T>", || {
            LibraryListPane::TabList(Default::default())
        });

        lib.add_item("components::Toast", || {
            LibraryListPane::Toast(Default::default())
        });

        lib
    }
}

impl<V: View> Library<V> {
    pub fn add_item(&mut self, name: &str, f: impl FnMut() -> LibraryListPane<V> + 'static) {
        let item = LibraryListItem::new(name);
        self.library_list.push(item);
        self.right_column.add_pane(f);
    }

    /// Apply or remove the System 9 theme class on `<body>`.
    ///
    /// This is a static method because it operates on the global DOM body element.
    /// Only call when `V` is `Web`.
    #[cfg(feature = "system9")]
    fn apply_theme(enabled: bool) {
        let body = web_sys::window()
            .unwrap_throw()
            .document()
            .unwrap_throw()
            .body()
            .unwrap_throw();
        let current = body.class_name();
        if enabled {
            if !current.contains("system-9") {
                let new_class = if current.is_empty() {
                    "system-9".to_string()
                } else {
                    format!("{current} system-9")
                };
                body.set_class_name(&new_class);
            }
        } else {
            let new_class = current
                .split_whitespace()
                .filter(|c| *c != "system-9")
                .collect::<Vec<_>>()
                .join(" ");
            body.set_class_name(&new_class);
        }
    }

    /// Set the theme toggle state (checkbox + body class + internal flag).
    #[cfg(feature = "system9")]
    pub fn set_theme_enabled(&mut self, enabled: bool) {
        use js_sys::wasm_bindgen::JsCast;

        self.theme_enabled = enabled;
        if V::is_view::<mogwai::web::Web>() {
            Self::apply_theme(enabled);
            // Set the checkbox checked state via web_sys DOM query
            if let Some(input) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("system-9-toggle"))
                .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                input.set_checked(enabled);
            }
        }
    }

    pub fn deselect_all(&mut self) {
        for item in self.library_list.iter_mut() {
            item.set_is_active(false);
        }
    }

    pub fn select_item(&mut self, index: usize) {
        self.deselect_all();
        if let Some(item) = self.library_list.get_mut(index) {
            item.set_is_active(true);
            self.right_column.select(index);
        }
    }

    pub async fn step(&mut self) {
        #[cfg(feature = "system9")]
        {
            let pane_fut = async {
                self.right_column.get_pane_mut().step().await;
                Err(None)
            };
            let list_fut = async {
                let event = self.library_list.step().await;
                Err(Some(event))
            };
            let theme_fut = async {
                self.theme_toggle_click.next().await;
                Ok(())
            };
            match pane_fut.or(list_fut).or(theme_fut).await {
                Err(Some(ListEvent { index, event: _ })) => {
                    log::info!("loading index {index}");
                    self.select_item(index);
                    if V::is_view::<mogwai::web::Web>() {
                        crate::storage::set_item("selected-item", &index).unwrap_throw();
                    }
                }
                Ok(()) => {
                    self.theme_enabled = !self.theme_enabled;
                    log::info!("theme toggle: {}", self.theme_enabled);
                    if V::is_view::<mogwai::web::Web>() {
                        Self::apply_theme(self.theme_enabled);
                        crate::storage::set_item("system-9-theme", &self.theme_enabled)
                            .unwrap_throw();
                    }
                }
                _ => {}
            }
        }

        #[cfg(not(feature = "system9"))]
        {
            let pane_fut = async {
                self.right_column.get_pane_mut().step().await;
                None
            };
            let list_fut = async {
                let event = self.library_list.step().await;
                Some(event)
            };
            if let Some(ListEvent { index, event: _ }) = pane_fut.or(list_fut).await {
                log::info!("loading index {index}");
                self.select_item(index);
                if V::is_view::<mogwai::web::Web>() {
                    crate::storage::set_item("selected-item", &index).unwrap_throw();
                }
            }
        }
    }
}

/// Main loop of the component library web app.
pub async fn main() {
    use mogwai::web::prelude::*;

    log::info!("Starting up the iti component library...");

    let mut lib = Library::<Web>::default();
    let storage = mogwai::web::window()
        .local_storage()
        .unwrap_throw()
        .unwrap_throw();
    if let Some(item_index_str) = storage.get_item("selected-item").unwrap_throw() {
        let index: usize = item_index_str.parse().unwrap_throw();
        lib.select_item(index);
    }

    mogwai::web::body().append_child(&lib);

    // Restore theme state from localStorage (must happen after append_child
    // so the checkbox element is in the DOM for get_element_by_id).
    #[cfg(feature = "system9")]
    if let Ok(Some(theme_enabled)) = crate::storage::get_item::<bool>("system-9-theme") {
        lib.set_theme_enabled(theme_enabled);
    }

    wasm_bindgen_futures::spawn_local(async move {
        loop {
            lib.step().await;
        }
    });
}
