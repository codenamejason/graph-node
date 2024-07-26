use graphman_primitives::DynamicContext;
use graphman_primitives::DynamicContextNotFound;
use graphman_primitives::ExtensibleGraphmanContext;

/// The data that is shared by all extensions.
pub struct GraphmanExtensionContext<Ctx> {
    inner: Ctx,
    dynamic: DynamicContext,
}

impl<Ctx> GraphmanExtensionContext<Ctx> {
    /// Creates a new shared context.
    pub fn new(inner: Ctx) -> Self {
        Self {
            inner,
            dynamic: DynamicContext::new(),
        }
    }
}

// Allows the commands to access their context.
impl<Ctx> AsRef<Ctx> for GraphmanExtensionContext<Ctx> {
    fn as_ref(&self) -> &Ctx {
        &self.inner
    }
}

// Allows the commands to access their context.
impl<Ctx> AsMut<Ctx> for GraphmanExtensionContext<Ctx> {
    fn as_mut(&mut self) -> &mut Ctx {
        &mut self.inner
    }
}

impl<Ctx> ExtensibleGraphmanContext for GraphmanExtensionContext<Ctx> {
    type Error = DynamicContextNotFound;

    fn extend<T>(&mut self, extension: T)
    where
        T: Send + Sync + 'static,
    {
        self.dynamic.extend(extension);
    }

    fn get<T>(&self) -> Result<&T, Self::Error>
    where
        T: Send + Sync + 'static,
    {
        self.dynamic.get()
    }
}
