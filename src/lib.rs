mod errors;
mod handlers;
mod providers;

use worker::*;

#[event(fetch)]
async fn fetch(req: Request, worker_env: Env, _worker_ctx: Context) -> Result<Response> {
    Router::new()
        .get_async(
            "/v1/price",
            |req, route_ctx| async move {
                handlers::price(&req, &route_ctx.env).await.or_else(|e| Ok(e.into_response()))
            },
        )
        .get_async(
            "/v1/dev/sync-fiat-rates",
            |_req, route_ctx| async move {
                match handlers::sync_fiat_rates(&route_ctx.env).await {
                    Err(e) => Ok(e.into_response()),
                    Ok(()) => Response::ok("Successfully updated rates.")
                }
            }
        )
        .run(req, worker_env)
        .await
}