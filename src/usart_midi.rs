use embassy_stm32::{
    mode::Async,
    usart::{Uart, UartRx},
};
use midly::{
    live::{LiveEvent, SystemCommon},
    stream::MidiStream,
    MidiMessage,
};

#[embassy_executor::task]
pub async fn midi_task(usart: Uart<'static, Async>) {
    let (_tx, rx) = usart.split();
    rx_task(rx).await;
    // todo:
    // tx_task(tx).await;
}

pub async fn rx_task(mut rx: UartRx<'static, Async>) -> ! {
    // Create a MIDI stream to handle incoming MIDI messages
    let mut midi_stream = MidiStream::new();

    // Buffer to hold incoming bytes
    let mut buffer = [0u8; 256];

    loop {
        // Read bytes from the USART
        if let Err(e) = rx.read(&mut buffer).await {
            // Handle read error (e.g., log it, retry, etc.)
            defmt::error!("Failed to read from USART: {:?}", e);
            continue;
        }

        // Handle the MIDI data received
        handle_midi(&mut midi_stream, &buffer);
    }
}

fn handle_midi(stream: &mut MidiStream, new_bytes: &[u8]) {
    stream.feed(new_bytes, |event| {
        // `midly` will automatically parse boundaries and present
        // parsed, zero-copy MIDI events here
        match event {
            // you can get at regular midi messages
            LiveEvent::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel },
            } => {
                // Handle Note On event
                // For example, you could print the key and velocity
                defmt::info!(
                    "Note event: channel={}, key={}, vel={}",
                    channel.as_int(),
                    key.as_int(),
                    vel.as_int()
                );
            }
            // and even things like sysex
            LiveEvent::Common(SystemCommon::SysEx(sysex)) => {
                // ...
            }
            _ => {}
        }
    });
}

// todo: implement tx_task
// pub async fn tx_task(_tx: UartTx<'static, Async>) {}
