//! A theme's `@font` files draw its labels (`render::theme_fonts`): the fixture script face shows
//! in the pixels of the key labels and the Smartbar, each shaped run names the face it came from,
//! a glyph the theme font lacks falls through to the default chain, and a broken or missing file
//! is no font at all. The fixtures are `tests/fixtures/THEME-FONT-SUBSETS.md`.
mod broken;
mod common;
mod drawn;
