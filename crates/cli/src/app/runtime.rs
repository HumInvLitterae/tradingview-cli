use tradingview_cdp::{self as transport, CdpClient, TransportConfig};
use tradingview_core::AppError;

pub async fn connect_runtime(config: &TransportConfig) -> Result<CdpClient, AppError> {
    let target = transport::discover_target(config).await?;
    CdpClient::connect(&target).await
}
