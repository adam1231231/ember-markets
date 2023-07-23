use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
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
            0 => Side::Bid,
            1 => Side::Ask,
            _ => unreachable!(),
        }
    }
}

impl From<StoredSide> for Sides {
    fn from(stored_side: StoredSide) -> Self {
        match stored_side.inner {
            0 => Sides::Bid,
            1 => Sides::Ask,
            _ => unreachable!(),
        }
    }
}

pub enum Sides {
    Bid,
    Ask,
}
