use crate::channel::{Either, Msg, Transport};
use anyhow::anyhow;
use crossbeam_channel::{select, unbounded, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Crossbeam;

impl Transport for Crossbeam {
    type Rx = Receiver<Msg>;
    type Tx = Sender<Msg>;

    fn make_channel(&self) -> (Self::Tx, Self::Rx) {
        unbounded()
    }

    fn recv(&self, rx: &mut Self::Rx) -> anyhow::Result<Box<dyn std::any::Any + Send + 'static>> {
        Ok(rx.recv()?)
    }

    fn send(
        &self,
        tx: &mut Self::Tx,
        value: Box<dyn std::any::Any + Send + 'static>,
    ) -> anyhow::Result<()> {
        Ok(tx
            .send(value)
            .map_err(|e| anyhow!("cannot send {:?}", e.0))?)
    }

    fn choice(&self, rx1: &mut Self::Rx, rx2: &mut Self::Rx) -> anyhow::Result<Either> {
        if rx1.same_channel(rx2) {
            return Ok(Either::Both(rx1.recv()?));
        }
        select! {
            recv(rx1) -> msg => Ok(Either::Left(msg?)),
            recv(rx2) -> msg => Ok(Either::Right(msg?)),
        }
    }
}
