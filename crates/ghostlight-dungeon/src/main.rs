mod app_session;
mod eve;
mod heimdall;
mod idunn_health;
mod mesh;
mod local;
mod play;
mod runtime;

use anyhow::bail;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if let [mode, flag, path] = args.as_slice() {
        if mode == "--local" && flag == "--state-root" {
            return runtime::run_local(PathBuf::from(path)).await;
        }
    }
    let state_root = match args.as_slice() {
        [flag, path] if flag == "--state-root" => Some(PathBuf::from(path)),
        [] => None,
        _ => bail!("usage: ghostlight-dungeon [--local] --state-root PATH"),
    };
    runtime::run(state_root).await
}
