//! MacOS System 9 Platinum-styled table component.
//!
//! Provides a table with sortable, resizable columns matching the classic
//! Finder folder list view aesthetic with raised/pressed bevel effects.
//!
//! # Overview
//!
//! The table is constructed with [`TableBuilder`] using a fluent API: each
//! [`TableBuilder::column`] call registers a column (header label + cell
//! accessor + sort comparator), and any subsequent sizing methods
//! ([`TableBuilder::width`], [`TableBuilder::width_percent`],
//! [`TableBuilder::width_auto`], [`TableBuilder::min_width`],
//! [`TableBuilder::fixed_width`]) apply to that last-added column. Call
//! [`TableBuilder::build`] to finish and get a [`Table`].
//!
//! Rows are added after construction via [`Table::push`]. The table reacts to
//! user input (column header clicks, sort-arrow clicks, column resize drags)
//! through the `step()` event loop driven by the caller; see [`Table`] for
//! driving semantics and [`TableEvent`] for the emitted events.
//!
//! A column's cell content can be any `Cell: ViewChild<V>`. It's inferred from
//! the `column()` accessor closure's return type — returning a `V::Element`
//! (e.g. from `rsx! { span() }`) is the common case; return any other
//! [`ViewChild<V>`] (such as a [`Button`](crate::components::button::Button))
//! to embed a full iti component as a cell. See [`Table`] for the constraint
//! that all columns share one `Cell` type.
//!
//! # Example
//!
//! ```ignore
//! use mogwai::prelude::*;
//! use iti::components::table::{TableBuilder, TableEvent, SortOrder};
//!
//! // Each row carries a `T` of your choosing; the table never constrains it.
//! struct FileEntry { name: String, size: String }
//!
//! let mut table = TableBuilder::new()
//!     .column(
//!         "Name",
//!         |file: &FileEntry, _| {
//!             rsx! { let s = span() { {V::Text::new(&file.name)} } }
//!             s
//!         },
//!         |a, b| a.name.cmp(&b.name),
//!     )
//!     .width_percent(60.0)
//!     .column(
//!         "Size",
//!         |file: &FileEntry, _| {
//!             rsx! { let s = span() { {V::Text::new(&file.size)} } }
//!             s
//!         },
//!         |a, b| a.size.cmp(&b.size),
//!     )
//!     .width(120)
//!     .use_scrollbar(true)
//!     .build();
//!
//! table.push(FileEntry { name: "readme.txt".into(), size: "1 KB".into() });
//! table.push(FileEntry { name: "notes.md".into(), size: "2 KB".into() });
//!
//! loop {
//!     match table.step_mut().await {
//!         TableEvent::HeaderClicked { col_index } => { /* sorted already */ }
//!         TableEvent::SortArrowClicked { sort_order } => { /* arrow toggled */ }
//!         TableEvent::User(()) => unreachable!("no per-cell futures"),
//!     }
//! }
//! ```

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

/// Column width sizing mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnSize {
    /// Fixed width in pixels (sub-pixel precision).
    Pixels(f64),
    /// Percentage of table width (0.0 - 100.0).
    Percent(f64),
    /// Auto-size: equal-share of remaining space.
    Auto,
}

type CreateCellFn<T, Cell> = Box<dyn Fn(&T, usize) -> Cell>;
type CompareCellFn<T> = Box<dyn Fn(&T, &T) -> std::cmp::Ordering>;

/// Column definition with accessor function and sizing constraints.
///
/// # Type parameters
///
/// - `T` — the row data model carried by the owning [`Table`]. It is
///   unconstrained: the column reads from it via `create_cell_fn` /
///   `compare_cell_fn` rather than via any trait bound on `T`, so `T` needs no
///   `Ord`/`Clone`/etc. — sorting is delegated to the comparator you supply.
/// - `Cell` — the cell content type produced by `create_cell_fn`. Inferred
///   from the return type of the `create_cell_fn` closure (typically `V::Element`
///   for a plain DOM element returned by `rsx! { span() }`); may be any type
///   implementing [`ViewChild<V>`]. All columns of a [`Table`] / [`TableBuilder`]
///   share the same `Cell` type.
pub struct Column<T, Cell> {
    header: String,
    create_cell_fn: CreateCellFn<T, Cell>,
    compare_cell_fn: CompareCellFn<T>,
    declared_size: ColumnSize, // User-declared width mode
    min_width: u32,            // Minimum width for resizing (default 50px)
    resizable: bool,           // Whether column can be resized (default true)
}

/// Private reactive state for column headers.
struct ColumnHeaderState {
    is_active: bool,   // True when this column is the active sort column
    is_resizing: bool, // True during resize drag operation
    size: ColumnSize,  // Current width mode (pixels/percent/auto)
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
        match self.size {
            ColumnSize::Pixels(w) => format!("width: {:.4}px; max-width: {:.4}px", w, w),
            ColumnSize::Percent(p) => format!("width: {:.4}%", p),
            ColumnSize::Auto => String::new(),
        }
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
///
/// # Type parameters
///
/// - `V: View` — the mogwai view abstraction.
/// - `T` — the row data carried by this row. Owned by the row and returned by
///   value from [`Table::remove`]; accessed by reference via [`Table::get`] /
///   [`Table::get_mut`].
pub struct TableRow<V: View, T> {
    tr: V::Element,
    #[allow(dead_code)]
    cells: Vec<V::Element>,
    data: T,
}

/// Events emitted by the table.
///
/// # Type parameters
///
/// - `Ev` — the user-defined event type yielded by the per-cell future passed
///   to [`StepWithMut::step_with_mut`]. Defaults to `()`: when the table is
///   driven via [`StepMut::step_mut`] (no per-cell futures) `Ev` stays `()` and
///   [`TableEvent::User`] is effectively unreachable, which is why it carries a
///   default. Use a non-`()` `Ev` only when at least one column's cell content
///   produces its own events that need to bubble up through the table loop.
///
/// # React-before-return contract
///
/// By the time any variant is returned, the table has **already** updated its
/// internal state and DOM to match the user action. For example, when
/// [`TableEvent::HeaderClicked`] is returned the column has already been
/// activated and the rows re-sorted; when [`TableEvent::SortArrowClicked`] is
/// returned the arrow icon has already toggled and rows re-sorted. The returned
/// event is therefore an observation, not a command to react to.
#[derive(Debug)]
pub enum TableEvent<Ev = ()> {
    /// User clicked a column header to set it as the active sort column.
    ///
    /// The table has already activated the column (or returned to entry order
    /// if the same column was clicked again) and re-sorted the rows. `col_index`
    /// is the column that was clicked.
    HeaderClicked { col_index: usize },

    /// User clicked the sort arrow to toggle direction or restore entry order.
    ///
    /// The arrow icon and row order have already been updated. `sort_order` is
    /// the new sort order in effect.
    SortArrowClicked { sort_order: SortOrder },

    /// A user event.
    ///
    /// One of the cells is returning an event from the per-cell future supplied
    /// via `step_with`. The table itself does not react to `User` events; it
    /// only forwards them.
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
/// # Features
///
/// - Column headers with raised/pressed bevel effects
/// - Dedicated sort arrow column (always visible)
/// - Resizable columns via drag handles
/// - Zebra-striped rows
/// - Single active sort column (or entry order when none active)
/// - Horizontal scroll overflow
///
/// # Type parameters
///
/// - `V: View` — the mogwai view abstraction (e.g. the web/DOM view).
/// - `T` — the per-row data model. Unconstrained; the table owns a `T` per
///   row, returns it by value from [`Table::remove`], and exposes `&T` /
///   `&mut T` via [`Table::get`] / [`Table::get_mut`]. Sorting and cell
///   rendering are delegated to per-column closures supplied at build time, so
///   `T` needs no trait bounds.
/// - `Cell` — the cell content type produced by the column accessor closures.
///   Inferred from the closure return type: returning a `V::Element` (e.g. from
///   `rsx! { span() }`) infers `Cell = V::Element`, the common case; returning
///   any other [`ViewChild<V>`], such as a full iti component (e.g.
///   [`Button`](crate::components::button::Button),
///   [`Checkbox`](crate::components::checkbox::Checkbox), etc.), infers that
///   component as `Cell` so a cell can be a full interactive component instead
///   of a bare element. All columns of a single `Table` share the same `Cell`
///   type, so mixing different component types across columns requires a
///   common `Cell` (often an enum with a manual [`ViewChild`] impl).
///
/// # Driving the event loop
///
/// The table does not run its own event loop. The caller drives it with the
/// `step()` convention (see [`StepMut::step_mut`] /
/// [`StepWithMut::step_with_mut`]) in a `loop { table.step_mut().await }`
/// pattern. Each `step` call awaits the next user action — a column header
/// click, sort-arrow click, column resize drag, or a per-cell future event —
/// and returns a [`TableEvent`].
///
/// ## React-before-return contract
///
/// By the time [`step_mut`] / [`step_with_mut`] returns, the table has already
/// updated its internal state and DOM to match the action (e.g. the active
/// column has been switched and rows re-sorted). The returned [`TableEvent`] is
/// an observation, not a command to react to. See [`TableEvent`] for the full
/// per-variant contract.
///
/// # Construction
///
/// Build a `Table` with [`TableBuilder`]; see the module-level example for a
/// complete usage walkthrough.
///
/// [`step_mut`]: StepMut::step_mut
/// [`step_with_mut`]: StepWithMut::step_with_mut
#[derive(ViewChild, ViewProperties)]
pub struct Table<V: View, T, Cell: ViewChild<V>> {
    #[child]
    #[properties]
    container: V::Element,
    table: V::Element,
    tbody: V::Element,
    headers: Vec<ColumnHeader<V>>,
    sort_header: SortArrowHeader<V>,
    rows: Vec<TableRow<V, T>>,
    columns: Vec<Column<T, Cell>>,
    active_sort_col: Proxy<Option<usize>>, // None = entry order
    sort_order: SortOrder,                 // Cached sort order value
    resize_state: Proxy<Option<ResizeState>>, // None when not resizing
    /// True once column widths have been measured and locked into state to
    /// prevent the browser from rescaling them. Set lazily on the first
    /// `step()` call after the table is laid out.
    normalized: bool,
}

/// Builder for constructing a [`Table`] with a fluent API.
///
/// Each [`TableBuilder::column`] call registers a new column (header label +
/// cell accessor + sort comparator). Sizing methods —
/// [`TableBuilder::width`], [`TableBuilder::width_percent`],
/// [`TableBuilder::width_auto`], [`TableBuilder::min_width`],
/// [`TableBuilder::fixed_width`] — apply to the **last-added** column, so the
/// chaining order is `column(...)` then sizing methods, repeat. Finish with
/// [`TableBuilder::build`] to obtain the [`Table`].
///
/// # Type parameters
///
/// - `V: View` — the mogwai view abstraction.
/// - `T` — the row data model the built [`Table`] will hold. Passed unchanged
///   to the table; see [`Table`] for what `T` requires (nothing).
/// - `Cell` — the cell content type the column accessor closures will return.
///   Inferred from the closure return type (see [`Table`]); see [`Table`] for
///   constraints on mixing different `Cell` types across columns.
///
/// # Example
///
/// See the module-level docs for a complete example. Short form:
///
/// ```ignore
/// let table = TableBuilder::new()
///     .column("Name", |item: &Row, _| { /* cell */ }, |a, b| a.name.cmp(&b.name))
///     .width_percent(60.0)
///     .column("Size", |item: &Row, _| { /* cell */ }, |a, b| a.size.cmp(&b.size))
///     .width(120)
///     .build();
/// ```
pub struct TableBuilder<V: View, T, Cell: ViewChild<V>> {
    use_scrollbar: bool,
    columns: Vec<Column<T, Cell>>,
    _view: std::marker::PhantomData<V>,
}

impl<V: View, T, Cell: ViewChild<V>> TableBuilder<V, T, Cell> {
    /// Create a new empty builder with no columns.
    ///
    /// Columns (and their sizing) are added via [`Self::column`] and the
    /// chaining methods; horizontal scrolling defaults to off (use
    /// [`Self::use_scrollbar`] to enable it).
    pub fn new() -> Self {
        Self {
            use_scrollbar: false,
            columns: vec![],
            _view: std::marker::PhantomData,
        }
    }

    /// Add a column with a header label, cell accessor, and sort comparator.
    ///
    /// # Parameters
    ///
    /// - **header** — name of the column, displayed in the header.
    /// - **create_cell_fn** — cell creation function. Takes a reference to the
    ///   row data `T` and the column index, returns the cell content of type
    ///   `Cell`. `Cell` is inferred from this closure's return type — returning
    ///   a `V::Element` (e.g. from `rsx! { span() }`) yields `Cell = V::Element`,
    ///   the common case; returning any other [`ViewChild<V>`] (such as a full
    ///   iti component) infers that type as `Cell`. Called once per row when the
    ///   row is added (e.g. via [`Table::push`]).
    /// - **compare_cell_fn** — sort comparison function. Compares two rows for
    ///   sort ordering and is invoked by [`Table::sort_by_column`].
    ///
    /// # Note
    ///
    /// Sizing methods ([`Self::width`], [`Self::width_percent`],
    /// [`Self::width_auto`], [`Self::min_width`], [`Self::fixed_width`]) apply
    /// to the column registered by the *immediately preceding* `column()` call.
    /// Call them right after `column()` to target the intended column. A newly
    /// added column defaults to [`ColumnSize::Auto`] sizing, `min_width` of
    /// `50`, and resizable `true`.
    pub fn column(
        mut self,
        header: impl Into<String>,
        create_cell_fn: impl Fn(&T, usize) -> Cell + 'static,
        compare_cell_fn: impl Fn(&T, &T) -> std::cmp::Ordering + 'static,
    ) -> Self {
        self.columns.push(Column {
            header: header.into(),
            create_cell_fn: Box::new(create_cell_fn),
            compare_cell_fn: Box::new(compare_cell_fn),
            declared_size: ColumnSize::Auto,
            min_width: 50,
            resizable: true,
        });
        self
    }

    /// Set a fixed pixel width for the **last-added** column.
    ///
    /// Overrides any prior [`Self::width_percent`] / [`Self::width_auto`] on
    /// the same column. Note that under `table-layout: fixed` the browser may
    /// still proportionally scale configured widths to fill the table; the
    /// table re-normalizes on the first `step()` after layout to make the
    /// configured widths match the rendered widths.
    pub fn width(mut self, width: u32) -> Self {
        if let Some(col) = self.columns.last_mut() {
            col.declared_size = ColumnSize::Pixels(width as f64);
        }
        self
    }

    /// Set a percentage width (0.0 – 100.0) for the **last-added** column.
    ///
    /// Overrides any prior [`Self::width`] / [`Self::width_auto`] on the same
    /// column. See [`Self::width`] for the normalization caveat under
    /// `table-layout: fixed`.
    pub fn width_percent(mut self, percent: f64) -> Self {
        if let Some(col) = self.columns.last_mut() {
            col.declared_size = ColumnSize::Percent(percent);
        }
        self
    }

    /// Set the **last-added** column to auto-sizing.
    ///
    /// Auto columns share equally in any width left over after the fixed
    /// (pixel) and percentage columns are laid out. This is the default for a
    /// newly added column; this method is only useful to revert a prior
    /// [`Self::width`] / [`Self::width_percent`] on the same column.
    pub fn width_auto(mut self) -> Self {
        if let Some(col) = self.columns.last_mut() {
            col.declared_size = ColumnSize::Auto;
        }
        self
    }

    /// Set the minimum width in pixels for the **last-added** column.
    ///
    /// This is a floor for user resize-drag operations only; it does **not**
    /// affect the column's initial layout width. Defaults to `50` for a newly
    /// added column.
    pub fn min_width(mut self, min_width: u32) -> Self {
        if let Some(col) = self.columns.last_mut() {
            col.min_width = min_width;
        }
        self
    }

    /// Make the **last-added** column non-resizable by the user.
    ///
    /// # Note
    ///
    /// Despite the name, this does **not** mean "fixed pixel width" — it only
    /// disables the resize-drag handle. The column's width still comes from
    /// [`Self::width`] / [`Self::width_percent`] / [`Self::width_auto`] (or
    /// the `Auto` default). To make a column both fixed-pixel and non-resizable,
    /// call both `width(px)` and `fixed_width()`.
    pub fn fixed_width(mut self) -> Self {
        if let Some(col) = self.columns.last_mut() {
            col.resizable = false;
        }
        self
    }

    /// Enable or disable a horizontal scrollbar on the built table.
    ///
    /// When enabled, the table's body clips to its container and shows a
    /// horizontal scrollbar for overflow; when disabled, overflow extends the
    /// layout. Defaults to `false`.
    pub fn use_scrollbar(mut self, use_scrollbar: bool) -> Self {
        self.use_scrollbar = use_scrollbar;
        self
    }

    /// Consume the builder and return the constructed [`Table`].
    ///
    /// The returned table starts empty; add rows with [`Table::push`].
    pub fn build(self) -> Table<V, T, Cell> {
        Table::from_builder(self)
    }
}

impl<V: View, T, Cell: ViewChild<V>> Default for TableBuilder<V, T, Cell> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View, T, Cell: ViewChild<V>> Table<V, T, Cell> {
    /// Create table from column definitions.
    fn from_builder(builder: TableBuilder<V, T, Cell>) -> Self {
        let TableBuilder {
            use_scrollbar,
            columns,
            _view,
        } = builder;
        // Create data column headers
        let mut headers = vec![];
        let num_columns = columns.len();
        for (col_index, col) in columns.iter().enumerate() {
            let mut state = Proxy::new(ColumnHeaderState {
                is_active: false,
                is_resizing: false,
                size: col.declared_size,
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

        // Resolve sort-arrow image URLs. With `embed-assets`, the SVG
        // bytes are compiled in and exposed as Blob URLs (memoized in
        // `assets::embedded`). Without the feature we fall back to the
        // relative path served by Trunk's `copy-dir` directive.
        let asc_src: String = if cfg!(feature = "embed-assets") {
            #[cfg(feature = "embed-assets")]
            {
                crate::assets::embedded::blob_url_for_table_sort_asc()
            }
            #[cfg(not(feature = "embed-assets"))]
            {
                unreachable!()
            }
        } else {
            "svg/table-sort-asc.svg".to_string()
        };
        let desc_src: String = if cfg!(feature = "embed-assets") {
            #[cfg(feature = "embed-assets")]
            {
                crate::assets::embedded::blob_url_for_table_sort_desc()
            }
            #[cfg(not(feature = "embed-assets"))]
            {
                unreachable!()
            }
        } else {
            "svg/table-sort-desc.svg".to_string()
        };

        rsx! {
            let arrow_img = img(
                class = "table-sort-arrow",
                src = sort_order(order => match order {
                    SortOrder::Ascending => asc_src.clone(),
                    SortOrder::Descending => desc_src.clone(),
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
            let container = div(class = "table-container") {
                let table = table(class = "table") {
                    {&colgroup_el}
                    {&thead}
                    {&tbody}
                }
            }
        }

        let table = Self {
            container,
            table,
            tbody,
            headers,
            sort_header,
            rows: vec![],
            columns,
            active_sort_col: Proxy::new(None),
            sort_order: SortOrder::Ascending,
            resize_state: Proxy::new(None),
            normalized: false,
        };
        table.set_use_scrollbar(use_scrollbar);
        table
    }

    /// Set whether the table body clips to its container with a horizontal
    /// scrollbar (`true`) or lets overflow extend the layout (`false`).
    ///
    /// This is the post-construction equivalent of
    /// [`TableBuilder::use_scrollbar`].
    pub fn set_use_scrollbar(&self, use_scrollbar: bool) {
        if use_scrollbar {
            self.add_class("table-scroll");
        } else {
            self.remove_class("table-scroll");
        }
    }

    fn create_row(&mut self, data: T) -> TableRow<V, T> {
        let mut cells = vec![];

        fn create_td<V: View, Cell: ViewChild<V>>(
            cell_content: Option<Cell>,
            col_idx: usize,
        ) -> V::Element {
            rsx! {
                let td = td(
                    class = "table-cell",
                    data:col_index = col_idx.to_string()
                ) {
                    {cell_content}
                }
            }
            td
        }

        // Create cells using column accessors
        for (col_idx, column) in self.columns.iter().enumerate() {
            let cell_content = (column.create_cell_fn)(&data, col_idx);
            let td = create_td::<V, Cell>(Some(cell_content), col_idx);
            cells.push(td);
        }
        // Create the last cell, which is always empty because it's under the sort header/button.
        cells.push(create_td::<V, Cell>(None, self.columns.len()));

        rsx! {
            let tr = tr(class = "table-row") {}
        }

        // Append cells to row
        for cell in &cells {
            tr.append_child(cell);
        }

        TableRow { tr, cells, data }
    }

    /// Add a row carrying `data` to the end of the table.
    ///
    /// The row's cells are produced immediately by the column accessor
    /// functions supplied via [`TableBuilder::column`]; the rendered cells
    /// reflect `data` at the moment `push` is called and are not reactive to
    /// later mutations of `T`. Use [`Table::get_mut`] to mutate the stored
    /// data (note that this does not re-render the existing cells).
    pub fn push(&mut self, data: T) {
        let row = self.create_row(data);
        // Append row to tbody
        self.tbody.append_child(&row.tr);
        self.rows.push(row);
    }

    /// Insert a row carrying `data` at `index`, shifting later rows down.
    ///
    /// # Panics
    ///
    /// Panics if `index > len` (out of bounds), matching `Vec::insert`.
    pub fn insert(&mut self, index: usize, data: T) {
        let row = self.create_row(data);
        // Insert row at the specified index in tbody
        let maybe_current_row_at_index = self.rows.get(index);
        self.tbody
            .insert_child_before(&row.tr, maybe_current_row_at_index.as_ref().map(|r| &r.tr));
        self.rows.insert(index, row);
    }

    /// Remove and return the row at `index`, dropping it from the DOM.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds, matching `Vec::remove`.
    pub fn remove(&mut self, index: usize) -> T {
        let row = self.rows.remove(index);
        self.tbody.remove_child(&row.tr);
        row.data
    }

    /// Borrow the row data at `index`, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.rows.get(index).map(|r| &r.data)
    }

    /// Mutably borrow the row data at `index`, or `None` if out of bounds.
    ///
    /// Note: mutating the returned `&mut T` does not re-render the row's cells,
    /// which were built from `data` at insertion time.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.rows.get_mut(index).map(|r| &mut r.data)
    }

    /// Number of rows currently in the table.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// `true` if the table has no rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Iterator over the row data (`&T`) in current DOM order.
    ///
    /// Note that DOM order reflects the last sort applied, not necessarily the
    /// original insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.rows.iter().map(|r| &r.data)
    }

    /// Set which column is actively sorted.
    ///
    /// Pass `Some(idx)` to activate column `idx` (highlighting its header and
    /// cells); pass `None` to deactivate the active column and return to entry
    /// order. This updates header/cell highlighting only — it does **not** sort
    /// the rows. Pair with [`Table::sort_by_column`] or
    /// [`Table::sort_by_entry_order`] to also reorder rows.
    pub fn set_active_sort_column(&mut self, col_index: Option<usize>) {
        self.active_sort_col.set(col_index);

        // Update header active states
        for (idx, header) in self.headers.iter_mut().enumerate() {
            header
                .state
                .modify(|s| s.is_active = Some(idx) == col_index);
        }

        // Update cell highlighting for active column. The `<td>` wrappers (in
        // `row.cells`) are `V::Element` and implement `ViewProperties`, so the
        // generic `add_class`/`remove_class` methods work without a web-specific
        // cast.
        for row in &self.rows {
            for (cell_idx, cell) in row.cells.iter().enumerate() {
                if Some(cell_idx) == col_index {
                    cell.add_class("active-column");
                } else {
                    cell.remove_class("active-column");
                }
            }
        }
    }

    /// Get the currently active sort column, or `None` when no column is
    /// active (entry order is in effect).
    pub fn get_active_sort_column(&self) -> Option<usize> {
        *self.active_sort_col
    }

    /// Get the current sort order in effect.
    ///
    /// Always returns a value (defaults to [`SortOrder::Ascending`] on a newly
    /// built table); see also [`Table::set_sort_order`] / [`Table::toggle_sort_order`].
    pub fn get_sort_order(&self) -> SortOrder {
        self.sort_order
    }

    /// Set the sort order, updating the sort-arrow icon to match.
    ///
    /// Reorders nothing on its own — pair with [`Table::sort_by_column`] or
    /// [`Table::sort_by_entry_order`] to apply the new order to the rows.
    pub fn set_sort_order(&mut self, order: SortOrder) {
        self.sort_header.sort_order.set(order);
        self.sort_order = order;
    }

    /// Toggle the sort order between ascending and descending, update the
    /// sort-arrow icon, and return the new order.
    ///
    /// Like [`Table::set_sort_order`], this does not reorder rows on its own.
    pub fn toggle_sort_order(&mut self) -> SortOrder {
        let new_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
        self.sort_header.sort_order.set(new_order);
        self.sort_order = new_order;
        new_order
    }

    /// Sort rows in place by the comparator of column `col_index`, in the
    /// given direction, and re-append them to the DOM in that order.
    ///
    /// Does nothing if `col_index` is out of bounds.
    ///
    /// # Note
    ///
    /// This does **not** update the active sort column — call
    /// [`Table::set_active_sort_column`] separately if you want the header and
    /// cells to reflect the active column. It also does not update the sort
    /// order / arrow icon; use [`Table::set_sort_order`] for that.
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

    /// Re-append rows in original insertion order (`Ascending`) or the reverse
    /// (`Descending`) to restore entry order in the DOM.
    ///
    /// # Note
    ///
    /// This does **not** clear the active sort column — call
    /// [`Table::set_active_sort_column`] with `None` separately if you want
    /// the header and cells to return to the un-highlighted state. It also does
    /// not update the sort order / arrow icon; use [`Table::set_sort_order`]
    /// for that.
    ///
    /// [`set_active_sort_column`]: Table::set_active_sort_column
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

    /// Drive one table interaction, racing header/sort/resize events against
    /// per-cell futures produced by `cell_step`.
    ///
    /// This is the shared engine behind [`StepMut::step_mut`] and
    /// [`StepWithMut::step_with_mut`]. Resize operations are handled in a loop
    /// and don't return events to the caller.
    ///
    /// ## Note
    /// By the time the event is returned, the table has already reacted to the
    /// event. For example, if `HeaderClick` is returned, the table has already
    /// re-sorted accordingly.
    async fn drive<Ev>(
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
                self.convert_all_to_pixels();
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
}
/// `step()` for tables with no per-cell event futures.
///
/// Each call awaits the next user action (header click, sort-arrow click, or
/// column resize drag) and returns the corresponding [`TableEvent`]. Per-cell
/// futures are not raced, so [`TableEvent::User`] is effectively unreachable
/// and `Ev` stays `()` — use [`StepWithMut`] (`step_with_mut`) instead when
/// individual cells produce their own events that need to bubble up.
///
/// Honors the react-before-return contract: see [`Table`] and [`TableEvent`].
impl<V: View, T, Cell: ViewChild<V>> StepMut for Table<V, T, Cell> {
    type Output = TableEvent;
    async fn step_mut(&mut self) -> TableEvent {
        self.drive(|_| std::future::pending().boxed()).await
    }
}

/// `step()` that also races per-cell event futures, one per row.
///
/// `f` is invoked once per row on every call, receiving `&mut T` (the row's
/// data) and returning a future whose output becomes the [`TableEvent::User`]
/// payload (typed as `Ev`). Whichever resolves first — a cell future or a
/// table-level action — wins the race and is returned. Use this variant when
/// cell content (e.g. an interactive control inside a cell) needs to surface
/// events through the table loop.
///
/// Honors the react-before-return contract: see [`Table`] and [`TableEvent`].
impl<V: View, T, Cell: ViewChild<V>> StepWithMut<T> for Table<V, T, Cell> {
    type Output<Ev: 'static> = TableEvent<Ev>;
    async fn step_with_mut<Ev>(
        &mut self,
        f: impl for<'a> FnMut(&'a mut T) -> Pin<Box<dyn Future<Output = Ev> + 'a>>,
    ) -> TableEvent<Ev>
    where
        Ev: 'static,
    {
        self.drive(f).await
    }
}
impl<V: View, T, Cell: ViewChild<V>> Table<V, T, Cell> {
    /// Measure rendered widths of all data column headers and write them back
    /// into state as Pixels.
    ///
    /// With `table-layout: fixed; width: 100%`, the browser scales configured
    /// column widths to fill the table. This means `state.size` (what we set)
    /// and the rendered width diverge. As soon as we change one size, the
    /// browser re-scales every column, amplifying our change by the scale factor.
    ///
    /// To break this cycle, we measure each column's rendered width using
    /// `getBoundingClientRect().width()` (sub-pixel precise) and write that exact
    /// value back into state. The configured sizes now equal the rendered widths
    /// and sum to the full table width, so the browser produces an identical
    /// layout with no shift. Sub-pixel precision avoids the integer-rounding snap
    /// that would occur when writing only `clientWidth`.
    ///
    /// Returns the captured widths so callers can use them as resize baselines.
    fn convert_all_to_pixels(&mut self) -> Vec<f64> {
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
            header.state.modify(|s| s.size = ColumnSize::Pixels(w));
        }
        widths
    }

    /// Convert all column sizes from Pixels (or other) to Percent based on current rendered widths.
    ///
    /// Measures each column's current rendered width and computes its percentage relative to
    /// the total table width.
    fn convert_all_to_percent(&mut self) {
        let table_width = self
            .table
            .dyn_el(|el: &web_sys::Element| el.get_bounding_client_rect().width())
            .unwrap_or(1.0)
            .max(1.0); // Prevent division by zero

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
            let percent = (w / table_width) * 100.0;
            header
                .state
                .modify(|s| s.size = ColumnSize::Percent(percent));
        }
    }

    /// Restore all column sizes to their originally declared sizes.
    ///
    /// Used after container resize ends to reset the layout to the user's original intent.
    #[allow(dead_code)]
    fn restore_declared_sizes(&mut self) {
        for (idx, header) in self.headers.iter_mut().enumerate() {
            if let Some(col) = self.columns.get(idx) {
                header.state.modify(|s| s.size = col.declared_size);
            }
        }
    }

    /// Handle the start of a column resize operation.
    fn handle_resize_start(&mut self, col_index: usize, mouse_x: i32) {
        // Re-normalize on every resize start so the system self-heals if the
        // container has been resized between operations.
        let initial_widths = self.convert_all_to_pixels();

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
                    .modify(|s| s.size = ColumnSize::Pixels(final_left_width));

                // Shrink donor columns
                for (donor_idx, new_width) in adjustments {
                    self.headers[donor_idx]
                        .state
                        .modify(|s| s.size = ColumnSize::Pixels(new_width));
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
                        .modify(|s| s.size = ColumnSize::Pixels(new_width));
                }

                // Grow right neighbor by the total space collected
                let right_neighbor_idx = col_index + 1;
                let right_initial = initial_widths[right_neighbor_idx];
                let new_right_width = right_initial + space_collected;
                self.headers[right_neighbor_idx]
                    .state
                    .modify(|s| s.size = ColumnSize::Pixels(new_right_width));
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

        // After resize ends, convert all columns back to percentages for fluid responsive layout
        self.convert_all_to_percent();
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
    struct TableLibraryItemInner<V: View> {
        #[child]
        container: V::Element,
        table: Table<V, FileEntry, V::Element>,
        log_text: Proxy<String>,
    }

    impl<V: View> TableLibraryItemInner<V> {
        fn new(with_scrollbar: bool) -> Self {
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
                .width_percent(40.0)
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
                .width_percent(30.0)
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
                .width_auto()
                .use_scrollbar(with_scrollbar)
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
            table.push(FileEntry {
                name: "Autobots".into(),
                date_modified: "Sat, Dec 19, 1337, 4:55 PM".into(),
                size: "666 K".into(),
                kind: "artificial intelligence".into(),
            });
            table.push(FileEntry {
                name: "Decepticons".into(),
                date_modified: "Sat, Dec 19, 1337, 4:55 PM".into(),
                size: "666 K".into(),
                kind: "artificial intelligence".into(),
            });

            if with_scrollbar {
                table.set_style("max-height", "200px");
            }

            let mut log_text = Proxy::new(
                "Click column headers to sort. Click arrow to toggle direction or restore entry order."
                    .to_string(),
            );

            rsx! {
                let container = div(class = "panel") {
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

    impl<V: View> StepMut for TableLibraryItemInner<V> {
        type Output = ();
        async fn step_mut(&mut self) {
            let event = self.table.step_mut().await;

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

    #[derive(ViewChild)]
    pub struct TableLibraryItem<V: View> {
        #[child]
        container: V::Element,
        table_with_scrollbar: TableLibraryItemInner<V>,
        table_without_scrollbar: TableLibraryItemInner<V>,
    }

    impl<V: View> Default for TableLibraryItem<V> {
        fn default() -> Self {
            let table_with_scrollbar = TableLibraryItemInner::new(true);
            let table_without_scrollbar = TableLibraryItemInner::new(false);
            rsx! {
                let container = div(class = "container-fluid") {
                    div(class = "row mb-4") {
                        p() { "With a scrollbar:" }
                        {&table_with_scrollbar}
                    }
                    div(class = "row mb-4") {
                        p() { bold() { "Without" } "a scrollbar:" }
                        {&table_without_scrollbar}
                    }
                }
            }
            Self {
                container,
                table_with_scrollbar,
                table_without_scrollbar,
            }
        }
    }

    impl<V: View> StepMut for TableLibraryItem<V> {
        type Output = ();
        async fn step_mut(&mut self) {
            loop {
                let with = self.table_with_scrollbar.step_mut();
                let without = self.table_without_scrollbar.step_mut();
                with.or(without).await;
            }
        }
    }
}
