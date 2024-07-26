use std::error::Error;

/// Used by extensions that need to access or extend the dynamic context.
pub trait ExtensibleGraphmanContext {
    type Error: Error + Send + Sync + 'static;

    /// Adds new data to the context that is available to other extensions.
    ///
    /// Requires a custom, unique type for each piece of data.
    fn extend<T>(&mut self, extension: T)
    where
        T: Send + Sync + 'static;

    /// Returns data previously set by other extensions.
    fn get<T>(&self) -> Result<&T, Self::Error>
    where
        T: Send + Sync + 'static;
}
