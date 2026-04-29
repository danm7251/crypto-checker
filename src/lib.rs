mod handlers;
mod providers;

use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async(
            "/v1/price",
            |req, ctx| async move { handlers::price(&req, &ctx.env).await },
        )
        .run(req, env)
        .await
}
