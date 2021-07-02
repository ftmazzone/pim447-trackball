use flume;
use std::error::Error;
use std::time::Duration;
use tokio::time::{sleep, timeout};

use pim447_trackball::pim447::{Command, Input, Trackball};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx_to_pim447, rx_to_pim447) = flume::unbounded::<Command>();
    let (tx_from_pim447, rx_from_pim447) = flume::unbounded::<Input>();

    let mut trackball: Trackball = Trackball::new()?;
    trackball.turn_on()?;
    trackball.set_contrast(0xff)?;
    trackball.set_colour(0xff, 0xff, 0xff, 0xff)?;
    println!("On");

    let task_read_inputs = tokio::spawn(async move {
        match read_inputs(trackball, tx_from_pim447, rx_to_pim447).await {
            Ok(_trackball) => (),
            _ => (),
        }
    });

    let _result = timeout(Duration::from_secs(30), get_inputs(tx_to_pim447.clone(),rx_from_pim447)).await;
    println!("Sending stop");
    tx_to_pim447.send(Command::TurnOff)?;
    task_read_inputs.await?;

    Ok(())
}

pub async fn get_inputs(
    tx: flume::Sender<Command>,
    rx: flume::Receiver<Input>,
) -> Result<(), Box<dyn Error>> {
    //If more than 5 results are received : change the coulour to red
    let mut i = 0;
    while !rx.is_disconnected() {
        let input = rx.recv_async().await?;
        println!("{:?}", input);

        if i > 5 {
            println!("Change colour to red");
            tx.send(Command::SetColour(0xff, 0, 0, 0))?;
        }

        i = i + 1;
    }
    Ok(())
}

pub async fn read_inputs(
    mut trackball: Trackball,
    tx: flume::Sender<Input>,
    rx: flume::Receiver<Command>,
) -> Result<Trackball, Box<dyn Error>> {
    let mut i: u16 = 0;

    let mut trackball_turn_off_command = false;

    while !trackball_turn_off_command {
        let input = trackball.read_input()?;
        if input.state_update {
            tx.send(input)?;
        }
        sleep(Duration::from_millis(50)).await;
        if !rx.is_empty() {
            match rx.recv() {
                Ok(Command::TurnOff) => {
                    trackball_turn_off_command = true;
                }
                Ok(Command::SetColour(r, g, b, w)) => {
                    trackball.set_colour(r, g, b, w)?;
                }
                Ok(_) => (),
                Err(_e) => (),
            }
        }

        i = i + 1;
    }
    trackball.turn_off()?;
    println!("Off");
    Ok(trackball)
}
