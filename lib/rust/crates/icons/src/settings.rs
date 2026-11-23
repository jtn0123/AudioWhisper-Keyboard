//! The settings screens' icons: the `@icon:name` each screen asks for, as `settingsIcons` gave
//! it. `home0` to `home11` are the home screen's rows.

/// The icon `@icon:name` names, or `None` for a name no screen uses.
pub fn settings(name: &str) -> Option<&'static str> {
    if let Some(home) = name.strip_prefix("home") {
        return HOME.get(home.parse::<usize>().ok()?).copied();
    }
    let at = SETTINGS.binary_search_by(|(n, _)| n.cmp(&name)).ok()?;
    Some(SETTINGS[at].1)
}

/// The home screen's rows, `home0` to `home11`.
const HOME: [&str; 12] = [
    "Filled.Language",
    "Outlined.Palette",
    "Outlined.Keyboard",
    "Filled.SmartButton",
    "Filled.Mic",
    "Filled.Spellcheck",
    "Filled.Gesture",
    "AutoMirrored.Outlined.Assignment",
    "Filled.SentimentSatisfiedAlt",
    "Filled.Extension",
    "Outlined.Build",
    "Outlined.Info",
];

/// Every other name, sorted so [`settings`] can search it.
pub(crate) const SETTINGS: [(&str, &str); 69] = [
    ("add", "Filled.Add"),
    ("archive", "Filled.Archive"),
    ("arrow_downward", "Filled.ArrowDownward"),
    ("arrow_drop_down", "Filled.ArrowDropDown"),
    ("arrow_upward", "Filled.ArrowUpward"),
    ("attach_file", "Filled.AttachFile"),
    ("back", "AutoMirrored.Filled.ArrowBack"),
    ("brightness_2", "Filled.Brightness2"),
    ("brightness_auto", "Filled.BrightnessAuto"),
    ("check_box", "Filled.CheckBox"),
    ("check_box_outline_blank", "Filled.CheckBoxOutlineBlank"),
    ("clear", "Filled.Clear"),
    ("clock", "Filled.Schedule"),
    ("code", "Filled.Code"),
    ("color_lens", "Filled.ColorLens"),
    ("contacts", "Filled.Contacts"),
    ("content_copy", "Filled.ContentCopy"),
    ("dark_mode", "Filled.DarkMode"),
    ("delete", "Filled.Delete"),
    ("description", "Outlined.Description"),
    ("done", "Filled.Done"),
    ("edit", "Filled.Edit"),
    ("emoji_symbols", "Outlined.EmojiSymbols"),
    ("error_outline", "Filled.ErrorOutline"),
    ("file_blank", "drawable.ic_file_blank"),
    ("file_download", "Outlined.FileDownload"),
    ("font_download", "Filled.FontDownload"),
    ("format_align_center", "Filled.FormatAlignCenter"),
    ("format_align_justify", "Filled.FormatAlignJustify"),
    ("format_align_left", "AutoMirrored.Filled.FormatAlignLeft"),
    ("format_align_right", "AutoMirrored.Filled.FormatAlignRight"),
    ("format_bold", "Filled.FormatBold"),
    ("format_color_fill", "Filled.FormatColorFill"),
    ("format_italic", "Filled.FormatItalic"),
    ("format_size", "Filled.FormatSize"),
    ("format_strikethrough", "Filled.FormatStrikethrough"),
    ("format_underlined", "Filled.FormatUnderlined"),
    ("help_outline", "AutoMirrored.Filled.HelpOutline"),
    ("history", "Filled.History"),
    ("incognito", "drawable.ic_incognito"),
    ("info", "Outlined.Info"),
    ("input", "AutoMirrored.Filled.Input"),
    ("keyboard", "Filled.Keyboard"),
    ("keyboard_keys", "drawable.ic_keyboard_keys"),
    ("language", "Filled.Language"),
    ("library_books", "AutoMirrored.Filled.LibraryBooks"),
    ("light_mode", "Filled.LightMode"),
    ("link", "Filled.Link"),
    ("mail", "Outlined.Mail"),
    ("open_in_full", "Filled.OpenInFull"),
    ("open_in_new", "AutoMirrored.Filled.OpenInNew"),
    ("padding", "Filled.Padding"),
    ("pageview", "Filled.Pageview"),
    ("palette", "Filled.Palette"),
    ("photo", "Filled.Photo"),
    ("picker", "Filled.Keyboard"),
    ("policy", "Outlined.Policy"),
    ("schedule", "Outlined.Schedule"),
    ("search", "Filled.Search"),
    ("settings_backup_restore", "Filled.SettingsBackupRestore"),
    ("share", "Filled.Share"),
    ("shop", "Filled.Shop"),
    ("space_bar", "Filled.SpaceBar"),
    ("straighten", "Filled.Straighten"),
    ("text_fields", "Filled.TextFields"),
    ("tune", "Filled.Tune"),
    ("warning", "Outlined.Warning"),
    ("wb_twilight", "Filled.WbTwilight"),
    ("wrap_text", "AutoMirrored.Filled.WrapText"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_sorted_so_it_can_be_searched() {
        assert!(SETTINGS.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(settings("home3"), Some("Filled.SmartButton"));
        assert_eq!(settings("home12"), None);
        assert_eq!(settings("clock"), Some("Filled.Schedule"));
        assert_eq!(settings("nothing"), None);
    }
}
