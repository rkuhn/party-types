use anyhow::Result;
use futures_lite::future::block_on;
use party_types::{
    pair, prot, rec,
    transports::FuturesChannel,
    Choice,
    ChoiceResult::{One, Two},
    End, Rx, Tx,
};
use std::thread::spawn;

prot!(prot_q, Tx<A, u32, Rx<A, u32, Tx<A, String, End>>>);

rec!(
    RecA,
    Choice<Q, u32, Tx<B, u32, Choice<C, u32, Tx<Q, u32, RecA>, B, u32, Tx<Q, u32, RecA>>>, Q, String, End>
);
prot!(prot_a, RecA);

rec!(RecB, Rx<A, u32, (Tx<C, u32, RecB>, Tx<A, u32, RecB>)>);
prot!(prot_b, RecB);

rec!(RecC, Rx<B, u32, Tx<A, u32, RecC>>);
prot!(prot_c, RecC);

struct Q;
struct A;
struct B;
struct C;

fn main() -> Result<()> {
    let (mut ch_qa, mut ch_aq) = pair::<Q, A, _>(&FuturesChannel);
    let (mut ch_ab, mut ch_ba) = pair::<A, B, _>(&FuturesChannel);
    let (mut ch_ac, mut ch_ca) = pair::<A, C, _>(&FuturesChannel);
    let (mut ch_bc, mut ch_cb) = pair::<B, C, _>(&FuturesChannel);

    let thread_a = spawn(move || -> Result<End> {
        block_on(async {
            let mut prot = prot_a().rec();
            loop {
                match prot.recv_same_async(&mut ch_aq).await? {
                    One(value, cont) => {
                        let cont = cont.send_async(&mut ch_ab, value).await?;
                        let (value, cont) = match cont.recv_async(&mut ch_ac, &mut ch_ab).await? {
                            One(value, cont) => (value, cont),
                            Two(value, cont) => (value, cont),
                        };
                        let cont = cont.send_async(&mut ch_aq, value).await?;
                        prot = cont.rec();
                    }
                    Two(v, cont) => {
                        println!("process A got string {}", v);
                        return Ok(cont);
                    }
                }
            }
        })
    });

    let thread_b = spawn(move || -> Result<End> {
        block_on(async {
            let mut prot = prot_b().rec();
            loop {
                let (value, p) = prot.recv_async(&mut ch_ba).await?;
                if value > 100 {
                    let cont = p.0.send_async(&mut ch_bc, value).await?;
                    prot = cont.rec();
                } else {
                    let cont = p.1.send_async(&mut ch_ba, value).await?;
                    prot = cont.rec();
                }
            }
        })
    });

    let thread_c = spawn(move || -> Result<End> {
        block_on(async {
            let mut prot = prot_c().rec();
            loop {
                let (value, p) = prot.recv_async(&mut ch_cb).await?;
                prot = p.send_async(&mut ch_ca, value).await?.rec();
            }
        })
    });

    // use the current thread for role Q
    let prot = prot_q();
    let prot = prot.send(&mut ch_qa, 1)?;
    let (value, prot) = prot.recv(&mut ch_qa)?;
    println!("received {}", value);
    let _prot: End = prot.send(&mut ch_qa, "stop".to_string())?;

    // all threads end now because A shuts down, killing the channel to B (which then shuts down),
    // killing the channel to C (which then shuts down)
    println!("1 {:?}", thread_a.join().unwrap());
    println!("2 {:?}", thread_b.join().unwrap());
    println!("3 {:?}", thread_c.join().unwrap());

    Ok(())
}
