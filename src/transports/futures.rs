use crate::channel::{AsyncTransport, Either, Msg, Transport};
use anyhow::anyhow;
use async_trait::async_trait;
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_lite::{
    future::{block_on, race},
    StreamExt,
};

#[derive(Clone, Debug, Copy)]
pub struct FuturesChannel;

impl Transport for FuturesChannel {
    type Rx = UnboundedReceiver<Msg>;
    type Tx = UnboundedSender<Msg>;

    fn make_channel(&self) -> (Self::Tx, Self::Rx) {
        unbounded()
    }

    fn recv(&self, rx: &mut Self::Rx) -> anyhow::Result<Msg> {
        block_on(rx.next()).ok_or_else(|| anyhow!("channel closed"))
    }

    fn send(&self, tx: &mut Self::Tx, value: Msg) -> anyhow::Result<()> {
        tx.unbounded_send(value)
            .map_err(|v| anyhow!("failed to send value {:?}", v))
    }

    fn choice(&self, rx1: &mut Self::Rx, rx2: &mut Self::Rx) -> anyhow::Result<Either> {
        block_on(race(async { rx1.next().await.map(Either::Left) }, async {
            rx2.next().await.map(Either::Right)
        }))
        .ok_or_else(|| anyhow!("channel closed"))
    }
}

#[async_trait]
impl AsyncTransport for FuturesChannel {
    async fn recv_async(&self, rx: &mut <Self as Transport>::Rx) -> anyhow::Result<Msg> {
        rx.next().await.ok_or_else(|| anyhow!("channel closed"))
    }

    async fn send_async(&self, tx: &mut <Self as Transport>::Tx, value: Msg) -> anyhow::Result<()> {
        tx.unbounded_send(value)
            .map_err(|v| anyhow!("failed to send value {:?}", v))
    }

    async fn choice_async(
        &self,
        rx1: &mut <Self as Transport>::Rx,
        rx2: &mut <Self as Transport>::Rx,
    ) -> anyhow::Result<Either> {
        race(async { rx1.next().await.map(Either::Left) }, async {
            rx2.next().await.map(Either::Right)
        })
        .await
        .ok_or_else(|| anyhow!("channel closed"))
    }
}
