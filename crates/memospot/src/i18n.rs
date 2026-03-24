use dialog::*;
use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
    unic_langid::LanguageIdentifier,
};
use log::{debug, error, warn};
use rust_embed::RustEmbed;
use std::collections::HashSet;
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

fn normalize_locale_tag(locale: &str) -> String {
    locale.trim().replace('_', "-")
}

fn mapped_fallback(locale: &str) -> Option<&'static str> {
    match locale {
        "zh-HK" | "zh-TW" => Some("zh-Hant"),
        "zh" => Some("zh-Hans"),
        _ => None,
    }
}

fn push_unique_locale(
    requested: &mut Vec<LanguageIdentifier>,
    seen: &mut HashSet<LanguageIdentifier>,
    locale: LanguageIdentifier,
) {
    if seen.insert(locale.clone()) {
        requested.push(locale);
    }
}

pub fn resolve_supported_locale(
    raw_locale: &str,
    available_languages: &[LanguageIdentifier],
) -> Option<LanguageIdentifier> {
    let normalized = normalize_locale_tag(raw_locale);
    if normalized.is_empty() {
        return None;
    }

    let parse_and_check = |locale: &str| {
        locale
            .parse::<LanguageIdentifier>()
            .ok()
            .filter(|parsed| available_languages.contains(parsed))
    };

    if let Some(locale) = parse_and_check(normalized.as_str()) {
        return Some(locale);
    }

    if let Some(mapped) = mapped_fallback(normalized.as_str())
        && let Some(locale) = parse_and_check(mapped)
    {
        return Some(locale);
    }

    let base_language = normalized.split('-').next().unwrap_or_default();
    if base_language.is_empty() {
        return None;
    }

    if let Some(mapped) = mapped_fallback(base_language)
        && let Some(locale) = parse_and_check(mapped)
    {
        return Some(locale);
    }

    let mut same_language_candidates: Vec<LanguageIdentifier> = available_languages
        .iter()
        .filter(|locale| locale.language.as_str() == base_language)
        .cloned()
        .collect();

    same_language_candidates.sort_by_key(|locale| locale.to_string());
    same_language_candidates.into_iter().next()
}

/// Initialize the application localization with the operating system's default locale.
///
/// Should be one of the first app calls, so the panic messages get localized.
pub fn localize() {
    let localizer = localizer();
    let mut requested_locales = i18n_embed::DesktopLanguageRequester::requested_languages();

    if requested_locales.is_empty() {
        requested_locales.push(LOCALE_LOADER.fallback_language().clone());
    }

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
pub fn reload(preferred_locale: &str) {
    if preferred_locale.is_empty() || preferred_locale == "system" {
        return;
    }

    let localizer = localizer();
    let available_languages = match localizer.available_languages() {
        Ok(languages) => languages,
        Err(error) => {
            error!("failed to list available locales: {error}");
            Vec::new()
        }
    };

    let preferred_locale = preferred_locale.trim();
    let mut requested_languages = Vec::new();
    let mut seen_languages = HashSet::new();

    if available_languages.is_empty() {
        let normalized = normalize_locale_tag(preferred_locale);
        if let Ok(preferred_language) = normalized.parse::<LanguageIdentifier>() {
            push_unique_locale(
                &mut requested_languages,
                &mut seen_languages,
                preferred_language,
            );
        }
    } else if let Some(preferred_language) =
        resolve_supported_locale(preferred_locale, &available_languages)
    {
        push_unique_locale(
            &mut requested_languages,
            &mut seen_languages,
            preferred_language,
        );
    }

    if requested_languages.is_empty() {
        let system_requested = i18n_embed::DesktopLanguageRequester::requested_languages();
        for system_locale in system_requested {
            if available_languages.is_empty() {
                push_unique_locale(
                    &mut requested_languages,
                    &mut seen_languages,
                    system_locale,
                );
                continue;
            }

            let system_locale_tag = system_locale.to_string();
            if let Some(resolved) =
                resolve_supported_locale(system_locale_tag.as_str(), &available_languages)
            {
                push_unique_locale(&mut requested_languages, &mut seen_languages, resolved);
            }
        }
    }

    if requested_languages.is_empty() {
        push_unique_locale(
            &mut requested_languages,
            &mut seen_languages,
            LOCALE_LOADER.fallback_language().clone(),
        );
    }

    match localizer.select(&requested_languages) {
        Ok(selected_languages) => {
            if let Some(selected) = selected_languages.first() {
                let requested = requested_languages.first().cloned();
                if requested.as_ref().is_some_and(|first| first == selected) {
                    debug!("locale overridden to {preferred_locale}");
                } else {
                    warn!(
                        "requested locale `{preferred_locale}` is unavailable; using `{selected}`"
                    );
                }
            }
        }
        Err(error) => {
            error!("failed to load locale override `{preferred_locale}`: {error}");
        }
    }
}
