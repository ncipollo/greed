# greed

Greed is a tool for automating trades 📈

| **Note:** Use at your own risk! This tool is in active development and may have bugs! |
|---------------------------------------------------------------------------------------|

# Investment Platforms

Currently, Greed supports the following investment platforms:

* [Alpaca](https://alpaca.markets/)

## Credentials

### Alpaca

To use Alpaca add the following variables to your environment:

```env
export APCA_API_KEY_ID='<live_key_id>'
export APCA_API_SECRET_KEY='<live_secret_key>'

# The below can be used for paper trading. Invoke greed with -s to use paper trading.
export SIMULATED_APCA_API_KEY_ID='<paper_key_id>'
export SIMULATED_APCA_API_SECRET_KEY='<paper_secret_key>'
```

# Running Greed 🚀

To run Greed in a simulated environment, you can use the following command:

```bash
greed run -s <path_to_config>
```

To run Greed in a live environment, you can use the following command:

```bash
greed run <path_to_config>
```

# Configuration

## Simple Configuration

Greed supports a simple CSV format for simple configurations. The following is an example configuration:

```csv
asset,amount,buy,sell,skip
$VTI ,50.0  ,5.0,1.0 ,false
SPY  ,25.0  ,1.0,    ,false
SKIP ,30.0  ,1.0,1.0 ,true
VEA  ,25.0  ,   ,2.0 ,
```

The following columns are supported:

* `asset`: The asset to trade
* `amount`: The amount of the asset buy, as a percentage of the total portfolio.
* `buy`: The percentage below the median price to buy the asset.
* `sell`: The percentage of gains to buy the asset.
* `skip`: If true, this row will be skipped. This can be useful if your sheet has rows which you don't intend to be part
  of the configuration.

## Advanced Configuration

For more advanced configurations, Greed supports a TOML configuration file. The following is an example configuration:
```toml
# The interval between each strategy run in seconds
interval = 300
# The platform to use for trading (alpaca)
platform = "alpaca"

[[strategies]]
# Each strategy can have an optional name
name = "ETF"

# Each stategy has for, when and do rules.
# In the following example we are buying $VTI when it is below the median price by 5% and selling when the gain is above 5%
[strategies.buy]
for = { stock = "$VTI" }
when = { below_median_percent = 5.0 }
do = { buy_percent = 10 }

# You can have as many strategies as you'd like in a single configuration.
[strategies.sell]
for = { stock = "$VTI" }
when = { gain_above_percent = 5.0 }
do = { sell_all = true }

[[strategies]]
name = "Chaos"

[strategies.buy]
for = { stock = "$UVXY" }
when = { below_median_percent = 2.0, median_period = "week" }
do = { buy_percent = 5 }

[strategies.sell]
for = { stock = "$UVXY" }
when = { gain_above_percent = 3.0 }
do = { sell_all = true }
```