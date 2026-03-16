mod handlers;

use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async(
            "/price",
            |req, _| async move { handlers::price(&req).await },
        )
        .run(req, env)
        .await
}
