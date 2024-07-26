use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphmanExtensionError {
    #[error("extension '{extension_name}' failed: {source}")]
    ExtensionFailed {
        extension_name: &'static str,
        source: anyhow::Error,
    },

    #[error("command failed in '{extension_name}': {source}")]
    CommandFailed {
        extension_name: &'static str,
        source: anyhow::Error,
    },

    #[error("context error in '{extension_name}': {source}")]
    Context {
        extension_name: &'static str,
        source: anyhow::Error,
    },

    #[error("datastore error in '{extension_name}': {source}")]
    Datastore {
        extension_name: &'static str,
        source: anyhow::Error,
    },
}

/// A little utility macro that allows creating [GraphmanExtensionError] bounded to an extension.
///
/// Usage:
/// - Call the `error_builder!("ExtensionName");` somewhere on top of an extension module;
/// - Call the `e!(ErrorVariant, source_error)` where necessary;
macro_rules! error_builder {
    ($ext:literal) => {
        macro_rules! e {
            ($kind:ident, $source:expr) => {
                $crate::GraphmanExtensionError::$kind {
                    extension_name: $ext,
                    source: $source,
                }
            };
        }
    };
}

pub(super) use error_builder;
