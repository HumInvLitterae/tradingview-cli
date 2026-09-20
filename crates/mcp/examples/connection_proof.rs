//! Opt-in development harness; each account/network effect is an explicit operation.

//! It deliberately has no registration, credential access or network default.

use tradingview_mcp::{Failure, check_local_admission, check_local_sse};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--credential-worker"] {
        if tradingview_mcp::credential_worker().await.is_err() {
            std::process::exit(1);
        }
        return;
    }
    if args != ["--local-admission"] {
        use tradingview_mcp::{ProofOperation as Op, run_proof_with_worker};
        if !(args.len() == 2 || (args.len() == 4 && args[2] == "--credential-worker-path")) {
            eprintln!(
                "Usage: connection_proof --local-admission | \
                 discover/login/status/authorize-store/daily/weekly/monthly/all/refresh/logout \
                 <proof-directory> [--credential-worker-path <absolute-executable>]"
            );
            std::process::exit(1);
        }
        let operation = match args[0].as_str() {
            "discover" => Op::Discover,
            "login" => Op::Login,
            "status" => Op::Status,
            "authorize-store" => Op::AuthorizeStore,
            "all" => Op::ReadAll,
            "daily" => Op::ReadDaily,
            "weekly" => Op::ReadWeekly,
            "monthly" => Op::ReadMonthly,
            "refresh" => Op::Refresh,
            "logout" => Op::Logout,
            _ => {
                eprintln!("Unknown proof operation");
                std::process::exit(1);
            }
        };
        let worker = args.get(3).map(std::path::Path::new);
        match run_proof_with_worker(std::path::Path::new(&args[1]), operation, worker).await {
            Ok(report) => {
                println!("{report}");
                if report.get("success") == Some(&serde_json::json!(false)) {
                    std::process::exit(1);
                }
            }
            Err(error) => {
                eprintln!("{}", serde_json::json!({"success": false, "error": error}));
                std::process::exit(1);
            }
        }
        return;
    }
    let result = async {
        let root = tempfile::tempdir().map_err(|_| Failure::LocalState)?;
        check_local_admission(&root.path().join("state")).await?;
        check_local_sse().await
    }
    .await;
    match result {
        Ok(()) => println!(
            "{}",
            serde_json::json!({
                "local_admission": "passed",
                "local_sse_decoder": "passed",
                "provider_requests": 0,
                "credential_access": false,
                "connection_proof": "pending"
            })
        ),
        Err(error) => {
            eprintln!("{}", serde_json::json!({"error":error}));
            std::process::exit(1);
        }
    }
}
