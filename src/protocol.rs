use crate::{
    channel::{Either, Transport},
    Channel,
};
use anyhow::{bail, Result};
use std::{fmt::Debug, marker::PhantomData};

/// Receive a value T from the given Role
pub struct Tx<Role, T, Cont> {
    pub(crate) cont: Cont,
    _ph: PhantomData<(Role, T)>,
}
impl<Role, T: Send + 'static, Cont> Tx<Role, T, Cont> {
    pub fn send<Tr: Transport>(self, channel: &mut Channel<Role, Tr>, value: T) -> Result<Cont> {
        channel.send(self, value)
    }
}
impl<Role, T, Cont: Default> Default for Tx<Role, T, Cont> {
    fn default() -> Self {
        Self {
            cont: Cont::default(),
            _ph: PhantomData,
        }
    }
}

/// Send a value T to the given Role
pub struct Rx<Role, T, Cont> {
    pub(crate) cont: Cont,
    _ph: PhantomData<(Role, T)>,
}
impl<Role, T: Debug + 'static, Cont> Rx<Role, T, Cont> {
    pub fn recv<Tr: Transport>(self, channel: &mut Channel<Role, Tr>) -> Result<(T, Cont)> {
        channel.recv(self)
    }
}
impl<Role, T, Cont: Default> Default for Rx<Role, T, Cont> {
    fn default() -> Self {
        Self {
            cont: Cont::default(),
            _ph: PhantomData,
        }
    }
}

/// End this protocol. It is good practice to demand this result from the execution
/// to prove that the protocol ran in full.
#[derive(Default, Debug)]
pub struct End;

/// This value is received when the choice is resolved.
pub enum ChoiceResult<T1, C1, T2, C2> {
    One(T1, C1),
    Two(T2, C2),
}

/// This node allows receiving from either R1 or R2, depending on an external choice.
///
/// The choice is modeled as a tuple on the sender side. The `Other` is either another
/// `Choice` or `End`.
pub struct Choice<Role1, T1, Cont1, Role2, T2, Cont2> {
    one: Cont1,
    two: Cont2,
    _ph: PhantomData<(Role1, T1, Role2, T2)>,
}

impl<Role1, T1, Cont1: Default, Role2, T2, Cont2: Default> Default
    for Choice<Role1, T1, Cont1, Role2, T2, Cont2>
{
    fn default() -> Self {
        Self {
            one: Default::default(),
            two: Default::default(),
            _ph: PhantomData,
        }
    }
}

impl<Role1, T1: 'static, Cont1, Role2, T2: 'static, Cont2>
    Choice<Role1, T1, Cont1, Role2, T2, Cont2>
{
    pub fn recv<Tr: Transport>(
        self,
        c1: &mut Channel<Role1, Tr>,
        c2: &mut Channel<Role2, Tr>,
    ) -> Result<ChoiceResult<T1, Cont1, T2, Cont2>> {
        Ok(match c1.tr.choice(&mut c1.rx, &mut c2.rx)? {
            Either::Left(v) => match v.downcast::<T1>() {
                Ok(v) => ChoiceResult::One(*v, self.one),
                Err(v) => bail!("got unexpected message {:?}", v),
            },
            Either::Right(v) => match v.downcast::<T2>() {
                Ok(v) => ChoiceResult::Two(*v, self.two),
                Err(v) => bail!("got unexpected message {:?}", v),
            },
            Either::Both(v) => match v.downcast::<T1>() {
                Ok(v) => ChoiceResult::One(*v, self.one),
                Err(v) => match v.downcast::<T2>() {
                    Ok(v) => ChoiceResult::Two(*v, self.two),
                    Err(v) => bail!("got unexpected message {:?}", v),
                },
            },
        })
    }
}
