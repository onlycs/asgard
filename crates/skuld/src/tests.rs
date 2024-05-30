#[test]
fn logger() {
    use crate::logger::SkuldLogger;
    use std::thread;

    SkuldLogger::new("log.txt".into())
        .unwrap()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    log::info!("Hello, world!");

    thread::spawn(move || {
        log::error!("Hello, world!");
    })
    .join()
    .unwrap();

    log::warn!("Hello, world!");

    async_std::task::block_on(async {
        log::debug!("Hello, world!");
    });

    panic!("This is a test panic!")
}
