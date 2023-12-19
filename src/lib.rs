use std::{marker::PhantomData, mem::ManuallyDrop};

/// Unique type implementing the `Brand` trait.
struct Opaque;

mod sealed {
    pub trait Sealed {}
}

/// Trait used as bound for brand variables.
pub trait Brand: sealed::Sealed {}

impl sealed::Sealed for Opaque {}

impl Brand for Opaque {}

/// Implementations of this trait are used to represent types with a free brand variable.
pub trait FreeBrand {
    type Bind<B: Brand>;
}

/// Given a type with a free brand variable represented via a [`FreeBrand`] instance, this creates a
/// type with a bound brand variable, which can later be substituted by a fresh brand variable.
#[repr(transparent)]
pub struct BoundBrand<F: FreeBrand>(F::Bind<Opaque>);

impl<F: FreeBrand> BoundBrand<F> {
    /// Substitues a fresh brand variable for the bound brand variable.
    pub fn substitute<B: Brand>(self, _fresh: FreshBrand<B>) -> F::Bind<B> {
        unsafe { std::mem::transmute_copy(&ManuallyDrop::new(self)) }
    }

    /// Binds a brand variable, abstracting away the specific brand used.
    pub fn bind<B: Brand>(value: F::Bind<B>) -> Self {
        unsafe { std::mem::transmute_copy(&ManuallyDrop::new(value)) }
    }
}

/// Marker type for a fresh brand variable that is guaranteed to be unique.
///
/// Often constructors for branded types as well as [`BoundBrand::substitute`] consume this.
///
/// An initial fresh brand can be obtained by implementing the [`WithFreshBrand`] trait.
pub struct FreshBrand<B: Brand>(PhantomData<B>);

impl<B: Brand> FreshBrand<B> {
    /// Produces two unique fresh brand variables by consuming a single fresh brand variable.
    #[inline(always)]
    pub fn split(self) -> (FreshBrand<impl Brand>, FreshBrand<impl Brand>) {
        (
            FreshBrand(PhantomData as PhantomData<Opaque>),
            FreshBrand(PhantomData as PhantomData<Opaque>),
        )
    }
}

impl<B: Brand> From<FreshBrand<B>> for PhantomData<B> {
    #[inline(always)]
    fn from(_: FreshBrand<B>) -> Self {
        PhantomData
    }
}

/// Trait to emulate a closure consuming a [`FreshBrand`] while being generic over the used brand variable.
///
/// To obtain an initial [`FreshBrand`], implement this trait for a custom type and invoke the
/// [`Self::run()`] method.
pub trait WithFreshBrand {
    /// The resulting output type.
    type Output;

    /// Consumes a [`FreshBrand`] producing some output.
    ///
    /// Implement this method for a custom type to obtain an initial [`FreshBrand`].
    fn with_fresh_brand(fresh_brand: FreshBrand<impl Brand>) -> Self::Output;

    /// Invoke [`Self::with_fresh_brand`] with a unique fresh brand variable.
    #[inline(always)]
    fn run() -> Self::Output {
        Self::with_fresh_brand(FreshBrand(PhantomData as PhantomData<Opaque>))
    }
}
