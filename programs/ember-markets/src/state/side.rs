use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
#[repr(u8)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

// An enum in a form that is zero-copyable
#[zero_copy]
pub struct StoredSide {
    pub inner: u64,
}

impl From<Side> for StoredSide {
    fn from(side: Side) -> Self {
        Self { inner: side as u64 }
    }
}

impl From<StoredSide> for Side {
    fn from(stored_side: StoredSide) -> Self {
        match stored_side.inner {
            0 => Side::Buy,
            1 => Side::Sell,
            _ => unreachable!(),
        }
    }
}

impl From<StoredSide> for Sides {
    fn from(stored_side: StoredSide) -> Self {
        match stored_side.inner {
            0 => Sides::Buy,
            1 => Sides::Sell,
            _ => unreachable!(),
        }
    }
}

pub enum Sides {
    Buy,
    Sell,
}
