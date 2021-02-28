use crate::{Rx, Tx};
use anyhow::{bail, Result};
use async_trait::async_trait;
use std::{any::Any, fmt::Debug, marker::PhantomData};

pub type Msg = Box<dyn Any + Send + 'static>;

pub trait Transport: Clone {
    type Rx;
    type Tx;

    /// Create a pair of channels, the first A->B, the second B->A
    fn make_channel(&self) -> (Self::Tx, Self::Rx);
    fn recv(&self, rx: &mut Self::Rx) -> Result<Msg>;
    fn send(&self, tx: &mut Self::Tx, value: Msg) -> Result<()>;
    fn choice(&self, rx1: &mut Self::Rx, rx2: &mut Self::Rx) -> Result<Either>;
}

#[async_trait]
pub trait AsyncTransport: Transport {
    async fn recv_async(&self, rx: &mut Self::Rx) -> Result<Msg>;
    async fn send_async(&self, tx: &mut Self::Tx, value: Msg) -> Result<()>;
    async fn choice_async(&self, rx1: &mut Self::Rx, rx2: &mut Self::Rx) -> Result<Either>;
}

pub enum Either {
    Left(Msg),
    Right(Msg),
    Both(Msg),
}

pub struct Channel<To, Tr: Transport> {
    pub(crate) tx: Tr::Tx,
    pub(crate) rx: Tr::Rx,
    pub(crate) tr: Tr,
    _ph: PhantomData<To>,
}

impl<To, Tr> Clone for Channel<To, Tr>
where
    Tr: Transport,
    Tr::Rx: Clone,
    Tr::Tx: Clone,
{
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: self.rx.clone(),
            tr: self.tr.clone(),
            _ph: PhantomData,
        }
    }
}

impl<To, Tr: Transport> Channel<To, Tr> {
    pub(crate) fn new(tr: Tr, tx: Tr::Tx, rx: Tr::Rx) -> Self {
        Self {
            tx,
            rx,
            tr,
            _ph: PhantomData,
        }
    }

    pub fn recv<T: Debug + 'static, Cont>(
        &mut self,
        protocol: Rx<To, T, Cont>,
    ) -> Result<(T, Cont)> {
        let value = self.tr.recv(&mut self.rx)?;
        let value = match value.downcast::<T>() {
            Ok(v) => v,
            Err(_v) => bail!("got unexpected message"),
        };
        Ok((*value, protocol.cont))
    }

    pub fn send<T: Send + 'static, Cont>(
        &mut self,
        protocol: Tx<To, T, Cont>,
        value: T,
    ) -> Result<Cont> {
        self.tr.send(&mut self.tx, Box::new(value))?;
        Ok(protocol.cont)
    }
}

impl<To, Tr: AsyncTransport> Channel<To, Tr> {
    pub async fn recv_async<T: Debug + 'static, Cont>(
        &mut self,
        protocol: Rx<To, T, Cont>,
    ) -> Result<(T, Cont)> {
        let value = self.tr.recv_async(&mut self.rx).await?;
        let value = match value.downcast::<T>() {
            Ok(v) => v,
            Err(_v) => bail!("got unexpected message"),
        };
        Ok((*value, protocol.cont))
    }

    pub async fn send_async<T: Send + 'static, Cont>(
        &mut self,
        protocol: Tx<To, T, Cont>,
        value: T,
    ) -> Result<Cont> {
        self.tr.send_async(&mut self.tx, Box::new(value)).await?;
        Ok(protocol.cont)
    }
}
