use std::error::Error;
use std::future::Future;
use std::pin::Pin;

/// Describes a dynamic future.
///
/// Used on commands and extensions when implementing [GraphmanCommand]
/// to make the associated type shorter.
pub type BoxedFuture<Output, Error> =
    Pin<Box<dyn Future<Output = Result<Output, Error>> + Send + 'static>>;

/// Describes a dynamic error.
///
/// Used on commands and extensions when implementing [GraphmanCommand]
/// to make the associated type shorter.
pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;

/// Describes any command or extension which can be executed
/// and produce either an output or an error.
///
/// It is useful when extending commands with additional features such as assigning unique IDs,
/// running commands in the background, or tracking command executions,
/// while maintaining a unified API.
pub trait GraphmanCommand<Ctx> {
    type Output;
    type Error;
    type Future: Future<Output = Result<Self::Output, Self::Error>> + Send + 'static;

    fn execute(self, ctx: Ctx) -> Self::Future;
}
