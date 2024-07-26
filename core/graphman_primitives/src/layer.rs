/// Makes it possible to add functionality to commands by layering
/// new functionality on top of existing functionality.
pub trait GraphmanLayer<C> {
    /// The type that contains the additional functionality and wraps the command.
    type Outer;

    /// Extends a command with additional functionality.
    fn layer(self, inner: C) -> Self::Outer;
}
