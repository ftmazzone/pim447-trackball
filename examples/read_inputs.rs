use flume;
use pim447_trackball::pim447::{Command, Input, Trackball};
use simple_signal::{self, Signal};
use std::{
    env, error::Error, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration
};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let (red, green, blue, white) = Trackball::convert_hex_colour_to_rgbw("#FFFFFF".to_string())?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |_signals| {
        log::info!("Received an interrupt signal");
        r.store(false, Ordering::SeqCst);
    });

    let (tx_to_pim447, rx_to_pim447) = flume::unbounded::<Command>();
    let (tx_from_pim447, rx_from_pim447) = flume::unbounded::<Input>();

    let mut trackball: Trackball = Trackball::new()?;
    trackball.execute_command(Command::TurnOn)?;
    trackball.execute_command(Command::SetContrast(0xff))?;
    trackball.execute_command(Command::SetColour(red, green, blue, white))?;
    log::info!("On");

    let task_read_inputs = tokio::spawn(async move {
        match read_inputs(trackball, tx_from_pim447, rx_to_pim447).await {
            Ok(_trackball) => (),
            _ => (),
        }
    });

    let _result = timeout(
        Duration::from_secs(120),
        get_inputs(running, tx_to_pim447.clone(), rx_from_pim447),
    )
    .await;
    log::info!("Sending stop");
    tx_to_pim447.send(Command::TurnOff)?;
    task_read_inputs.await?;

    Ok(())
}

pub async fn get_inputs(
    running: Arc<AtomicBool>,
    tx: flume::Sender<Command>,
    rx: flume::Receiver<Input>,
) -> Result<(), Box<dyn Error>> {
    //If more than 5 results are received : change the coulour to red
    let mut i: u32 = 0;
    while running.load(Ordering::SeqCst) && !rx.is_disconnected() {
        let result = timeout(Duration::from_secs(1), rx.recv_async()).await;
        match result {
            Ok(Ok(input)) => {
                log::info!("Input {:?}", input);
                i = i + 1;
            }
            Ok(Err(e)) => log::info!("get_inputs Error {}", e),
            Err(_e) => (),
        }

        if i == 5 {
            log::info!("Change colour to red");
            tx.send(Command::SetColour(0xff, 0, 0, 0))?;
        }
    }
    log::info!("Command 'TurnOff' sent");
    tx.send(Command::TurnOff)?;
    Ok(())
}

pub async fn read_inputs(
    mut trackball: Trackball,
    tx: flume::Sender<Input>,
    rx: flume::Receiver<Command>,
) -> Result<Trackball, Box<dyn Error>> {
    let mut trackball_turn_off_command = false;

    while !trackball_turn_off_command && !rx.is_disconnected() {
        let input = trackball.read_input()?;
        if input.state_update {
            tx.send(input)?;
        }
        let result = timeout(Duration::from_millis(100), rx.recv_async()).await;
        match result {
            Ok(Ok(Command::TurnOff)) => {
                trackball_turn_off_command = true;
            }
            Ok(Ok(command)) => {
                trackball.execute_command(command)?;
            }
            Ok(Err(e)) => log::info!("read_inputs Error {}", e),
            Err(_e) => (),
        }
    }
    trackball.execute_command(Command::TurnOff)?;
    log::info!("Off");
    Ok(trackball)
}
