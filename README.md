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
