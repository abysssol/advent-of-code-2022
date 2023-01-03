//! Various helpers to get input from the command line,
//! specifically made for advent of code 2022.
//!
//! The simplest way to get input is to hook into [`with`].
//! ```no_run
//! use input::Description;
//!
//! input::with(
//!     Description {
//!         name: "<name>",
//!         bin_name: "<binary-name>".into(),
//!         description: "<description>",
//!         version: (0, 0, 0),
//!     },
//!     |input| {
//!         // app logic here
//!         Ok(())
//!     },
//! );
//! ```

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::FusedIterator;
use std::ops::Deref;
use std::{env, fmt, fs, io, process};

/// Provides input for advent of code to the provided function.
///
/// Provides a [`String`] with the input collected from standard input or a file,
/// as specified with command line arguments.
/// If any errors are encountered, they will be displayed and the app will exit.
pub fn with(description: Description, main: impl FnOnce(String) -> Result<(), SomeError>) {
    if let Err(error) = get(description).and_then(main) {
        eprintln!("{error:#}");
        process::exit(1);
    }
}

/// Returns a [`String`] containing input.
///
/// Returns a [`String`] with the input collected from standard input or a file,
/// as specified with command line arguments.
///
/// # Errors
///
/// An error is returned if no arguments are passed,
/// or if an error is encountered while reading input from stdin or a file.
pub fn get(description: Description) -> Result<String, SomeError> {
    let input = Input::from_args(env::args(), description)
        .map_err(NoInput::display_help)?
        .read_to_string()?;

    Ok(input)
}

/// Metadata of the app to be used when displaying help information.
#[derive(Debug, Clone)]
pub struct Description {
    pub name: &'static str,
    /// The name of the binary executing this code.
    pub bin_name: Cow<'static, str>,
    pub description: &'static str,
    pub version: (u16, u16, u16),
}

/// The location to search for input; either a named file or stdin.
#[derive(Debug, Clone)]
pub enum Input {
    File(String),
    Stdin,
}

impl Input {
    /// Parse arguments for input source.
    ///
    /// # Errors
    ///
    /// If help information is requested, version information is requested,
    /// or no arguments are passed at all, then [`NoInput`] is returned.
    pub fn from_args(
        mut args: impl Iterator<Item = String>,
        mut description: Description,
    ) -> Result<Self, NoInput> {
        if let Some(bin_name) = args.next() {
            description.bin_name = Cow::from(bin_name);
        }

        let file_is = |file, description| match file {
            Some(file) => Ok(Self::File(file)),
            None => Err(NoInput::NoArgs(description)),
        };

        let input = args.next();
        match input.as_deref() {
            Some("--help" | "-h") => Err(NoInput::Help(description)),
            Some("--version" | "-V") => Err(NoInput::Version(description)),
            Some("--stdin" | "-0") => Ok(Self::Stdin),
            Some("--") => file_is(args.next(), description),
            _ => file_is(input, description),
        }
    }

    /// Returns a [`String`] containing the input collected from standard input or a file.
    ///
    /// # Errors
    ///
    /// If an error is encountered while reading input from stdin or a file,
    /// then [`io::Error`] is returned.
    /// See [`fs::read_to_string`] and [`io::read_to_string`] for more information.
    pub fn read_to_string(self) -> Result<String, IoError> {
        match self {
            Self::File(ref file) => fs::read_to_string(file),
            Self::Stdin => io::read_to_string(io::stdin()),
        }
        .map_err(|error| IoError { input: self, error })
    }
}

/// An error returned when no input source is specified.
#[derive(Debug, Clone)]
pub enum NoInput {
    /// No valid arguments have been found.
    NoArgs(Description),
    /// Help text has been requested.
    Help(Description),
    /// Version information has been requested.
    Version(Description),
}

impl NoInput {
    /// Return the app metadata, ignoring the error cause.
    pub const fn description(&self) -> &Description {
        match self {
            Self::NoArgs(description) | Self::Help(description) | Self::Version(description) => {
                description
            }
        }
    }

    /// Display help or version information, then exit. Does nothing with [`Self::NoArgs`].
    #[must_use]
    pub fn display_help(self) -> Self {
        if let Self::Help(_) | Self::Version(_) = self {
            println!("{self}");
            process::exit(0);
        }

        self
    }
}

impl Error for NoInput {}

impl Display for NoInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Description {
            name,
            bin_name,
            description,
            version: (major, minor, patch),
        } = self.description();

        match self {
            Self::NoArgs(_) => write!(
                f,
                "\
The following required argument was not provided: <FILE>

Usage: {bin_name} [OPTIONS] [FILE]

For more information try '--help'"
            ),
            Self::Help(_) => write!(
                f,
                "\
{name} {major}.{minor}.{patch}
Solution app for advent of code 2022.
{description}

Usage: {bin_name} [OPTIONS] [FILE]

Args:
    <FILE>    File to read as input

Options:
    -h, --help       Print help information
    -V, --version    Print version information
    -0  --stdin      Read input from stdin instead of a file"
            ),
            Self::Version(_) => write!(f, "{name} {major}.{minor}.{patch}"),
        }
    }
}

/// An error wrapping [`io::Error`] with more context.
#[derive(Debug)]
pub struct IoError {
    pub input: Input,
    pub error: io::Error,
}

impl Error for IoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

impl Display for IoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.input {
            Input::File(ref file) => write!(f, "can't read file '{file}'"),
            Input::Stdin => write!(f, "can't read from stdin"),
        }
    }
}

/// A thread safe dynamically typed error.
///
/// Use the alternate formatting to display all error sources.
/// ```
/// use input::{Input, IoError, SomeError};
/// use std::io::{self, ErrorKind};
///
/// let error = IoError {
///     input: Input::Stdin,
///     error: io::Error::new(ErrorKind::NotFound, "os error opening stdin"),
/// };
/// let error = SomeError::new(error);
///
/// // with normal formatting, just pass through to the inner error
/// assert_eq!(format!("{error}"), "can't read from stdin");
///
/// // with alternate formatting, pretty print error and all sources
/// assert_eq!(
///     format!("{error:#}"),
///     concat!(
///         "error: can't read from stdin\n",
///         "  - os error opening stdin\n"
///     )
/// );
/// ```
#[derive(Debug)]
pub struct SomeError(pub Box<dyn Error + Send + Sync + 'static>);

impl SomeError {
    pub fn new(error: impl Error + Send + Sync + 'static) -> Self {
        Self(Box::new(error))
    }

    /// Iterate over this error and all of its sources.
    pub const fn iter(&self) -> ErrorChain<'_> {
        ErrorChain::new(&*self.0)
    }

    pub fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.deref().source()
    }
}

impl Display for SomeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let error = &*self.0;

        if f.alternate() {
            writeln!(f, "error: {error}")?;
            for error in self.iter().skip(1) {
                writeln!(f, "  - {error}")?;
            }

            Ok(())
        } else {
            Display::fmt(error, f)
        }
    }
}

impl<E: Error + Send + Sync + 'static> From<E> for SomeError {
    fn from(error: E) -> Self {
        Self::new(error)
    }
}

impl<'a> IntoIterator for &'a SomeError {
    type Item = &'a (dyn Error + 'static);
    type IntoIter = ErrorChain<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over an error and all of its sources.
#[derive(Debug, Clone)]
pub struct ErrorChain<'a>(Option<&'a (dyn Error + 'static)>);

impl<'a> ErrorChain<'a> {
    pub const fn new(error: &'a (dyn Error + 'static)) -> Self {
        Self(Some(error))
    }
}

impl<'a> Iterator for ErrorChain<'a> {
    type Item = &'a (dyn Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0;
        self.0 = next.and_then(Error::source);
        next
    }
}

impl<'a> FusedIterator for ErrorChain<'a> {}
