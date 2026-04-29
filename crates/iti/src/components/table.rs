//! MacOS System 9 Platinum-styled table component.
//!
//! Provides a table with sortable, resizable columns matching the classic
//! Finder folder list view aesthetic with raised/pressed bevel effects.

use std::future::Future;
use std::pin::Pin;

use futures_lite::FutureExt;
use mogwai::{
    future::{race_all, MogwaiFutureExt},
    prelude::*,
    web::{WebElement, WebEvent},
};
use wasm_bindgen::UnwrapThrowExt;

/// Sort direction for the active column or entry order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

type CreateCellFn<V, T> = Box<dyn Fn(&T, usize) -> <V as View>::Element>;
type CompareCellFn<T> = Box<dyn Fn(&T, &T) -> std::cmp::Ordering>;

/// Column definition with accessor function and sizing constraints.
pub struct Column<V: View, T> {
    header: String,
    create_cell_fn: CreateCellFn<V, T>,
    compare_cell_fn: CompareCellFn<T>,
    width: Option<u32>, // None = auto-size based on content
    min_width: u32,     // Minimum width for resizing (default 50px)
    resizable: bool,    // Whether column can be resized (default true)
}

/// Private reactive state for column headers.
struct ColumnHeaderState {
    is_active: bool,    // True when this column is the active sort column
    is_resizing: bool,  // True during resize drag operation
    width: Option<f64>, // Current width in pixels, sub-pixel precise (None = auto)
}

impl ColumnHeaderState {
    fn class(&self) -> String {
        let mut classes = vec!["table-header"];
        if self.is_active {
            classes.push("active");
        }
        if self.is_resizing {
            classes.push("resizing");
        }
        classes.join(" ")
    }

    fn style(&self) -> String {
        // Sub-pixel precision: format to 4 decimal places to avoid noisy long
        // floating-point representations while preserving more precision than the
        // browser can render. CSS accepts fractional pixel values directly.
        self.width
            .map(|w| format!("width: {:.4}px; max-width: {:.4}px", w, w))
            .unwrap_or_default()
    }
}

/// Column header cell with click and resize listeners.
struct ColumnHeader<V: View> {
    th: V::Element,
    #[allow(dead_code)]
    label: V::Element,
    #[allow(dead_code)]
    resize_handle: V::Element,
    on_click: V::EventListener,
    on_resize_mousedown: V::EventListener,
    state: Proxy<ColumnHeaderState>,
    col_index: usize,
}

/// Sort arrow column (dedicated rightmost header cell).
struct SortArrowHeader<V: View> {
    th: V::Element,
    #[allow(dead_code)]
    arrow_img: V::Element,
    on_click: V::EventListener,
    sort_order: Proxy<SortOrder>,
}

/// A single table row with rendered cells.
pub struct TableRow<V: View, T> {
    tr: V::Element,
    #[allow(dead_code)]
    cells: Vec<V::Element>,
    data: T,
}

/// Events emitted by the table.
#[derive(Debug)]
pub enum TableEvent<Ev = ()> {
    /// User clicked a column header to set it as the active sort column.
    HeaderClicked { col_index: usize },

    /// User clicked the sort arrow to toggle direction or restore entry order.
    ///
    /// Includes the new sort order.
    SortArrowClicked { sort_order: SortOrder },

    /// A user event.
    ///
    /// One of the cells is returning an event, called from `step_with`.
    User(Ev),
}

/// Internal state for column resize operation.
#[derive(PartialEq)]
struct ResizeState {
    col_index: usize, // Which column is being resized (left of the resize handle)
    start_x: i32,     // Initial mouse X position
    initial_widths: Vec<f64>, // Initial widths of ALL columns at resize start (sub-pixel precise)
    last_processed_mouse_x: i32, // Last mouse_x value we processed (for debouncing)
}

/// Internal events for table interaction (not exposed to users).
enum InternalEvent<Ev = ()> {
    HeaderClick(usize),
    SortArrowClick,
    ResizeStart { col_index: usize, mouse_x: i32 },
    User(Ev),
}

/// Events during a resize operation (internal only).
enum ResizeEvent {
    Move(i32), // Mouse X position
    End,       // Mouseup or escape
}

/// MacOS System 9 Platinum-styled table with sortable columns.
///
/// ## Features
///
/// - Column headers with raised/pressed bevel effects
/// - Dedicated sort arrow column (always visible)
/// - Resizable columns via drag handles
/// - Zebra-striped rows
/// - Single active sort column (or entry order when none active)
/// - Horizontal scroll overflow
///
/// ## Example
///
/// ```ignore
/// let table = TableBuilder::new()
///     .column("Name", |item, _| rsx! { span() { {V::Text::new(&item.name)} } })
///     .width(200)
///     .column("Size", |item, _| rsx! { span() { {V::Text::new(&item.size)} } })
///     .width(100)
///     .build();
/// ```
#[derive(ViewChild, ViewProperties)]
pub struct Table<V: View, T> {
    #[child]
    #[properties]
    container: V::Element,
    #[allow(dead_code)]
    table: V::Element,
    #[allow(dead_code)]
    thead: V::Element,
    #[allow(dead_code)]
    tbody: V::Element,
    headers: Vec<ColumnHeader<V>>,
    sort_header: SortArrowHeader<V>,
    rows: Vec<TableRow<V, T>>,
    columns: Vec<Column<V, T>>,
    active_sort_col: Proxy<Option<usize>>, // None = entry order
    active_sort_col_val: Option<usize>,    // Cached value for reading
    sort_order: SortOrder,                 // Cached sort order value
    resize_state: Proxy<Option<ResizeState>>, // None when not resizing
    /// True once column widths have been measured and locked into state to
    /// prevent the browser from rescaling them. Set lazily on the first
    /// `step()` call after the table is laid out.
    normalized: bool,
}

/// Builder for constructing tables with a fluent API.
pub struct TableBuilder<V: View, T> {
    columns: Vec<Column<V, T>>,
}

impl<V: View, T> TableBuilder<V, T> {
    pub fn new() -> Self {
        Self { columns: vec![] }
    }

    /// Add a column with header label and accessor function.
    ///
    /// ## Parameters
    /// * **header** - name of the column, displayed in the header.
    /// * **create_cell_fn** - cell creation function.
    ///   Takes a reference to row data `T` and the index of the column.
    /// * **compare_cell_fn** - sort comparison function.
    ///   Compares two rows for sort ordering.
    pub fn column(
        mut self,
        header: impl Into<String>,
        create_cell_fn: impl Fn(&T, usize) -> V::Element + 'static,
        compare_cell_fn: impl Fn(&T, &T) -> std::cmp::Ordering + 'static,
    ) -> Self {
        self.columns.push(Column {
            header: header.into(),
            create_cell_fn: Box::new(create_cell_fn),
            compare_cell_fn: Box::new(compare_cell_fn),
            width: None,
            min_width: 50,
            resizable: true,
        });
        self
    }

    /// Set width for the last added column.
    pub fn width(mut self, width: u32) -> Self {
        if let Some(col) = self.columns.last_mut() {
            col.width = Some(width);
        }
        self
    }

    /// Set minimum resize width for the last added column.
    pub fn min_width(mut self, min_width: u32) -> Self {
        if let Some(col) = self.columns.last_mut() {
            col.min_width = min_width;
        }
        self
    }

    /// Make the last added column non-resizable.
    pub fn fixed_width(mut self) -> Self {
        if let Some(col) = self.columns.last_mut() {
            col.resizable = false;
        }
        self
    }

    /// Build the table.
    pub fn build(self) -> Table<V, T> {
        Table::from_columns(self.columns)
    }
}

impl<V: View, T> Default for TableBuilder<V, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View, T> Table<V, T> {
    /// Create table from column definitions.
    fn from_columns(columns: Vec<Column<V, T>>) -> Self {
        // Create data column headers
        let mut headers = vec![];
        let num_columns = columns.len();
        for (col_index, col) in columns.iter().enumerate() {
            let mut state = Proxy::new(ColumnHeaderState {
                is_active: false,
                is_resizing: false,
                width: col.width.map(|w| w as f64),
            });

            // Resize handle - only add to columns that have a right neighbor (not last column)
            let is_last_column = col_index == num_columns - 1;
            rsx! {
                let resize_handle = div(
                    class = "table-resize-handle",
                    on:mousedown = on_resize_mousedown
                ) {}
            }

            rsx! {
                let label = span(
                    class = "table-header-label",
                    title = &col.header
                ) {
                    {V::Text::new(&col.header)}
                }
            }

            rsx! {
                let th = th(
                    class = state(s => s.class()),
                    style = state(s => s.style()),
                    on:click = on_click
                ) {
                    {&label}
                }
            }

            // Conditionally append resize handle after th creation
            if !is_last_column {
                th.append_child(&resize_handle);
            }

            headers.push(ColumnHeader {
                th,
                label,
                resize_handle,
                on_click,
                on_resize_mousedown,
                state,
                col_index,
            });
        }

        // Create sort arrow column
        let mut sort_order = Proxy::new(SortOrder::Ascending);

        rsx! {
            let arrow_img = img(
                class = "table-sort-arrow",
                src = sort_order(order => match order {
                    SortOrder::Ascending => "svg/table-sort-asc.svg",
                    SortOrder::Descending => "svg/table-sort-desc.svg",
                }),
                alt = "Sort"
            ) {}
        }

        rsx! {
            let th = th(
                class = "table-header table-sort-column",
                on:click = on_click
            ) {
                {&arrow_img}
            }
        }

        let sort_header = SortArrowHeader {
            th,
            arrow_img,
            on_click,
            sort_order,
        };

        // Create table structure
        rsx! {
            let tr_headers = tr() {}
        }

        // Append header cells
        for header in &headers {
            tr_headers.append_child(&header.th);
        }
        tr_headers.append_child(&sort_header.th);

        rsx! {
            let thead = thead() {
                {&tr_headers}
            }
        }

        rsx! {
            let tbody = tbody() {}
        }

        // Build <colgroup> with <col> elements. The sort column gets an inline
        // width: 20px declaration, which browsers honor strictly under
        // `table-layout: fixed` to prevent it from being scaled with the rest of
        // the table during proportional space distribution. Data columns use
        // bare <col> elements; their widths come from inline styles on the <th>
        // cells (which are reactive via Proxy bindings).
        rsx! {
            let colgroup_el = colgroup() {}
        }
        for _ in 0..num_columns {
            rsx! {
                let col_el = col() {}
            }
            colgroup_el.append_child(&col_el);
        }
        rsx! {
            let sort_col_el = col(style = "width: 20px") {}
        }
        colgroup_el.append_child(&sort_col_el);

        rsx! {
            let table = table(class = "table-platinum") {
                {&colgroup_el}
                {&thead}
                {&tbody}
            }
        }

        rsx! {
            let container = div(class = "table-platinum-container") {
                {&table}
            }
        }

        Self {
            container,
            table,
            thead,
            tbody,
            headers,
            sort_header,
            rows: vec![],
            columns,
            active_sort_col: Proxy::new(None),
            active_sort_col_val: None,
            sort_order: SortOrder::Ascending,
            resize_state: Proxy::new(None),
            normalized: false,
        }
    }

    /// Add a row to the table.
    pub fn push(&mut self, data: T)
    where
        T: 'static,
    {
        let mut cells = vec![];

        // Create cells using column accessors
        for (col_idx, column) in self.columns.iter().enumerate() {
            let cell_content = (column.create_cell_fn)(&data, col_idx);

            rsx! {
                let td = td(
                    class = "table-cell",
                    data:col_index = col_idx.to_string()
                ) {
                    {cell_content}
                }
            }

            cells.push(td);
        }

        rsx! {
            let tr = tr(class = "table-row") {}
        }

        // Append cells to row
        for cell in &cells {
            tr.append_child(cell);
        }

        let row = TableRow { tr, cells, data };

        // Append row to tbody
        self.tbody.append_child(&row.tr);
        self.rows.push(row);
    }

    /// Insert a row at the specified index.
    pub fn insert(&mut self, index: usize, data: T)
    where
        T: 'static,
    {
        let mut cells = vec![];

        for (col_idx, column) in self.columns.iter().enumerate() {
            let cell_content = (column.create_cell_fn)(&data, col_idx);

            rsx! {
                let td = td(
                    class = "table-cell",
                    data:col_index = col_idx.to_string()
                ) {
                    {cell_content}
                }
            }

            cells.push(td);
        }

        rsx! {
            let tr = tr(class = "table-row") {}
        }

        // Append cells to row
        for cell in &cells {
            tr.append_child(cell);
        }

        let row = TableRow { tr, cells, data };

        // Insert row at the specified index in tbody
        // Note: mogwai doesn't expose insert_before, so we just append for now
        // TODO: Implement proper insertion order if needed
        self.tbody.append_child(&row.tr);
        self.rows.insert(index, row);
    }

    /// Remove a row by index.
    pub fn remove(&mut self, index: usize) -> T {
        let row = self.rows.remove(index);
        row.data
    }

    /// Get row data reference.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.rows.get(index).map(|r| &r.data)
    }

    /// Get mutable row data reference.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.rows.get_mut(index).map(|r| &mut r.data)
    }

    /// Get the number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Iterate over row data.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.rows.iter().map(|r| &r.data)
    }

    /// Set which column is actively sorted (None = entry order).
    pub fn set_active_sort_column(&mut self, col_index: Option<usize>) {
        self.active_sort_col.set(col_index);
        self.active_sort_col_val = col_index;

        // Update header active states
        for (idx, header) in self.headers.iter_mut().enumerate() {
            header
                .state
                .modify(|s| s.is_active = Some(idx) == col_index);
        }

        // Update cell highlighting for active column
        for row in &self.rows {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                if Some(cell_idx) == col_index {
                    cell.dyn_el(|el: &web_sys::Element| {
                        el.class_list().add_1("active-column").ok();
                    });
                } else {
                    cell.dyn_el(|el: &web_sys::Element| {
                        el.class_list().remove_1("active-column").ok();
                    });
                }
            }
        }
    }

    /// Get the currently active sort column (None = entry order).
    pub fn get_active_sort_column(&self) -> Option<usize> {
        self.active_sort_col_val
    }

    /// Get the current sort order (always has a value).
    pub fn get_sort_order(&self) -> SortOrder {
        self.sort_order
    }

    /// Set the sort order (updates arrow icon).
    pub fn set_sort_order(&mut self, order: SortOrder) {
        self.sort_header.sort_order.set(order);
        self.sort_order = order;
    }

    /// Toggle sort order and return new value.
    pub fn toggle_sort_order(&mut self) -> SortOrder {
        let new_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
        self.sort_header.sort_order.set(new_order);
        self.sort_order = new_order;
        new_order
    }

    /// Sort rows by the given column index and order.
    ///
    /// Does nothing if `col_index` is out of bounds.
    pub fn sort_by_column(&mut self, col_index: usize, sort_order: SortOrder) {
        if let Some(col) = self.columns.get(col_index) {
            let mut rows = self.rows.iter().collect::<Vec<_>>();
            rows.sort_by(|a, b| {
                let cmp = (col.compare_cell_fn)(&a.data, &b.data);
                match sort_order {
                    SortOrder::Ascending => cmp,
                    SortOrder::Descending => cmp.reverse(),
                }
            });

            // Re-append all rows in the new sorted order to update the DOM
            // In mogwai/web, re-appending an element moves it to the end
            for row in rows {
                self.tbody.append_child(&row.tr);
            }
        }
    }

    /// Restore original insertion order.
    pub fn sort_by_entry_order(&self, sort_order: SortOrder) {
        let mut rows = self.rows.iter().collect::<Vec<_>>();
        if matches!(sort_order, SortOrder::Descending) {
            rows.reverse();
        }
        // Re-append all rows in entry order to update the DOM
        for row in rows {
            self.tbody.append_child(&row.tr);
        }
    }

    /// Wait for any user action (header click, sort click, or resize start).
    async fn wait_for_user_action<Ev>(
        &mut self,
        cell_step: &mut impl FnMut(&mut T) -> Pin<Box<dyn Future<Output = Ev> + '_>>,
    ) -> InternalEvent<Ev> {
        let Self {
            headers,
            sort_header,
            rows,
            ..
        } = self;
        // Data column header clicks
        let _header_clicks = headers.iter().map(|h| {
            async {
                let col_idx = h.col_index;
                let _ev = h.on_click.next().await;
                InternalEvent::HeaderClick(col_idx)
            }
            .boxed_local()
        });

        // Resize handle mousedown events
        let _header_mousedowns = headers.iter().map(|h| {
            async {
                let col_idx = h.col_index;
                let event = h.on_resize_mousedown.next().await;
                // Extract mouse X position from the event
                let mouse_x = event
                    .dyn_ev(|e: &web_sys::MouseEvent| e.client_x())
                    .unwrap_or(0);
                InternalEvent::ResizeStart {
                    col_index: col_idx,
                    mouse_x,
                }
            }
            .boxed_local()
        });

        // Sort arrow column click
        let sort_fut = async {
            sort_header.on_click.next().await;
            InternalEvent::SortArrowClick
        }
        .boxed_local();

        let user = rows.iter_mut().map(|row| {
            let t = &mut row.data;
            cell_step(t).map(InternalEvent::User).boxed_local()
        });

        // Race all futures
        let mut all_futures = vec![];
        all_futures.extend(_header_clicks);
        all_futures.extend(_header_mousedowns);
        all_futures.push(sort_fut);
        all_futures.extend(user);
        race_all(all_futures).await
    }

    /// Wait for resize drag events (mousemove or mouseup on document).
    async fn wait_for_resize_event(&self) -> ResizeEvent {
        // Get document
        let document = web_sys::window().unwrap_throw().document().unwrap_throw();

        // Create mousemove future
        let mousemove_fut = async {
            let event = document.listen("mousemove").next().await;
            let mouse_x = event
                .dyn_ev(|e: &web_sys::MouseEvent| e.client_x())
                .unwrap_or(0);
            ResizeEvent::Move(mouse_x)
        };

        // Create mouseup future
        let mouseup_fut = async {
            let ev = document.listen("mouseup").next().await;
            ev.stop_propagation();
            ResizeEvent::End
        };

        // Race them
        mousemove_fut.or(mouseup_fut).await
    }

    /// Wait for the next table event, or a user event from a table cell.
    ///
    /// This method handles both normal user interactions (header clicks, sort clicks)
    /// and column resizing internally. Resize operations are handled in a loop and
    /// don't return events to the caller.
    ///
    /// ## Note
    /// By the time the event is returned, the table has already reacted to the event.
    /// For example, if `HeaderClick` is returned, the table has already re-sorted accordingly.
    pub async fn step_with<Ev>(
        &mut self,
        mut cell_step: impl FnMut(&mut T) -> Pin<Box<dyn Future<Output = Ev> + '_>>,
    ) -> TableEvent<Ev> {
        // Lazy mount-time normalization. On the first call after the table is
        // laid out, measure rendered widths and write them back to state. This
        // ensures the layout is self-consistent (no browser rescaling) before
        // the user ever interacts, eliminating the visible glitch on first
        // mousedown. Skipped if the table isn't laid out yet (e.g., it's in a
        // hidden tab); will retry on the next call.
        if !self.normalized {
            let table_width = self
                .table
                .dyn_el(|el: &web_sys::Element| el.get_bounding_client_rect().width())
                .unwrap_or(0.0);
            if table_width > 0.0 {
                self.normalize_widths();
                self.normalized = true;
            }
        }

        loop {
            // Wait for a user action
            let event = self.wait_for_user_action(&mut cell_step).await;

            match event {
                InternalEvent::HeaderClick(col_index) => {
                    let current_active = self.get_active_sort_column();
                    let current_order = self.get_sort_order();

                    if current_active == Some(col_index) {
                        // This tab was previously active, so deselect it and return
                        // to entry order.
                        self.set_active_sort_column(None);
                        self.sort_by_entry_order(current_order);
                    } else {
                        self.set_active_sort_column(Some(col_index));
                        self.sort_by_column(col_index, current_order);
                    }

                    return TableEvent::HeaderClicked { col_index };
                }
                InternalEvent::SortArrowClick => {
                    let new_order = self.toggle_sort_order();
                    if let Some(col_index) = self.get_active_sort_column() {
                        self.sort_by_column(col_index, new_order);
                    } else {
                        self.sort_by_entry_order(new_order);
                    }
                    return TableEvent::SortArrowClicked {
                        sort_order: new_order,
                    };
                }
                InternalEvent::ResizeStart { col_index, mouse_x } => {
                    // Enter resize mode
                    self.handle_resize_start(col_index, mouse_x);

                    // Loop until resize ends
                    loop {
                        let resize_event = self.wait_for_resize_event().await;

                        match resize_event {
                            ResizeEvent::Move(mouse_x) => {
                                self.handle_resize_move(mouse_x);
                                // Continue resize loop
                            }
                            ResizeEvent::End => {
                                self.handle_resize_end().await;
                                // Break out of resize loop, back to waiting for user actions
                                break;
                            }
                        }
                    }
                    // Loop continues - wait for next user action
                }
                InternalEvent::User(ev) => return TableEvent::User(ev),
            }
        }
    }

    /// Wait for the next table event.
    ///
    /// This method handles both normal user interactions (header clicks, sort clicks)
    /// and column resizing internally. Resize operations are handled in a loop and
    /// don't return events to the caller.
    ///
    /// ## Note
    /// By the time the event is returned, the table has already reacted to the event.
    /// For example, if `HeaderClick` is returned, the table has already re-sorted accordingly.
    pub async fn step(&mut self) -> TableEvent {
        self.step_with(|_| std::future::pending().boxed()).await
    }

    /// Measure rendered widths of all data column headers and write them back
    /// into state.
    ///
    /// With `table-layout: fixed; width: 100%`, the browser scales configured
    /// column widths to fill the table. This means `state.width` (what we set)
    /// and the rendered width diverge. As soon as we change one width, the
    /// browser re-scales every column, amplifying our change by the scale factor.
    ///
    /// To break this cycle, we measure each column's rendered width using
    /// `getBoundingClientRect().width()` (sub-pixel precise) and write that exact
    /// value back into state. The configured widths now equal the rendered widths
    /// and sum to the full table width, so the browser produces an identical
    /// layout with no shift. Sub-pixel precision avoids the integer-rounding snap
    /// that would occur when writing only `clientWidth`.
    ///
    /// Returns the captured widths so callers can use them as resize baselines.
    fn normalize_widths(&mut self) -> Vec<f64> {
        let widths: Vec<f64> = self
            .headers
            .iter()
            .map(|h| {
                h.th.dyn_el(|el: &web_sys::Element| el.get_bounding_client_rect().width())
                    .unwrap_or(100.0)
            })
            .collect();

        for (idx, header) in self.headers.iter_mut().enumerate() {
            let w = widths[idx];
            header.state.modify(|s| s.width = Some(w));
        }
        widths
    }

    /// Handle the start of a column resize operation.
    fn handle_resize_start(&mut self, col_index: usize, mouse_x: i32) {
        // Re-normalize on every resize start so the system self-heals if the
        // container has been resized between operations.
        let initial_widths = self.normalize_widths();

        // Store resize state
        let new_state = Some(ResizeState {
            col_index,
            start_x: mouse_x,
            initial_widths,
            last_processed_mouse_x: mouse_x, // Initialize to start position
        });
        self.resize_state.modify(|s| *s = new_state);

        // Set resizing flag on the header
        self.headers[col_index]
            .state
            .modify(|s| s.is_resizing = true);

        // Add global cursor class to body
        let document = web_sys::window().unwrap_throw().document().unwrap_throw();
        if let Some(body) = document.body() {
            body.class_list().add_1("table-resizing").ok();
        }
    }

    /// Handle mouse movement during column resize.
    ///
    /// Implements zero-sum resizing where the resize handle (on the right edge of
    /// col_index) stays under the mouse cursor. All width calculations are performed
    /// from the initial_widths baseline captured at resize start to prevent cumulative
    /// errors and ensure 1:1 mouse tracking. Widths use sub-pixel precision to avoid
    /// integer-rounding artifacts on first mousedown.
    fn handle_resize_move(&mut self, mouse_x: i32) {
        const MIN_WIDTH_PX: f64 = 16.0; // ~1em minimum

        // Read resize state directly via Deref
        let resize_info = (*self.resize_state).as_ref().map(|s| {
            (
                s.col_index,
                s.start_x,
                s.initial_widths.clone(),
                s.last_processed_mouse_x,
            )
        });

        if let Some((col_index, start_x, initial_widths, last_processed_mouse_x)) = resize_info {
            // DEBOUNCE: Skip if we've already processed this mouse position
            if mouse_x == last_processed_mouse_x {
                return;
            }

            // Calculate how far the mouse has moved from the start (integer mouse coords)
            let delta_int = mouse_x - start_x;
            if delta_int == 0 {
                return; // No movement
            }
            let delta = delta_int as f64;

            // Get initial width of the column being resized
            let start_width = initial_widths.get(col_index).copied().unwrap_or(100.0);

            if delta > 0.0 {
                // DRAG RIGHT: col_index grows, take from right neighbors (col_index+1, +2, ...)
                let target_width = (start_width + delta).max(MIN_WIDTH_PX);
                let actual_change = target_width - start_width;

                let growth_needed = actual_change;
                let mut space_collected: f64 = 0.0;
                let mut adjustments: Vec<(usize, f64)> = vec![]; // (idx, new_width)

                // Collect space from right neighbors
                let mut donor_idx = col_index + 1;
                while space_collected < growth_needed && donor_idx < initial_widths.len() {
                    let donor_initial = initial_widths[donor_idx];
                    let can_give = (donor_initial - MIN_WIDTH_PX).max(0.0);
                    let take = can_give.min(growth_needed - space_collected);

                    if take > 0.0 {
                        let new_donor_width = donor_initial - take;
                        adjustments.push((donor_idx, new_donor_width));
                        space_collected += take;
                    }

                    donor_idx += 1;
                }

                // If we couldn't collect any space, can't resize
                if space_collected <= 0.0 {
                    return;
                }

                // Apply changes: grow left column by space_collected
                let final_left_width = start_width + space_collected;
                self.headers[col_index]
                    .state
                    .modify(|s| s.width = Some(final_left_width));

                // Shrink donor columns
                for (donor_idx, new_width) in adjustments {
                    self.headers[donor_idx]
                        .state
                        .modify(|s| s.width = Some(new_width));
                }
            } else {
                // DRAG LEFT: handle moves left, taking space from col_index and
                // cascading to col_index-1, col_index-2, ... if col_index hits
                // MIN_WIDTH. The full requested shrink (uncapped delta) is
                // distributed across col_index and its left neighbors, each
                // donating up to (donor_initial - MIN_WIDTH_PX). Total collected
                // space goes to col_index+1 (the right neighbor).
                let requested_shrink = delta.abs();

                // Check if right neighbor exists to receive donated space
                if col_index + 1 >= initial_widths.len() {
                    return;
                }

                let mut space_collected: f64 = 0.0;
                let mut adjustments: Vec<(usize, f64)> = vec![]; // (idx, new_width)

                // Cascade through col_index, col_index-1, col_index-2, ...
                // col_index is the first donor; if it hits MIN_WIDTH, the
                // remaining demand cascades to its left neighbors.
                let mut donor_idx = col_index as i32;
                while space_collected < requested_shrink && donor_idx >= 0 {
                    let donor_usize = donor_idx as usize;
                    let donor_initial = initial_widths[donor_usize];
                    let can_give = (donor_initial - MIN_WIDTH_PX).max(0.0);
                    let take = can_give.min(requested_shrink - space_collected);

                    if take > 0.0 {
                        let new_width = donor_initial - take;
                        adjustments.push((donor_usize, new_width));
                        space_collected += take;
                    }

                    donor_idx -= 1;
                }

                // If we couldn't collect any space (all donors at MIN_WIDTH), can't resize
                if space_collected <= 0.0 {
                    return;
                }

                // Apply changes: shrink donor columns
                for (donor_idx, new_width) in adjustments {
                    self.headers[donor_idx]
                        .state
                        .modify(|s| s.width = Some(new_width));
                }

                // Grow right neighbor by the total space collected
                let right_neighbor_idx = col_index + 1;
                let right_initial = initial_widths[right_neighbor_idx];
                let new_right_width = right_initial + space_collected;
                self.headers[right_neighbor_idx]
                    .state
                    .modify(|s| s.width = Some(new_right_width));
            }

            // Update last processed mouse position to prevent duplicate processing
            self.resize_state.modify(|s| {
                if let Some(state) = s.as_mut() {
                    state.last_processed_mouse_x = mouse_x;
                }
            });
        }
    }

    /// Handle the end of a column resize operation.
    async fn handle_resize_end(&mut self) {
        // Read the col_index before clearing state
        let col_index = (*self.resize_state).as_ref().map(|s| s.col_index);

        if let Some(col_idx) = col_index {
            // Clear resizing flag
            self.headers[col_idx]
                .state
                .modify(|s| s.is_resizing = false);
        }

        // Clear resize state
        self.resize_state.modify(|s| *s = None);

        // Remove global cursor class from body
        let document = web_sys::window().unwrap_throw().document().unwrap_throw();
        if let Some(body) = document.body() {
            body.class_list().remove_1("table-resizing").ok();
        }

        // We need to debounce _one_ header click that gets queued when a mouseup occurs inside the
        // same header that started the resize process, so we don't accidentally select it,
        // toggling the sort order.
        let mut clicks_or_timeout = self
            .headers
            .iter()
            .map(|column_header| column_header.on_click.next().map(|_| ()).boxed_local())
            .collect::<Vec<_>>();
        clicks_or_timeout.push(
            async {
                mogwai::time::wait_millis(10).await;
            }
            .boxed_local(),
        );
        race_all(clicks_or_timeout).await;
    }
}

#[cfg(feature = "library")]
pub mod library {
    use crate::components::alert::Alert;

    use super::*;

    #[derive(Clone)]
    pub struct FileEntry {
        pub name: String,
        pub date_modified: String,
        pub size: String,
        pub kind: String,
    }

    #[derive(ViewChild)]
    pub struct TableLibraryItem<V: View> {
        #[child]
        container: V::Element,
        table: Table<V, FileEntry>,
        log_text: Proxy<String>,
    }

    impl<V: View> Default for TableLibraryItem<V> {
        fn default() -> Self {
            let mut table = TableBuilder::new()
                .column(
                    "Name",
                    |file: &FileEntry, _| {
                        rsx! {
                            let span_el = span() { {V::Text::new(&file.name)} }
                        }
                        span_el
                    },
                    |a, b| a.name.cmp(&b.name),
                )
                .width(250)
                .column(
                    "Date Modified",
                    |file: &FileEntry, _| {
                        rsx! {
                            let span_el = span() { {V::Text::new(&file.date_modified)} }
                        }
                        span_el
                    },
                    |a, b| a.date_modified.cmp(&b.date_modified),
                )
                .width(200)
                .column(
                    "Size",
                    |file: &FileEntry, _| {
                        rsx! {
                            let span_el = span() { {V::Text::new(&file.size)} }
                        }
                        span_el
                    },
                    |a, b| a.size.cmp(&b.size),
                )
                .width(80)
                .column(
                    "Kind",
                    |file: &FileEntry, _| {
                        rsx! {
                            let span_el = span() { {V::Text::new(&file.kind)} }
                        }
                        span_el
                    },
                    |a, b| a.kind.cmp(&b.kind),
                )
                .width(180)
                .build();

            // Sample data from reference image
            table.push(FileEntry {
                name: "Apple LaserWriter Software".into(),
                date_modified: "Sat, Dec 17, 2020, 8:13 PM".into(),
                size: "22 K".into(),
                kind: "folder".into(),
            });
            table.push(FileEntry {
                name: "AppleScript".into(),
                date_modified: "Sat, Dec 19, 2020, 4:23 PM".into(),
                size: "212 K".into(),
                kind: "folder".into(),
            });
            table.push(FileEntry {
                name: "ColorSync Extras".into(),
                date_modified: "Sat, Dec 19, 2020, 4:45 PM".into(),
                size: "458 K".into(),
                kind: "folder".into(),
            });
            table.push(FileEntry {
                name: "FireWire".into(),
                date_modified: "Sat, Dec 09, 2020, 3:23 PM".into(),
                size: "1.2 M".into(),
                kind: "folder".into(),
            });
            table.push(FileEntry {
                name: "Font Extras".into(),
                date_modified: "Sat, Sep 11, 2020, 12:45 PM".into(),
                size: "2.5 M".into(),
                kind: "folder".into(),
            });
            table.push(FileEntry {
                name: "Calculator".into(),
                date_modified: "Sat, Dec 19, 2020, 4:45 PM".into(),
                size: "68 K".into(),
                kind: "application program".into(),
            });
            table.push(FileEntry {
                name: "Sherlock 2.0".into(),
                date_modified: "Sat, Dec 19, 2020, 4:55 PM".into(),
                size: "24 K".into(),
                kind: "application program".into(),
            });

            let mut log_text = Proxy::new(
                "Click column headers to sort. Click arrow to toggle direction or restore entry order."
                    .to_string(),
            );

            rsx! {
                let container = div() {
                    {&table}
                    div(class = "mt-3 p-2") {
                        let alert = {Alert::new("Awaiting user events...", crate::components::Flavor::Info)}
                    }
                }
            }
            log_text.on_update(move |text| {
                alert.set_text(text);
            });

            Self {
                container,
                table,
                log_text,
            }
        }
    }

    impl<V: View> TableLibraryItem<V> {
        pub async fn step(&mut self) {
            let event = self.table.step().await;

            match event {
                TableEvent::HeaderClicked { col_index } => {
                    let col_name = match col_index {
                        0 => "Name",
                        1 => "Date Modified",
                        2 => "Size",
                        3 => "Kind",
                        _ => "Unknown",
                    };

                    let current_active = self.table.get_active_sort_column();
                    let current_order = self.table.get_sort_order();

                    // Check if clicking the active column (deactivate it)
                    if current_active.is_none() {
                        self.log_text.set("Returned to entry order".to_string());
                    } else {
                        self.log_text
                            .set(format!("Sorting by: {} ({:?})", col_name, current_order));
                    }
                }

                TableEvent::SortArrowClicked { sort_order } => {
                    if self.table.get_active_sort_column().is_some() {
                        // Re-sort by active column in new direction
                        self.log_text
                            .set(format!("Toggled sort direction: {:?}", sort_order));
                    } else {
                        // No active column - reverse entry order
                        self.log_text
                            .set(format!("Reversed entry order: {:?}", sort_order));
                    }
                }

                TableEvent::User(_) => {}
            }
        }
    }
}
