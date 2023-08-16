#![allow(clippy::single_component_path_imports)]

#[cfg(feature = "dylink")]
#[allow(unused_imports)]
use dylink;

mod interlude {
    pub use deps::*;
    pub use tracing::{debug, error, info, trace, warn};
}

mod utils;

use crate::interlude::*;
use teloxide::prelude::*;

shadow_rs::shadow!(build);

fn main() {
    utils::setup_tracing().unwrap();
    #[cfg(feature = "dylink")]
    tracing::warn!("dylink enabled");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap_or_log()
        .block_on(start_bot())
        .unwrap_or_log();
}

async fn start_bot() -> eyre::Result<()> {
    let cx = Context {};
    let cx = std::sync::Arc::new(cx);

    let bot = Bot::from_env();
    bot.delete_webhook().await?;

    let handler = Update::filter_message().endpoint(default_handler);

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![cx])
        .enable_ctrlc_handler()
        .build();
    if let Ok(webhook_url) = utils::get_env_var("WEBHOOK_URL") {
        use teloxide::update_listeners::*;
        let socket_addr = (
            [0, 0, 0, 0],
            utils::get_env_var("PORT")
                .unwrap_or_else(|_| "8443".to_string())
                .parse()
                .map_err(|err| eyre::eyre!("error parsing env var PORT: {err}"))?,
        )
            .into();
        let (listener, stop_sig, router) = webhooks::axum_to_router(
            bot,
            webhooks::Options::new(
                socket_addr,
                webhook_url
                    .parse()
                    .map_err(|err| eyre::eyre!("error parsing WEBHOOK_URL: {err}"))?,
            ),
        )
        .await?;

        let server_task = tokio::spawn(async move {
            let router = router
                .merge(axum::Router::new().route(
                    "/up",
                    axum::routing::get(|| async {
                        axum::Json(serde_json::json! ({
                            "buildTime": build::BUILD_TIME,
                            "pkgVersion": build::PKG_VERSION,
                            "commitDate": build::COMMIT_DATE,
                            "commitHash": build::COMMIT_HASH,
                            "rustVersion": build::RUST_VERSION,
                            "rustChannel": build::RUST_CHANNEL,
                            "cargoVersion": build::CARGO_VERSION,
                        }))
                    }),
                ))
                .layer(
                    tower_http::trace::TraceLayer::new_for_http()
                        .on_response(
                            tower_http::trace::DefaultOnResponse::new()
                                .level(tracing::Level::INFO)
                                .latency_unit(tower_http::LatencyUnit::Micros),
                        )
                        .on_failure(
                            tower_http::trace::DefaultOnFailure::new()
                                .level(tracing::Level::ERROR)
                                .latency_unit(tower_http::LatencyUnit::Micros),
                        ),
                );
            info!("starting webhook listener at {socket_addr:?}");
            axum::Server::bind(&socket_addr)
                .serve(router.into_make_service())
                .with_graceful_shutdown(stop_sig)
                .await
        });
        dispatcher
            .dispatch_with_listener(
                listener,
                LoggingErrorHandler::with_custom_text("error at dispatch listener"),
            )
            .await;
        server_task.await??;
    }
    // start in long polling mode otherwise
    else {
        dispatcher.dispatch().await;
    }

    Ok(())
}

#[derive(Debug)]
struct Context {}

type SharedContext = std::sync::Arc<Context>;

async fn default_handler(bot: Bot, msg: Message, cx: SharedContext) -> eyre::Result<()> {
    bot.send_message(msg.chat.id, format!("ok!")).await?;
    Ok(())
}
