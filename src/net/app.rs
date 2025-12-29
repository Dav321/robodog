use crate::peripheral::servo::{SERVO_SIGNAL, ServoTask};
use crate::{include_flash_bytes, include_flash_str};
use embassy_time::Duration;
use picoserve::response::{DebugValue, File, Redirect};
use picoserve::routing::{PathRouter, get, get_service, parse_path_segment};
use picoserve::{AppBuilder, AppRouter, Router, Server};

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

    Server::new(app, config, &mut http_buffer)
        .listen_and_serve(id, stack, port, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}

pub struct AppProps;

impl AppBuilder for AppProps {
    type PathRouter = impl PathRouter;

    fn build_app(self) -> Router<Self::PathRouter> {
        Router::new()
            .route("/", get(|| async move { Redirect::to("/index.html") }))
            .route(
                "/index.html",
                get_service(File::html(include_flash_str!("www/index.html"))),
            )
            .route(
                "/calibrate.html",
                get_service(File::html(include_flash_str!("www/calibrate.html"))),
            )
            .route(
                "/index.css",
                get_service(File::css(include_flash_str!("www/index.css"))),
            )
            .route(
                "/JetBrainsMono-Regular.woff2",
                get_service(File::with_content_type(
                    "font/woff2",
                    include_flash_bytes!("www/JetBrainsMono-Regular.woff2"),
                )),
            )
            .route(
                "/index.js",
                get_service(File::javascript(include_flash_str!("www/index.js"))),
            )
            .route(
                "/calibrate.js",
                get_service(File::javascript(include_flash_str!("www/calibrate.js"))),
            )
            .route(
                (
                    "/pos",
                    parse_path_segment(),
                    parse_path_segment(),
                    parse_path_segment(),
                ),
                get(|pos: (i16, i16, i16)| async move {
                    SERVO_SIGNAL.signal(ServoTask::MOVE(
                        pos.0 as f32 / 100.0,
                        pos.1 as f32 / 100.0,
                        pos.2 as f32 / 100.0,
                    ));
                    DebugValue(pos)
                }),
            )
            .route(
                ("/pwm", parse_path_segment(), parse_path_segment()),
                get(|data: (u8, u16)| async move {
                    let pwm = data.1 as f32 / 6666.66;
                    SERVO_SIGNAL.signal(ServoTask::CALIBRATION(data.0, pwm));
                    DebugValue(pwm)
                }),
            )
            .route(
                "/home",
                get(|| async move {
                    SERVO_SIGNAL.signal(ServoTask::HOME);
                    DebugValue("Home")
                }),
            )
    }
}
