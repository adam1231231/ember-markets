use anchor_lang::prelude::*;

use crate::consts::ORDER_BOOK_SIZE;
use crate::ember_errors::EmberErr;
use crate::state::side::{Sides, StoredSide};

#[account(zero_copy)]
pub struct OrderBookState {
    pub buy_side: OrderBook,
    pub sell_side: OrderBook,
    pub market_key: Pubkey,
}

#[zero_copy]
pub struct OrderBook {
    pub side: StoredSide,
    pub best_order_idx: u64,
    pub worst_order_idx: u64,
    pub orders: [Order; ORDER_BOOK_SIZE],
}

#[zero_copy]
pub struct Order {
    pub price: u64,
    pub amount: u64,
    pub uid: u64,
    pub prev: u64,
    pub next: u64,
}

impl Default for Order {
    fn default() -> Self {
        Self {
            price: 0,
            amount: 0,
            uid: 0,
            prev: 0,
            next: 0,
        }
    }
}

impl OrderBook {
    pub fn insert_order(&mut self, amount: u64, price: u64, uid: u64) -> Result<()> {
        let mut order = Order::default();
        order.uid = uid;
        order.amount = amount;
        order.price = price;

        let mut prev_index: Option<u64> = Some(0);
        for i in 0..self.orders.len() {
            if self.is_price_better(price, self.orders[i].price) {
                let order_idx = self.get_empty_node().unwrap_or_else(|| {
                    let i = self.worst_order_idx;
                    self.remove_order(i);
                    i
                });

                order.prev = prev_index.unwrap_or(0);

                order.next = i as u64;

                self.place_order(order.clone(), order_idx);
            }
            prev_index = Some(i as u64);
        }
        // if order's price is the worst, insert it if there's an empty node
        let order_idx = self.get_empty_node().ok_or(EmberErr::OrderBookFull)?;
        order.prev = prev_index.unwrap_or(0);
        order.next = 0;
        self.place_order(order, order_idx);
        Ok(())
    }

    fn is_price_better(&self, lhs: u64, rhs: u64) -> bool {
        match self.side.into() {
            Sides::Buy => lhs > rhs,
            Sides::Sell => lhs < rhs,
        }
    }

    fn get_empty_node(&self) -> Option<u64> {
        for i in 0..self.orders.len() {
            if self.orders[i].uid == 0 {
                return Some(i as u64);
            }
        }
        None
    }

    fn place_order(&mut self, order: Order, i: u64) {
        assert_eq!(self.orders[i as usize].uid, 0);

        if order.prev == 0 {
            self.best_order_idx = i;
        } else {
            self.orders[order.prev as usize].prev = i;
        }

        if order.next == 0 {
            self.worst_order_idx = i;
        } else {
            self.orders[order.next as usize].prev = i;
        }
        self.orders[i as usize] = order;
    }

    fn remove_order(&mut self, i: u64) {
        let order: Order = self.orders[i as usize].clone();
        if order.prev == 0 {
            self.best_order_idx = order.next;
        } else {
            let to_remove_order = self.orders.get_mut(order.prev as usize).unwrap();
            to_remove_order.next = order.next;
        }

        if order.next == 0 {
            self.worst_order_idx = order.prev;
        } else {
            self.orders[order.next as usize].prev = order.prev;
        }
        self.orders[i as usize] = Order::default();
    }

    fn match_orders(&mut self, amount: u64) -> u64 {
        let mut filled_amount = 0;
        let mut total_cost = 0;
        let mut i = self.best_order_idx;
        while filled_amount < amount && i != 0 {
            let order = self.orders.get_mut(i as usize).unwrap();
            let amount_to_fill = std::cmp::min(order.amount, amount - filled_amount);
            order.amount -= amount_to_fill;
            total_cost += amount_to_fill * order.price;
            filled_amount += amount_to_fill;
            i = order.next;
            if order.amount == 0 {
                self.remove_order(i);
            }
        }
        total_cost
    }
}
