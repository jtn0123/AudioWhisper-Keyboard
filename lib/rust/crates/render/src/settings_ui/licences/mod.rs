//! ThirdPartyLicensesScreen: AboutLibraries' `LibrariesContainer` (Traditional rows, dialog
//! detail mode) over the generated `aboutlibraries.json`. Kotlin hands over the raw resource's
//! text (settings op 21); parsing, ordering, row text, layout and drawing happen here, and only
//! the rows and dialog lines on screen are laid out and drawn.
pub mod data;
mod dialog;
pub mod html;
mod list;
mod scanner;
mod scanner_table;
mod schema;
mod style;
mod tagsoup;
use super::App;
use std::sync::Arc;

pub const ROUTE: &str = "settings/about/third-party-licenses";
/// The document name Kotlin answers a `["load", name]` plan with.
pub const LIBRARIES: &str = "libraries";
/// Library rows' control ids: `ROW_ID + index`.
pub const ROW_ID: i32 = 10000;
/// The dialog's focusable, scrolling license text.
pub const BODY_ID: i32 = 9;

/// Each row's height and top, for one width, density and font scale.
#[derive(Clone, Debug, Default)]
pub(super) struct Geometry {
    pub key: [u32; 4],
    pub tops: Vec<f32>,
    pub heights: Vec<f32>,
}
impl Geometry {
    pub fn total(&self) -> f32 {
        self.tops
            .last()
            .zip(self.heights.last())
            .map_or(0.0, |(t, h)| t + h)
    }
    /// The row at content offset `y` and how far into it `y` is.
    pub fn at(&self, y: f32) -> (usize, f32) {
        let i = self.tops.partition_point(|&top| top <= y).saturating_sub(1);
        (i, y - self.tops.get(i).copied().unwrap_or(0.0))
    }
}
/// The open dialog's text, laid out once at its width.
#[derive(Debug)]
pub(super) struct Body {
    pub library: usize,
    pub width: f32,
    pub size: f32,
    pub text: String,
    pub layout: Arc<crate::android_text::Layout>,
}
#[derive(Debug, Default)]
pub struct State {
    /// The parsed resource, once Kotlin has sent it; until then the list is empty, as
    /// `produceLibraries`' initial null is.
    pub data: Option<Arc<data::Libraries>>,
    /// A `load` plan went out and has not been answered.
    pub requested: bool,
    pub(super) geometry: Geometry,
    /// `remember { visibleItemsInfo.size }`: the visible row count at the first laid-out frame.
    pub(super) visible_items: Option<usize>,
    /// The first visible row and its offset (negative: scrolled past), for the scrollbar.
    pub(super) first: Option<(usize, f32, f32)>,
    pub(super) body: Option<Body>,
}
impl App {
    /// A document Kotlin read for a `load` plan (settings op 21).
    pub fn set_document(&mut self, name: &str, text: &str) {
        if name == LIBRARIES {
            self.licences.data = Some(Arc::new(data::parse(text)));
            self.licences.requested = false;
            self.licences.geometry = Geometry::default();
            self.licences.body = None;
        }
    }
    /// Whether the top screen is the libraries list.
    pub fn on_licences(&self) -> bool {
        self.stack.last().is_some_and(|r| r == ROUTE)
    }
    /// Whether the open dialog is a library's license.
    pub(super) fn licence_modal(&self) -> bool {
        self.modal
            .as_ref()
            .is_some_and(|m| matches!(self.rows()[m.row].kind, super::catalogue::Kind::Libraries))
    }
    /// `onDialogRequest`: a row's tap opens its license dialog at the top.
    pub(super) fn open_licence(&mut self, row: usize, library: usize) {
        self.scrollbar_start[1] = -1;
        self.scrollbar_last[1] = -1;
        self.licences.body = None;
        self.modal = Some(super::Modal {
            row,
            selected: library.to_string(),
            values: vec![],
            scroll: 0.0,
            enabled: true,
        });
        self.focus = -1;
    }
    /// Keyboard focus order: the back button, then every library row.
    pub(super) fn licence_focus(&self) -> Vec<i32> {
        let count = self.licences.data.as_ref().map_or(0, |d| d.libraries.len());
        (0..count as i32).map(|j| ROW_ID + j).collect()
    }
}
