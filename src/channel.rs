use std::{any::Any, fmt::Debug, marker::PhantomData};

use crate::{Rx, Tx};
use anyhow::{bail, Result};

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
            Err(v) => bail!("got unexpected message {:?}", v),
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
