mod channel;
mod protocol;

pub use channel::Channel;
pub use protocol::{Choice, ChoiceResult, End, Rx, Tx};

pub mod transports {
    mod crossbeam;
    pub use crossbeam::Crossbeam;

    mod futures;
    pub use futures::FuturesChannel;
}

pub fn pair<A, B, Tr: channel::Transport>(tr: &Tr) -> (Channel<B, Tr>, Channel<A, Tr>) {
    let (ta, ra) = tr.make_channel();
    let (tb, rb) = tr.make_channel();
    (
        Channel::new(tr.clone(), tb, ra),
        Channel::new(tr.clone(), ta, rb),
    )
}

#[macro_export]
macro_rules! rec {
    ($x:ident, $t:ty) => {
        #[derive(Default)]
        struct $x;
        impl $x {
            #[allow(clippy::type_complexity)]
            pub fn rec(self) -> $t {
                Default::default()
            }
        }
    };
}
#[macro_export]
macro_rules! prot {
    ($n:ident, $t:ty) => {
        #[allow(clippy::type_complexity)]
        fn $n() -> $t {
            Default::default()
        }
    };
}
