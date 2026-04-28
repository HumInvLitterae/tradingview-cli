use tradingview_core::AppError;

use crate::{
    cdp::CdpClient,
    transport::{self, TransportConfig},
};

pub async fn connect_runtime(config: &TransportConfig) -> Result<CdpClient, AppError> {
    let target = transport::discover_target(config).await?;
    CdpClient::connect(&target).await
}
