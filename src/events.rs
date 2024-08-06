use embassy_time::{Duration, Ticker};

#[embassy_executor::task]
async fn event_loop() -> ! {
    let mut ticker = Ticker::every(Duration::from_millis(50));
    loop {
        ticker.next().await;
    }
}
