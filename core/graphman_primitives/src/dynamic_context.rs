use std::any::{Any, TypeId};
use std::collections::HashMap;

use thiserror::Error;

use crate::ExtensibleGraphmanContext;

/// Provides a way for command extensions to share additional
/// command execution data with other extensions.
pub struct DynamicContext {
    inner: HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
}

#[derive(Debug, Error)]
#[error("dynamic context '{0}' not found")]
pub struct DynamicContextNotFound(&'static str);

impl DynamicContext {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

impl ExtensibleGraphmanContext for DynamicContext {
    type Error = DynamicContextNotFound;

    fn extend<T>(&mut self, extension: T)
    where
        T: Send + Sync + 'static,
    {
        self.inner.insert(TypeId::of::<T>(), Box::new(extension));
    }

    fn get<T>(&self) -> Result<&T, Self::Error>
    where
        T: Send + Sync + 'static,
    {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
            .ok_or_else(|| DynamicContextNotFound(std::any::type_name::<T>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct A(i32);

    #[derive(Debug, PartialEq, Eq)]
    struct B(i32);

    #[test]
    fn access_undefined_extensions() {
        let ctx = DynamicContext::new();

        assert!(ctx.get::<A>().is_err());
        assert!(ctx.get::<B>().is_err());
    }

    #[test]
    fn access_defined_extensions() {
        let mut ctx = DynamicContext::new();

        ctx.extend(A(1));
        ctx.extend(B(2));

        assert_eq!(ctx.get::<A>().unwrap(), &A(1));
        assert_eq!(ctx.get::<B>().unwrap(), &B(2));
    }

    #[test]
    fn overwrite_extensions() {
        let mut ctx = DynamicContext::new();

        ctx.extend(A(1));
        ctx.extend(A(2));

        assert_eq!(ctx.get::<A>().unwrap(), &A(2));
    }
}
