[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
# Cryptocurrency Trading bot

This repository contains a trading bot written in Rust that implements automatic trading on the [https://www.coss.io/](coss.io) platform.

It currently supports the following trading mechanisms:
 * Grid trading

As it is written in Rust, a multi-platform language, it can be run on any machine (Windows, Linux, macOS, etc...).

This bot has originally written to participate to the COSS contest. My handle on Telegram is __@greglem__.

__WARNING: This bot is provided AS-IS and must only be used by experienced cryptocurrency traders. The developer cannot be held responsible for any financial loss any user of this bot may encounter.__

# Installation instructions

## Installing rust

The best way to install Rust on your machine is to follow the quick installation instructions on [rustup.rs](https://rustup.rs/).

## Build the sources

First download the sources of the bot using git:

```bash
git clone https://github.com/glemercier/cryptobot.git
```

Run the build:

```bash
cargo build --release
```

The executable program will be generated under: __target/release/cryptobot__

# Trading bot configuration

The configuration of the bot is done using a configuration file called __config.conf__. It can be edited using any text editor and should contain the following information:

```
credentials {
  public_key = "<public key of the COSS account>"
  secret_key = "<secret key of the COSS account>"
}

gridbot {
  pair = "<name of the pair to trade>"
  upper_limit = <upper limit of the grids>
  lower_limit = <lower limit of the grids>
  order_amount = <amount to buy/sell for each grid>
  number_of_grids = <number of grids to create between set limits>
}
```

The COSS API credentials (public/secret keys) can be generated on the COSS platform under __Account -> API Management__. The gridbot parameters need to be adjusted to match the expected trading strategy. Following is a simple example on the ETH/USDT pair:

```
gridbot {
  pair = "ETH_USDT"
  upper_limit = 160
  lower_limit = 140
  order_amount = 0.072
  number_of_grids = 20
}
```

# Launching the bot

When launched, the bot will look for the __config.conf__ file in the current directory. Make sure this file is valid and present, then run the bot:

```bash
target/release/cryptobot
```

The bot is not interactive and automatically place the orders by monitoring the target price in real-time. It displays everything it does in the console, and reports any error the same way. If you need to stop the bot, simply press __Ctrl + C__. Note that it will not cancel any orders that have been placed during the trading process, make sure to check and close the orders manually if needed using the COSS platform.
