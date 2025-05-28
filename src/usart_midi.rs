use defmt::info;
use embassy_stm32::{
    mode::Async,
    usart::{Uart, UartRx},
};
use grounded::uninit::GroundedArrayCell;
use midly::{live::LiveEvent, stream::MidiStream, MidiMessage};

const SIZE: usize = 256;
//DMA buffer must be in special region. Refer https://embassy.dev/book/#_stm32_bdma_only_working_out_of_some_ram_regions
// #[link_section = ".sram1_bss"]
// static TX_BUFFER: GroundedArrayCell<u8, SIZE> = GroundedArrayCell::uninit();
#[link_section = ".sram1_bss"]
static RX_BUFFER: GroundedArrayCell<u8, SIZE> = GroundedArrayCell::uninit();

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

    let buffer: &mut [u8] = unsafe {
        RX_BUFFER.initialize_all_copied(0);
        let (ptr, len) = RX_BUFFER.get_ptr_len();
        core::slice::from_raw_parts_mut(ptr, len)
    };

    loop {
        // Read bytes from the USART
        if let Err(e) = rx.read(buffer).await {
            // Handle read error (e.g., log it, retry, etc.)
            defmt::error!("Failed to read from USART: {:?}", e);
            continue;
        }

        // Handle the MIDI data received
        handle_midi(&mut midi_stream, buffer);
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
            _ => info!("Unhandled MIDI event"),
        }
    });
}

// todo: implement tx_task
// pub async fn tx_task(_tx: UartTx<'static, Async>) {}
