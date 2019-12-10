/*
 *
 * Copyright 2019 Gregory Lemercier, All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
 * either express or implied. See the License for the specific
 * language governing permissions and limitations under the License.
 *
 */

use crate::coss::{Client, OrderSide, OrderStatus, OrderType};
use serde::Deserialize;
use std::result::Result;

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct Configuration {
    pair: String,
    upper_limit: f32,
    lower_limit: f32,
    order_amount: f32,
    number_of_grids: u32,
}

pub(crate) struct Gridbot {
    config: Configuration,
    client: Client,
    order_ids: Vec<String>,
}

impl Gridbot {
    pub fn new(config: Configuration, client: Client) -> Gridbot {
        Gridbot {
            config: config,
            client: client,
            order_ids: vec![],
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        let coins: Vec<&str> = self.config.pair.split("_").collect();

        // Check config parameters
        if self.config.upper_limit < 0.0 || self.config.lower_limit < 0.0 {
            return Err("Limits cannot be negative values".to_string());
        }

        if self.config.upper_limit < self.config.lower_limit {
            return Err("Upper limit must be higher than lower limit".to_string());
        }

        // Get current balance for each coin of the pai
        let balances: Vec<f32> = coins
            .iter()
            .map(|coin| self.client.get_available_balance(coin))
            .collect();

        // Get current market price
        let current_price: f32 = self
            .client
            .get_market_price(self.config.pair.as_str())
            .unwrap();

        println!("Current price: {}", current_price);

        // Check limits
        if current_price < self.config.lower_limit || current_price > self.config.upper_limit {
            return Err(format!("The current price for this pair is {} and should fit within the lower/upper limits",
                current_price));
        }

        // Check balances are sufficient
        let order_step: f32 = (self.config.upper_limit - self.config.lower_limit)
            / self.config.number_of_grids as f32;
        let num_sell_orders: u32 = ((self.config.upper_limit - current_price) / order_step) as u32;
        let num_buy_orders: u32 = ((current_price - self.config.lower_limit) / order_step) as u32;

        let mut required_sell_coins: f32 = 0.0;
        let mut sell_orders: Vec<f32> = vec![];
        for i in 1..(num_sell_orders + 1) {
            let order = current_price + (i as f32 * order_step);
            required_sell_coins += self.config.order_amount;
            sell_orders.push(order);
        }

        let mut required_buy_coins: f32 = 0.0;
        let mut buy_orders: Vec<f32> = vec![];
        for i in 1..(num_buy_orders + 1) {
            let order = current_price - (i as f32 * order_step);
            required_buy_coins += self.config.order_amount * order;
            buy_orders.push(order);
        }

        if balances[0] < required_sell_coins {
            return Err(format!(
                "You need at least {} {} to start this bot (available: {})",
                required_sell_coins, coins[0], balances[0]
            ));
        }

        if balances[1] < required_buy_coins {
            return Err(format!(
                "You need at least {} {} to start this bot (available: {})",
                required_buy_coins, coins[1], balances[1]
            ));
        }

        println!("Balances:");
        println!("\t{} ETH", balances[0]);
        println!("\t{} USDT", balances[1]);

        // Place buy orders
        for order_price in buy_orders {
            let order = self
                .client
                .add_order(
                    self.config.pair.as_str(),
                    OrderType::LIMIT,
                    OrderSide::BUY,
                    self.config.order_amount,
                    order_price,
                )
                .expect(format!("Failed to place buy order at {}", order_price).as_str());
            println!("Placed buy order @ {} {}", order_price, coins[1]);
            self.order_ids.push(order.order_id);
        }

        // Place sell orders
        for order_price in sell_orders {
            let order = self
                .client
                .add_order(
                    self.config.pair.as_str(),
                    OrderType::LIMIT,
                    OrderSide::SELL,
                    self.config.order_amount,
                    order_price,
                )
                .expect(format!("Failed to place sell order at {}", order_price).as_str());
            println!("Placed sell order @ {} {}", order_price, coins[1]);
            self.order_ids.push(order.order_id);
        }

        Ok(())
    }

    pub fn process(&mut self) -> Result<(), String> {
        let mut to_remove: Vec<String> = vec![];

        for id in &self.order_ids {
            let order = self
                .client
                .get_order_details(id.as_str())
                .expect("Failed to get order details");

            match order.status {
                OrderStatus::filled => {
                    println!("Order @ {} was filled", order.order_price);
                    to_remove.push(id.clone());
                }
                OrderStatus::canceled => {
                    to_remove.push(id.clone());
                }
                _ => {
                    // Don't do anything
                }
            }
        }

        // Remove orders that no longer need monitoring
        self.order_ids.retain(|x| !to_remove.contains(x));

        Ok(())
    }
}
