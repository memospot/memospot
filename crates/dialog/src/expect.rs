use crate::dialog::panic_dialog;
use crate::limit_width::LimitWidthExt;

/// Port of https://crates.io/crates/dialog-expect.
/// A trait with 1 function, `dialog_expect`,
/// which is implemented for [`std::option::Option<T>`] and [`std::result::Result<T, E>`].
pub trait ExpectDialogExt<T> {
    fn expect_dialog<M: std::fmt::Display>(self, msg: M) -> T;
}

impl<T> ExpectDialogExt<T> for std::option::Option<T> {
    /// Returns the contained [`Some`] value, consuming the `self` value.
    ///
    /// GUI alternative to [`std::result::Result::expect`].
    ///
    /// # Panics
    ///
    /// If the self value equals [`None`], panics with a dialog showing the passed message.
    ///
    /// Failures
    /// If the platform is unsupported by the dialog library, the program will panic to stdout.
    fn expect_dialog<M: std::fmt::Display>(self, msg: M) -> T {
        match self {
            Some(v) => v,
            None => {
                panic_dialog(msg);
            }
        }
    }
}

impl<T, E: std::fmt::Display> ExpectDialogExt<T> for std::result::Result<T, E> {
    /// Returns the contained [`Ok`] value, consuming the `self` value.
    ///
    /// GUI alternative to [`std::result::Result::expect`].
    ///
    /// # Panics
    ///
    /// If the value is an [`Err`], panics with a dialog showing
    /// the passed error message followed by the content of the [`Err`].
    /// Display format is `{Msg}:\n\n{err}`. Error will be width-limited to 96 characters.
    ///
    /// # Failures
    /// If the platform is unsupported by the dialog library, the program will panic to stdout.
    fn expect_dialog<M: std::fmt::Display>(self, msg: M) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                let mut s = msg.to_string();
                let ucfirst = s.remove(0).to_uppercase().to_string() + &s;
                panic_dialog(format!("{}:\n\n{}", ucfirst, e.limit_width()));
            }
        }
    }
}
