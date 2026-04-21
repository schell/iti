//! Sandboxed component gallery for browsing and testing components in isolation.
use futures_lite::FutureExt;
use js_sys::wasm_bindgen::UnwrapThrowExt;
use mogwai::{prelude::*, web::body};

use crate::components::{
    button::library::ButtonLibraryItem,
    button_group::library::ButtonGroupLibraryItem,
    checkbox::library::CheckboxLibraryItem,
    dropdown::library::DropdownLibraryItem,
    list::{library::ListLibraryItem, List, ListEvent},
    modal::library::ModalLibraryItem,
    pane::{library::PaneRetainLibraryItem, RestartPanes},
    platinum_kit::OverhaulLibraryItem,
    progress::library::ProgressLibraryItem,
    radio::library::RadioLibraryItem,
    select::library::SelectLibraryItem,
    slider::library::SliderLibraryItem,
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
    Button(ButtonLibraryItem<V>),
    ButtonGroup(ButtonGroupLibraryItem<V>),
    Checkbox(CheckboxLibraryItem<V>),
    Dropdown(DropdownLibraryItem<V>),
    List(ListLibraryItem<V>),
    Modal(ModalLibraryItem<V>),
    Overhaul(OverhaulLibraryItem<V>),
    PaneRetain(Box<PaneRetainLibraryItem<V>>),
    Progress(ProgressLibraryItem<V>),
    Radio(RadioLibraryItem<V>),
    Select(SelectLibraryItem<V>),
    Slider(SliderLibraryItem<V>),
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
            LibraryListPane::Button(item) => item.as_boxed_append_arg(),
            LibraryListPane::ButtonGroup(item) => item.as_boxed_append_arg(),
            LibraryListPane::Checkbox(item) => item.as_boxed_append_arg(),
            LibraryListPane::Dropdown(item) => item.as_boxed_append_arg(),
            LibraryListPane::List(item) => item.as_boxed_append_arg(),
            LibraryListPane::Modal(item) => item.as_boxed_append_arg(),
            LibraryListPane::Overhaul(item) => item.as_boxed_append_arg(),
            LibraryListPane::PaneRetain(item) => item.as_boxed_append_arg(),
            LibraryListPane::Progress(item) => item.as_boxed_append_arg(),
            LibraryListPane::Radio(item) => item.as_boxed_append_arg(),
            LibraryListPane::Select(item) => item.as_boxed_append_arg(),
            LibraryListPane::Slider(item) => item.as_boxed_append_arg(),
            LibraryListPane::TabList(item) => item.as_boxed_append_arg(),
            LibraryListPane::Toast(item) => item.as_boxed_append_arg(),
        }
    }
}

impl<V: View> LibraryListPane<V> {
    pub async fn step(&mut self) {
        let body = body();
        body.set_style("background-color", crate::color::LAVENDER);
        match self {
            LibraryListPane::Button(item) => item.step().await,
            LibraryListPane::ButtonGroup(item) => item.step().await,
            LibraryListPane::Checkbox(item) => item.step().await,
            LibraryListPane::Dropdown(item) => item.step().await,
            LibraryListPane::List(item) => item.step().await,
            LibraryListPane::Modal(item) => item.step().await,
            LibraryListPane::PaneRetain(item) => item.step().await,
            LibraryListPane::Progress(item) => item.step().await,
            LibraryListPane::Radio(item) => item.step().await,
            LibraryListPane::Select(item) => item.step().await,
            LibraryListPane::Slider(item) => item.step().await,
            LibraryListPane::TabList(item) => item.step().await,
            LibraryListPane::Toast(item) => item.step().await,
            LibraryListPane::Overhaul(item) => {
                item.step().await;
            }
            LibraryListPane::Default(_) => std::future::pending().await,
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
    right_column_pane_ids: Vec<crate::id::Id<LibraryListPane<V>>>,
}

impl<V: View> Default for Library<V> {
    fn default() -> Self {
        rsx! {
            let right_column_wrapper = div(class = "col") {}
        }

        let right_column = RestartPanes::new(right_column_wrapper, LibraryListPane::default());

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

        let mut lib = Self {
            main,
            library_list,
            right_column,
            right_column_pane_ids: vec![],
        };

        lib.add_item("components::Button", || {
            LibraryListPane::Button(Default::default())
        });

        lib.add_item("components::ButtonGroup<T>", || {
            LibraryListPane::ButtonGroup(Default::default())
        });

        lib.add_item("components::Checkbox", || {
            LibraryListPane::Checkbox(Default::default())
        });

        lib.add_item("components::Dropdown", || {
            LibraryListPane::Dropdown(Default::default())
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

        lib.add_item("components::RadioGroup", || {
            LibraryListPane::Radio(Default::default())
        });

        lib.add_item("components::Select", || {
            LibraryListPane::Select(Default::default())
        });

        lib.add_item("components::Slider", || {
            LibraryListPane::Slider(Default::default())
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

        lib.add_item("Platinum Kit", || {
            LibraryListPane::Overhaul(Default::default())
        });

        lib
    }
}

impl<V: View> Library<V> {
    pub fn add_item(&mut self, name: &str, f: impl FnMut() -> LibraryListPane<V> + 'static) {
        let item = LibraryListItem::new(name);
        self.library_list.push(item);
        let id = self.right_column.add_pane(f);
        self.right_column_pane_ids.push(id);
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
            if let Some(id) = self.right_column_pane_ids.get(index) {
                let _ = self.right_column.select(id);
            }
        }
    }

    pub async fn step(&mut self) {
        let pane_fut = async {
            self.right_column.current_pane_mut().step().await;
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

    wasm_bindgen_futures::spawn_local(async move {
        loop {
            lib.step().await;
        }
    });
}
