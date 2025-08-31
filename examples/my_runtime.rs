use sparrow::app::App;
use sparrow::runtime::{Executor, Task};
use sparrow::types::Response;
use std::net::TcpListener;
use std::sync::Arc; // 自作ランタイム

fn main() {
    let ex = Executor::new();

    // App の準備
    let mut app = App::new();
    app.at("/", |_req| {
        let resp = http::Response::builder()
            .status(200)
            .body("Hello from custom runtime!".into())
            .unwrap();
        async move { Ok(resp) }
    });

    let app = Arc::new(app);

    // 自作 Executor 上で App を動かすタスク
    let listen_task = Task::new(async move {
        let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
        println!("Listening on http://127.0.0.1:3000");

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let app = app.clone();
                std::thread::spawn(move || {
                    // TCP から読み込み、App のルーティングに投げる簡易化
                    // (本格的には HTTP パーサや Body 読み込みが必要)
                    println!("Accepted a connection!");
                });
            }
        }
    });

    ex.spawn(listen_task);
    ex.run();
}
