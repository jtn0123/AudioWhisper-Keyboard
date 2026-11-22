//! The extension editor's screens (`settings_ui/extensions/editor`): ExtensionEditScreen and the
//! sheets it switched between, each a screen here as the theme editor's dialogs are. Rows whose
//! kind is `Ext(Part::Editor(_))` are painted by that module; Cancel, Save, Apply, Create and
//! Add are the bottom bar the subtype editor has (`Lang::Cancel`, `Lang::Save`), and the
//! unsaved changes question and the invalid metadata message are dialogs on hidden rows.
use super::{Cond, Kind, Row, Screen, extensions::Part, languages::Lang, row};
use crate::settings_ui::extensions::editor as screens;

/// The extension editor's own parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit {
    /// Opening the extension, or why it cannot be edited.
    Status,
    /// The box of Metadata, Dependencies and Files.
    Manage,
    /// The themes: a header with Add, then a box each with Delete and Edit.
    Components,
    /// The metadata's fields, in ManageMetaDataScreen's order: 0 the id, 1 the version, 2 the
    /// title, 3 the description, 4 the keywords, 5 the homepage, 6 the issue tracker, 7 the
    /// maintainers and 8 the license.
    Meta(usize),
    /// An info card of the row's label.
    Note,
    /// The dependencies: what there is, or that there are none.
    Dependencies,
    /// A folder's files, 0 the fonts and 1 the images: a header with Add, then a row each.
    Files(usize),
    /// The file name, on the file's properties and on adding a picked file.
    FileName,
    /// Create component's From existing and Empty.
    From,
    /// The components a new one can copy.
    Sources,
    /// The empty component's fields: 0 the id, 1 the label, 2 the authors.
    New(usize),
}

const fn edit(e: Edit) -> Row {
    row("", "", "", Kind::Ext(Part::Editor(e)))
}
const fn field(label: &'static str, e: Edit) -> Row {
    row("", label, "", Kind::Ext(Part::Editor(e)))
}
const fn info(label: &'static str, shown: Cond) -> Row {
    row("", label, "", Kind::Ext(Part::Editor(Edit::Note))).shown(shown)
}
const fn screen(route: &'static str, title: &'static str, rows: &'static [Row]) -> Screen {
    Screen {
        route,
        title,
        icon_space: true,
        preview: false,
        rows,
    }
}
const CANCEL: Row = row("", "action__cancel", "", Kind::Lang(Lang::Cancel));
const fn confirm(label: &'static str) -> Row {
    row("", label, "", Kind::Lang(Lang::Save))
}

/// The editor's title is Create or Edit theme extension (`extensions::ext_title`). Its bottom
/// bar shows once the extension is open.
pub const EDIT: Screen = screen(
    screens::EDIT,
    "ext__editor__title_edit_theme",
    &[
        edit(Edit::Status),
        edit(Edit::Manage),
        edit(Edit::Components),
        CANCEL.shown(Cond::ExtEditor),
        confirm("action__save").shown(Cond::ExtEditor),
        row(
            "",
            "action__discard_confirm_title",
            "action__discard_confirm_message",
            Kind::Lang(Lang::Unsaved),
        )
        .shown(Cond::Never),
        row(
            "",
            "ext__editor__metadata__title_invalid",
            "",
            Kind::Lang(Lang::Message),
        )
        .shown(Cond::Never),
    ],
);
pub const META: Screen = screen(
    screens::META,
    "ext__editor__metadata__title",
    &[
        field("ext__meta__id", Edit::Meta(0)),
        field("ext__meta__version", Edit::Meta(1)),
        field("ext__meta__title", Edit::Meta(2)),
        field("ext__meta__description", Edit::Meta(3)),
        field("ext__meta__keywords", Edit::Meta(4)),
        field("ext__meta__homepage", Edit::Meta(5)),
        field("ext__meta__issue_tracker", Edit::Meta(6)),
        field("ext__meta__maintainers", Edit::Meta(7)),
        field("ext__meta__license", Edit::Meta(8)),
        CANCEL,
        confirm("action__apply"),
    ],
);
/// ManageDependenciesScreen's note was English; it is a string here.
pub const DEPENDENCIES: Screen = screen(
    screens::DEPENDENCIES,
    "ext__editor__dependencies__title",
    &[
        info("settings__ext_editor__dependencies_note", Cond::Always),
        edit(Edit::Dependencies),
    ],
);
pub const FILES: Screen = screen(
    screens::FILES,
    "ext__editor__files__title",
    &[
        field("ext__editor__files__type_fonts", Edit::Files(0)),
        field("ext__editor__files__type_images", Edit::Files(1)),
    ],
);
/// A file's properties: its name, Cancel and Apply, and Delete in the top bar, where the
/// subtype editor has its Delete.
pub const FILE: Screen = screen(
    screens::FILE,
    "general__properties",
    &[
        field("general__file_name", Edit::FileName),
        CANCEL,
        confirm("action__apply"),
    ],
);
pub const FILE_ADD: Screen = screen(
    screens::FILE_ADD,
    "action__import_file",
    &[
        field("general__file_name", Edit::FileName),
        CANCEL,
        confirm("action__add"),
    ],
);
pub const COMPONENT_ADD: Screen = screen(
    screens::COMPONENT_ADD,
    "ext__editor__create_component__title_theme",
    &[
        edit(Edit::From),
        edit(Edit::Sources),
        info(
            "ext__editor__create_component__from_empty_warning",
            Cond::ExtEmpty,
        ),
        field("ext__meta__id", Edit::New(0)).shown(Cond::ExtEmpty),
        field("ext__meta__label", Edit::New(1)).shown(Cond::ExtEmpty),
        field("ext__meta__authors", Edit::New(2)).shown(Cond::ExtEmpty),
        CANCEL,
        confirm("action__create").on(Cond::ExtCreate),
    ],
);
