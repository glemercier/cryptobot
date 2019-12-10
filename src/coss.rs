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

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::SystemTime;

static COSS_API_BASE_URL: &str = "https://trade.coss.io";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct Credentials {
    pub public_key: String,
    pub secret_key: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct Asset {
    pub currency_code: Option<String>,
    pub address: Option<String>,
    pub total: String,
    pub available: String,
    pub in_order: String,
    pub memo: Option<String>,
    pub memoLabel: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Price {
    pub symbol: String,
    pub price: String,
    pub updated_time: u64,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum OrderStatus {
    open,
    canceled,
    filled,
    partial_fill,
    cancelling,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OrderResponse {
    pub order_id: String,
    pub account_id: String,
    pub order_symbol: String,
    pub order_side: String,
    pub status: OrderStatus,
    pub createTime: u64,
    pub r#type: String,
    pub order_price: String,
    pub order_size: String,
    pub executed: String,
    pub stop_price: String,
    pub avg: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OrderAddResponse {
    pub hex_id: String,
    pub order_id: String,
    pub account_id: String,
    pub order_symbol: String,
    pub order_side: String,
    pub status: String,
    pub createTime: u64,
    pub r#type: String,
    pub timeMatching: u64,
    pub order_price: String,
    pub order_size: String,
    pub executed: String,
    pub stop_price: String,
    pub avg: String,
    pub total: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CancelOrderResponse {
    pub order_id: String,
    pub order_symbol: String,
}

enum HttpRequest {
    GET,
    POST,
    DELETE,
}

pub(crate) enum OrderSide {
    BUY,
    SELL,
}

pub(crate) enum OrderType {
    MARKET,
    LIMIT,
}

fn get_timestamp() -> String {
    format!(
        "{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}

fn get_url(suffix: &str) -> String {
    COSS_API_BASE_URL.to_string() + suffix
}

pub(crate) struct Client {
    credentials: Credentials,
}

impl Client {
    pub fn new(creds: Credentials) -> Client {
        Client { credentials: creds }
    }

    fn api_request(
        &self,
        req: HttpRequest,
        url: String,
        to_sign: String,
        mut params: Vec<(String, String)>,
    ) -> Result<String, reqwest::Error> {
        let mut mac = Hmac::<Sha256>::new_varkey(self.credentials.secret_key.as_bytes()).unwrap();
        mac.input(to_sign.as_bytes());

        let sig: Vec<String> = mac
            .result()
            .code()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        params.push(("timestamp".to_string(), format!("{}", get_timestamp())));

        match req {
            HttpRequest::GET => Ok(reqwest::Client::new()
                .get(url.as_str())
                .header("Content-Type", "application/json")
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Authorization", self.credentials.public_key.clone())
                .header("Signature", sig.concat())
                .query(&params)
                .send()?
                .text()?),
            HttpRequest::POST => Ok(reqwest::Client::new()
                .post(url.as_str())
                .header("Content-Type", "application/json")
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Authorization", self.credentials.public_key.clone())
                .header("Signature", sig.concat())
                .query(&params)
                .body(to_sign)
                .send()?
                .text()?),
            HttpRequest::DELETE => Ok(reqwest::Client::new()
                .delete(url.as_str())
                .header("Content-Type", "application/json")
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Authorization", self.credentials.public_key.clone())
                .header("Signature", sig.concat())
                .query(&params)
                .body(to_sign)
                .send()?
                .text()?),
        }
    }

    pub fn get_balances(&self) -> Result<Vec<Asset>, serde_json::error::Error> {
        let to_sign = format!("timestamp={}", get_timestamp());
        let balances: Vec<Asset> = serde_json::from_str(
            self.api_request(
                HttpRequest::GET,
                get_url("/c/api/v1/account/balances"),
                to_sign,
                vec![],
            )
            .unwrap()
            .as_str(),
        )?;

        Ok(balances)
    }

    pub fn get_balance(&self, coin: &str) -> Result<Asset, serde_json::error::Error> {
        let balances = self.get_balances().unwrap();
        let asset: Asset = match balances.into_iter().find(|b| match &b.currency_code {
            Some(a) => a == coin,
            None => false,
        }) {
            Some(a) => a,
            None => Asset::default(),
        };

        Ok(asset)
    }

    pub fn get_available_balance(&self, coin: &str) -> f32 {
        match self.get_balance(coin) {
            Ok(asset) => match asset.available.parse::<f32>() {
                Ok(balance) => balance,
                Err(_) => 0.0,
            },
            Err(_) => 0.0,
        }
    }

    pub fn get_market_price(&self, pair: &str) -> Result<f32, serde_json::error::Error> {
        let to_sign = format!("timestamp={}", get_timestamp());
        let params: Vec<(String, String)> = vec![("symbol".to_string(), pair.to_string())];
        let price: Vec<Price> = serde_json::from_str(
            self.api_request(
                HttpRequest::GET,
                get_url("/c/api/v1/market-price"),
                to_sign,
                params,
            )
            .unwrap()
            .as_str(),
        )?;

        Ok(price[0].price.parse::<f32>().unwrap())
    }

    pub fn get_orders(&self, pair: &str) -> Result<Vec<OrderResponse>, serde_json::error::Error> {
        let to_sign: String = format!(
            "
        {{
            \"symbol\": \"{}\"
            \"from_id\": null,
            \"limit\": 50,
            \"recvWindow\": 5000,
            \"timestamp\": \"{}\"
        }}",
            pair.to_string(),
            get_timestamp()
        );

        let orders: Vec<OrderResponse> = serde_json::from_str(
            self.api_request(
                HttpRequest::POST,
                get_url("/c/api/v1/order/list/all"),
                to_sign,
                vec![],
            )
            .unwrap()
            .as_str(),
        )?;

        Ok(orders)
    }

    pub fn get_order_details(
        &self,
        order_id: &str,
    ) -> Result<OrderResponse, serde_json::error::Error> {
        let to_sign: String = format!(
            "
        {{
            \"order_id\": \"{}\",
            \"timestamp\": \"{}\"
        }}",
            order_id.to_string(),
            get_timestamp()
        );

        println!("{}", to_sign);

        let orders: OrderResponse = serde_json::from_str(
            self.api_request(
                HttpRequest::POST,
                get_url("/c/api/v1/order/details"),
                to_sign,
                vec![],
            )
            .unwrap()
            .as_str(),
        )?;

        Ok(orders)
    }

    pub fn add_order(
        &self,
        pair: &str,
        r#type: OrderType,
        side: OrderSide,
        size: f32,
        price: f32,
    ) -> Result<OrderAddResponse, serde_json::error::Error> {
        let to_sign: String = format!(
            "
        {{
            \"order_symbol\": \"{}\",
            \"order_side\": \"{}\",
            \"type\": \"{}\",
            \"order_size\": {:.3},
            \"order_price\": {:.3},
            \"timestamp\": {}
        }}",
            pair.to_string(),
            match side {
                OrderSide::BUY => "BUY",
                _ => "SELL",
            },
            match r#type {
                OrderType::MARKET => "market",
                _ => "limit",
            },
            size,
            price,
            get_timestamp()
        );

        let resp = self
            .api_request(
                HttpRequest::POST,
                get_url("/c/api/v1/order/add/"),
                to_sign,
                vec![],
            )
            .unwrap();

        println!("{}", resp);

        let response: OrderAddResponse = serde_json::from_str(resp.as_str())?;

        Ok(response)
    }

    pub fn cancel_order(
        &self,
        pair: &str,
        id: &str,
    ) -> Result<CancelOrderResponse, serde_json::error::Error> {
        let to_sign: String = format!(
            "
        {{
            \"order_symbol\": \"{}\",
            \"order_id\": \"{}\",
            \"timestamp\": {}
        }}",
            pair.to_string(),
            id,
            get_timestamp()
        );

        let response: CancelOrderResponse = serde_json::from_str(
            self.api_request(
                HttpRequest::DELETE,
                get_url("/c/api/v1/order/cancel"),
                to_sign,
                vec![],
            )
            .unwrap()
            .as_str(),
        )?;

        Ok(response)
    }
}
