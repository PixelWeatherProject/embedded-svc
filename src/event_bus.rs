use core::fmt::Debug;
use core::result::Result;
use core::time::Duration;

pub trait ErrorType {
    type Error: Debug;
}

impl<E> ErrorType for &E
where
    E: ErrorType,
{
    type Error = E::Error;
}

impl<E> ErrorType for &mut E
where
    E: ErrorType,
{
    type Error = E::Error;
}

pub trait Spin: ErrorType {
    fn spin(&mut self, duration: Option<Duration>) -> Result<(), Self::Error>;
}

pub trait Postbox<P>: ErrorType {
    fn post(&self, payload: &P, wait: Option<Duration>) -> Result<bool, Self::Error>;
}

impl<'a, P, PB> Postbox<P> for &'a mut PB
where
    PB: Postbox<P> + ErrorType,
{
    fn post(&self, payload: &P, wait: Option<Duration>) -> Result<bool, Self::Error> {
        (**self).post(payload, wait)
    }
}

impl<'a, P, PB> Postbox<P> for &'a PB
where
    PB: Postbox<P> + ErrorType,
{
    fn post(&self, payload: &P, wait: Option<Duration>) -> Result<bool, Self::Error> {
        (*self).post(payload, wait)
    }
}

pub trait EventBus<P>: ErrorType {
    type Subscription<'a>
    where
        Self: 'a;

    fn subscribe<'a, F>(&'a self, callback: F) -> Result<Self::Subscription<'a>, Self::Error>
    where
        F: FnMut(&P) + Send + 'a;
}

impl<'e, P, E> EventBus<P> for &'e E
where
    E: EventBus<P>,
{
    type Subscription<'a> = E::Subscription<'a> where Self: 'a;

    fn subscribe<'a, F>(&'a self, callback: F) -> Result<Self::Subscription<'a>, Self::Error>
    where
        F: FnMut(&P) + Send + 'a,
    {
        (**self).subscribe(callback)
    }
}

impl<'e, P, E> EventBus<P> for &'e mut E
where
    E: EventBus<P>,
{
    type Subscription<'a> = E::Subscription<'a> where Self: 'a;

    fn subscribe<'a, F>(&'a self, callback: F) -> Result<Self::Subscription<'a>, Self::Error>
    where
        F: FnMut(&P) + Send + 'a,
    {
        (**self).subscribe(callback)
    }
}

pub trait PostboxProvider<P>: ErrorType {
    type Postbox<'a>: Postbox<P, Error = Self::Error>
    where
        Self: 'a;

    fn postbox(&self) -> Result<Self::Postbox<'_>, Self::Error>;
}

impl<'p, P, PP> PostboxProvider<P> for &'p mut PP
where
    PP: PostboxProvider<P>,
{
    type Postbox<'a> = PP::Postbox<'a> where Self: 'a;

    fn postbox(&self) -> Result<Self::Postbox<'_>, Self::Error> {
        (**self).postbox()
    }
}

impl<'p, P, PP> PostboxProvider<P> for &'p PP
where
    PP: PostboxProvider<P>,
{
    type Postbox<'a> = PP::Postbox<'a> where Self: 'a;

    fn postbox(&self) -> Result<Self::Postbox<'_>, Self::Error> {
        (*self).postbox()
    }
}

#[cfg(feature = "nightly")]
pub mod asynch {
    pub use super::{ErrorType, Spin};

    pub trait Sender {
        type Data: Send;
        type Result: Send;

        async fn send(&self, value: Self::Data) -> Self::Result;
    }

    impl<S> Sender for &mut S
    where
        S: Sender,
    {
        type Data = S::Data;
        type Result = S::Result;

        async fn send(&self, value: Self::Data) -> Self::Result {
            (**self).send(value).await
        }
    }

    impl<S> Sender for &S
    where
        S: Sender,
    {
        type Data = S::Data;
        type Result = S::Result;

        async fn send(&self, value: Self::Data) -> Self::Result {
            (*self).send(value).await
        }
    }

    pub trait Receiver {
        type Result: Send;

        async fn recv(&self) -> Self::Result;
    }

    impl<R> Receiver for &mut R
    where
        R: Receiver,
    {
        type Result = R::Result;

        async fn recv(&self) -> Self::Result {
            (**self).recv().await
        }
    }

    impl<R> Receiver for &R
    where
        R: Receiver,
    {
        type Result = R::Result;

        async fn recv(&self) -> Self::Result {
            (*self).recv().await
        }
    }

    pub trait EventBus<P>: ErrorType {
        type Subscription<'a>: Receiver<Result = P>
        where
            Self: 'a;

        async fn subscribe(&self) -> Result<Self::Subscription<'_>, Self::Error>;
    }

    impl<E, P> EventBus<P> for &mut E
    where
        E: EventBus<P>,
    {
        type Subscription<'a> = E::Subscription<'a> where Self: 'a;

        async fn subscribe(&self) -> Result<Self::Subscription<'_>, Self::Error> {
            (**self).subscribe().await
        }
    }

    impl<E, P> EventBus<P> for &E
    where
        E: EventBus<P>,
    {
        type Subscription<'a> = E::Subscription<'a> where Self: 'a;

        async fn subscribe(&self) -> Result<Self::Subscription<'_>, Self::Error> {
            (**self).subscribe().await
        }
    }

    pub trait PostboxProvider<P>: ErrorType {
        type Postbox<'a>: Sender<Data = P>
        where
            Self: 'a;

        async fn postbox(&self) -> Result<Self::Postbox<'_>, Self::Error>;
    }

    impl<PB, P> PostboxProvider<P> for &mut PB
    where
        PB: PostboxProvider<P>,
    {
        type Postbox<'a> = PB::Postbox<'a> where Self: 'a;

        async fn postbox(&self) -> Result<Self::Postbox<'_>, Self::Error> {
            (**self).postbox().await
        }
    }

    impl<PB, P> PostboxProvider<P> for &PB
    where
        PB: PostboxProvider<P>,
    {
        type Postbox<'a> = PB::Postbox<'a> where Self: 'a;

        async fn postbox(&self) -> Result<Self::Postbox<'_>, Self::Error> {
            (**self).postbox().await
        }
    }
}
