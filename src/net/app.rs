use crate::peripheral::servo::{SERVO_CALIBRATION_SIGNAL, SERVO_SIGNAL};
use embassy_time::Duration;
use picoserve::response::{DebugValue, File, Redirect};
use picoserve::routing::{PathRouter, get, get_service, parse_path_segment};
use picoserve::{AppBuilder, AppRouter, Router};

pub const WEB_TASK_POOL_SIZE: usize = 8;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn app_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::listen_and_serve(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
    )
    .await
}

pub struct AppProps;

impl AppBuilder for AppProps {
    type PathRouter = impl PathRouter;

    fn build_app(self) -> Router<Self::PathRouter> {
        Router::new()
            .route("/", get(|| Redirect::to("/index.html")))
            .route(
                "/index.html",
                get_service(File::html(include_str!("www/index.html"))),
            )
            .route(
                "/calibrate.html",
                get_service(File::html(include_str!("www/calibrate.html"))),
            )
            .route(
                "/index.css",
                get_service(File::css(include_str!("www/index.css"))),
            )
            .route(
                "/JetBrainsMono-Regular.woff2",
                get_service(File::with_content_type(
                    "font/woff2",
                    include_bytes!("www/JetBrainsMono-Regular.woff2"),
                )),
            )
            .route(
                "/index.js",
                get_service(File::javascript(include_str!("www/index.js"))),
            )
            .route(
                "/calibrate.js",
                get_service(File::javascript(include_str!("www/calibrate.js"))),
            )
            .route(
                ("/pos", parse_path_segment(), parse_path_segment()),
                get(|pos: (u16, u16)| async move {
                    SERVO_SIGNAL.signal(pos);
                    DebugValue(pos)
                }),
            )
            .route(
                ("/pwm", parse_path_segment()),
                get(|pwm: u16| async move {
                    SERVO_CALIBRATION_SIGNAL.signal(pwm);
                    DebugValue(pwm)
                }),
            )
    }
}
