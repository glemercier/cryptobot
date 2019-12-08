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

use crate::coss::{Client, OrderSide, OrderType};
use serde::Deserialize;

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
}

impl Gridbot {
    pub fn new(config: Configuration, client: Client) -> Gridbot {
        Gridbot {
            config: config,
            client: client,
        }
    }

    pub fn initialize(&self) {
        let coins: Vec<&str> = self.config.pair.split("_").collect();

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
            eprintln!("The current price for this pair is {} and should fit within the lower/upper limits. Quitting.", current_price);
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
            eprintln!(
                "You need at least {} {} to start this bot (available: {})",
                required_sell_coins, coins[0], balances[0]
            );
        }

        if balances[1] < required_buy_coins {
            eprintln!(
                "You need at least {} {} to start this bot (available: {})",
                required_buy_coins, coins[1], balances[1]
            );
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
        }
    }
}
