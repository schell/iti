//! Component library gallery — renders the Platinum Kit as the root view.
use mogwai::web::body;

use crate::components::platinum_kit::OverhaulLibraryItem;

/// Main loop of the component library web app.
///
/// Renders the [`OverhaulLibraryItem`] (the Platinum Kit) directly as the
/// root view and drives its event loop.
pub async fn main() {
    use mogwai::web::prelude::*;

    log::info!("Starting up the iti component library...");

    let mut kit = OverhaulLibraryItem::<Web>::default();

    body().set_style("background-color", crate::color::LAVENDER);
    body().append_child(&kit);

    wasm_bindgen_futures::spawn_local(async move {
        loop {
            kit.step_mut().await;
        }
    });
}
