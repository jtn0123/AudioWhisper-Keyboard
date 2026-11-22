//! Where the settings layout and focus order meet the language screens' own rows.
use super::{form, packs};
use crate::settings_ui::{App, catalogue::languages::Lang};
use crate::{panels::Panel, text::Text};
use keys::Rect;

impl App {
    /// Paints row `i`, part `l`; its height. Dialog rows and the bottom bar's buttons take none.
    pub(in crate::settings_ui) fn lang_part(
        &mut self,
        p: &mut Panel,
        i: usize,
        l: Lang,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        match l {
            Lang::Presets => self.presets_row(p, i, at, clip, text),
            Lang::Gap => self.px(32.0),
            Lang::Search => self.search_row(p, i, at, clip, text),
            Lang::Locales => self.locales_row(p, i, at, clip, text),
            Lang::Import => self.import_row(p, at, clip, text),
            Lang::Packs => self.packs_row(p, at, clip, text),
            Lang::Message
            | Lang::AllPresets
            | Lang::DeletePack
            | Lang::Cancel
            | Lang::Save
            | Lang::Discard
            | Lang::DeleteRule => 0.0,
        }
    }
    /// Row `i`'s place in the focus order. The presets row stands for the top bar's Delete
    /// when editing, so it follows Back.
    pub(in crate::settings_ui) fn language_focus(&self, i: usize, l: Lang) -> Vec<i32> {
        match l {
            Lang::Presets if self.header_delete() => vec![form::DELETE_ID],
            Lang::Presets if self.draft.as_ref().is_some_and(|d| d.adding) => {
                let n = self.suggested_presets().len().min(form::MOST) as i32;
                let mut out: Vec<i32> = (0..n).map(|k| form::SUGGESTION_ID + k).collect();
                out.push(form::ALL_ID);
                out
            }
            Lang::Search | Lang::Cancel | Lang::Save => vec![1000 + i as i32],
            Lang::Locales => self.locales_focus(i),
            Lang::Import => vec![packs::IMPORT_ID],
            Lang::Packs => self.packs_focus(),
            _ => vec![],
        }
    }
}
