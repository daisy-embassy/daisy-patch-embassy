use embassy_stm32::{gpio, mode::Async, spi::Spi};
use embedded_hal_async::{
    delay::DelayNs,
    spi::{ErrorType, Operation, SpiBus, SpiDevice},
};
use grounded::uninit::GroundedArrayCell;

const SIZE: usize = 256;
//DMA buffer must be in special region. Refer https://embassy.dev/book/#_stm32_bdma_only_working_out_of_some_ram_regions
#[link_section = ".sram1_bss"]
static TX_BUFFER: GroundedArrayCell<u8, SIZE> = GroundedArrayCell::uninit();
#[link_section = ".sram1_bss"]
static RX_BUFFER: GroundedArrayCell<u8, SIZE> = GroundedArrayCell::uninit();

pub struct DmaSpi<'a> {
    spi: Spi<'a, Async>,
    cs: gpio::Output<'a>,
    tx_buffer: &'static mut [u8],
    rx_buffer: &'static mut [u8],
}

impl<'a> DmaSpi<'a> {
    pub fn new(spi: Spi<'a, Async>, cs: gpio::Output<'a>) -> Self {
        let rx_buffer: &mut [u8] = unsafe {
            RX_BUFFER.initialize_all_copied(0);
            let (ptr, len) = RX_BUFFER.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };
        let tx_buffer: &mut [u8] = unsafe {
            RX_BUFFER.initialize_all_copied(0);
            let (ptr, len) = TX_BUFFER.get_ptr_len();
            core::slice::from_raw_parts_mut(ptr, len)
        };

        Self {
            spi,
            cs,
            tx_buffer,
            rx_buffer,
        }
    }
}

impl ErrorType for DmaSpi<'_> {
    type Error = embassy_stm32::spi::Error;
}

impl SpiBus<u8> for DmaSpi<'_> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let len = words.len();
        let buf_slice = &mut self.tx_buffer[0..len];
        buf_slice.copy_from_slice(words);
        self.spi.write(buf_slice).await
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let len = words.len();
        let buf_slice = &mut self.rx_buffer[0..len];
        self.spi.read(buf_slice).await?;
        words.copy_from_slice(buf_slice);
        Ok(())
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        let rx_slice = {
            let len = read.len();
            &mut self.rx_buffer[0..len]
        };

        let tx_slice = {
            let len = write.len();
            let s = &mut self.tx_buffer[0..len];
            s.copy_from_slice(write);
            &mut self.tx_buffer[0..len]
        };
        self.spi.transfer(rx_slice, tx_slice).await?;
        read.copy_from_slice(rx_slice);
        Ok(())
    }

    // todo! I'm not sure if this is correct
    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let len = words.len();
        let buf_slice = &mut self.tx_buffer[0..len];
        self.spi.transfer_in_place(buf_slice).await?;
        words.copy_from_slice(buf_slice);
        Ok(())
    }
}

impl SpiDevice for DmaSpi<'_> {
    async fn transaction(
        &mut self,
        operations: &mut [embedded_hal_async::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        self.cs.set_low();

        let op_res = 'ops: {
            for op in operations {
                let res = match op {
                    Operation::Read(buf) => <Self as SpiBus>::read(self, buf).await,
                    Operation::Write(buf) => <Self as SpiBus>::write(self, buf).await,
                    Operation::Transfer(read, write) => {
                        <Self as SpiBus>::transfer(self, read, write).await
                    }
                    Operation::TransferInPlace(buf) => {
                        <Self as SpiBus>::transfer_in_place(self, buf).await
                    }
                    Operation::DelayNs(ns) => match self.flush().await {
                        Err(e) => Err(e),
                        Ok(()) => {
                            embassy_time::Delay {}.delay_ns(*ns).await;
                            Ok(())
                        }
                    },
                };
                if let Err(e) = res {
                    break 'ops Err(e);
                }
            }
            Ok(())
        };

        // On failure, it's important to still flush and deassert CS.
        let flush_res = self.flush().await;
        self.cs.set_high();

        op_res?;
        flush_res?;

        Ok(())
    }
}
