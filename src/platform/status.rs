use crate::error::GreedError;
use crate::platform::account::Account;
use crate::platform::order::Order;
use crate::platform::position::Position;
use crate::platform::FinancialPlatform;
use serde::Serialize;

/// A snapshot of the portfolio's current status: the account, and -- when
/// `full` is requested -- the open positions and orders. This is the shared,
/// side-effect-free core used by both `fetch_status` (CLI) and
/// `status_handler` (HTTP API), per the architecture note in CLAUDE.md that
/// CLI commands and HTTP endpoints should call into the same core library and
/// just render the result differently.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PortfolioStatus {
    pub account: Account,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<Vec<Position>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<Order>>,
}

/// Fetches the account info, and -- when `full` is true -- the open
/// positions and orders. Positions with a near-zero market value (absolute
/// value under a cent) are filtered out, mirroring the CLI's existing
/// display filter; since this is the single core function both the CLI and
/// the API call, that filter now applies to both rather than being
/// presentation-only.
///
/// When `full` is false, `positions`/`orders` are left as `None` (and
/// omitted from the JSON entirely, per `PortfolioStatus`'s
/// `skip_serializing_if`) instead of fetching and then discarding them, so
/// the server makes no unnecessary platform calls -- matching the CLI's
/// existing behavior of only fetching full status when asked.
pub async fn fetch(
    platform: &dyn FinancialPlatform,
    full: bool,
) -> Result<PortfolioStatus, GreedError> {
    let account = platform.account().await?;
    if !full {
        return Ok(PortfolioStatus {
            account,
            positions: None,
            orders: None,
        });
    }

    let positions = platform
        .positions()
        .await?
        .into_iter()
        .filter(|p| p.market_value.unwrap_or(0.0).abs() >= 0.01)
        .collect();
    let orders = platform.open_orders().await?;

    Ok(PortfolioStatus {
        account,
        positions: Some(positions),
        orders: Some(orders),
    })
}

#[cfg(test)]
mod test {
    use crate::asset::AssetSymbol;
    use crate::platform::account::Account;
    use crate::platform::order::Order;
    use crate::platform::position::Position;
    use crate::platform::status::{fetch, PortfolioStatus};
    use crate::platform::MockPlatform;

    #[tokio::test]
    async fn fetch_not_full_omits_positions_and_orders() {
        let account = Account::fixture();
        let platform = MockPlatform::new().with_account(account.clone());
        let status = fetch(&platform, false).await.expect("failed to fetch");
        assert_eq!(
            status,
            PortfolioStatus {
                account,
                positions: None,
                orders: None,
            }
        );
    }

    #[tokio::test]
    async fn fetch_full_includes_positions_and_orders() {
        let order = Order::fixture(AssetSymbol::new("vti"));
        let position = Position {
            market_value: Some(100.0),
            ..Position::fixture(AssetSymbol::new("vti"))
        };
        let platform = MockPlatform::new()
            .with_positions(vec![position.clone()])
            .with_open_orders(vec![order.clone()]);

        let status = fetch(&platform, true).await.expect("failed to fetch");

        assert_eq!(status.positions, Some(vec![position]));
        assert_eq!(status.orders, Some(vec![order]));
    }

    #[tokio::test]
    async fn fetch_full_filters_near_zero_market_value_positions() {
        let kept = Position {
            market_value: Some(1.0),
            ..Position::fixture(AssetSymbol::new("vti"))
        };
        let filtered = Position {
            market_value: Some(0.001),
            ..Position::fixture(AssetSymbol::new("voo"))
        };
        let platform = MockPlatform::new().with_positions(vec![kept.clone(), filtered]);

        let status = fetch(&platform, true).await.expect("failed to fetch");

        assert_eq!(status.positions, Some(vec![kept]));
    }

    #[test]
    fn serialize_without_positions_or_orders_omits_fields() {
        let status = PortfolioStatus {
            account: Account::fixture(),
            positions: None,
            orders: None,
        };

        let json = serde_json::to_value(&status).expect("failed to serialize");
        assert!(json.get("account").is_some());
        assert!(json.get("positions").is_none());
        assert!(json.get("orders").is_none());
    }

    #[test]
    fn serialize_with_positions_and_orders_includes_fields() {
        let status = PortfolioStatus {
            account: Account::fixture(),
            positions: Some(vec![Position::fixture(AssetSymbol::new("vti"))]),
            orders: Some(vec![Order::fixture(AssetSymbol::new("vti"))]),
        };

        let json = serde_json::to_value(&status).expect("failed to serialize");
        assert!(json["positions"].is_array());
        assert!(json["orders"].is_array());
    }
}
