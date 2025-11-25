use dialog::*;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, LanguageLoader, Localizer,
};
use log::{debug, error};
use rust_embed::RustEmbed;
use std::slice;
use std::sync::LazyLock;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LOCALE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect_dialog("error while loading fallback locale");

    loader
});

/// Get a localized string.
///
/// Most of the time, it's better to use the `fl!` macro instead.
pub fn fl(message_id: &str) -> String {
    LOCALE_LOADER.get(message_id)
}
/// Get a localized string.
///
/// This does compile time check to ensure the `message_id` is valid.
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LOCALE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LOCALE_LOADER, $message_id, $($args), *)
    }};
}

/// Get the `Localizer` to be used for localizing this library.
pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LOCALE_LOADER, &Localizations))
}

/// Initialize the application localization with the operating system's default locale.
///
/// Should be one of the first app calls, so the panic messages get localized.
pub fn localize() {
    let localizer = localizer();
    let requested_locales = i18n_embed::DesktopLanguageRequester::requested_languages();

    #[cfg(debug_assertions)]
    {
        println!("requested_locales: {requested_locales:?}");
        for locale in requested_locales.iter() {
            println!("Requested locale for localization: {locale}");
        }
    }

    localizer
        .select(&requested_locales)
        .expect_dialog("error while loading locale");
}

/// Reload the localization with the user-preferred locale, if possible.
///
/// Should be called after the configuration has been loaded.
/// TODO: add fallback handling.
/// TODO: fix override on KDE and macOS. (confirmed working only on GNOME and Windows)
pub fn reload(preferred_locale: &str) {
    if preferred_locale.is_empty() || preferred_locale == "system" {
        return;
    }

    let localizer = localizer();
    let requested_locales = i18n_embed::DesktopLanguageRequester::requested_languages();

    if let Some(locale) = requested_locales
        .iter()
        .find(|l| l.language.as_str() == preferred_locale)
    {
        if let Err(error) = localizer.select(slice::from_ref(locale)) {
            error!("failed to load locale: {error}");
            return;
        }
        debug!("locale overridden to {preferred_locale}");
    }
}
