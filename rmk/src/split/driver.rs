///! The abstracted driver layer of the split keyboard.
///!
use crate::keyboard::{key_event_channel, KeyEvent};

use super::SplitMessage;
use defmt::{debug, error};

#[derive(Debug, Clone, Copy, defmt::Format)]
pub(crate) enum SplitDriverError {
    SerialError,
    EmptyMessage,
    DeserializeError,
    SerializeError,
    BleError(u8),
}

/// Split message reader from other split devices
pub(crate) trait SplitReader {
    async fn read(&mut self) -> Result<SplitMessage, SplitDriverError>;
}

/// Split message writer to other split devices
pub(crate) trait SplitWriter {
    async fn write(&mut self, message: &SplitMessage) -> Result<usize, SplitDriverError>;
}

/// PeripheralMatrixMonitor runs in central.
/// It reads split message from peripheral and updates key matrix cache of the peripheral.
///
/// When the central scans the matrix, the scanning thread sends sync signal and gets key state cache back.
///
/// The `ROW` and `COL` are the number of rows and columns of the corresponding peripheral's keyboard matrix.
/// The `ROW_OFFSET` and `COL_OFFSET` are the offset of the peripheral's matrix in the keyboard's matrix.
pub(crate) struct PeripheralMatrixMonitor<
    const ROW: usize,
    const COL: usize,
    const ROW_OFFSET: usize,
    const COL_OFFSET: usize,
    R: SplitReader,
> {
    /// Receiver
    receiver: R,
    /// Peripheral id
    id: usize,
}

impl<
        const ROW: usize,
        const COL: usize,
        const ROW_OFFSET: usize,
        const COL_OFFSET: usize,
        R: SplitReader,
    > PeripheralMatrixMonitor<ROW, COL, ROW_OFFSET, COL_OFFSET, R>
{
    pub(crate) fn new(receiver: R, id: usize) -> Self {
        Self { receiver, id }
    }

    /// Run the monitor.
    ///
    /// The monitor receives from the peripheral and forward the message to key_event_channel.
    pub(crate) async fn run(mut self) -> ! {
        loop {
            match self.receiver.read().await {
                Ok(received_message) => {
                    debug!("Received peripheral message: {}", received_message);
                    if let SplitMessage::Key(e) = received_message {
                        // Check row/col
                        if e.row as usize > ROW || e.col as usize > COL {
                            error!("Invalid peripheral row/col: {} {}", e.row, e.col);
                            continue;
                        }
                        key_event_channel
                            .send(KeyEvent {
                                row: e.row + ROW_OFFSET as u8,
                                col: e.col + COL_OFFSET as u8,
                                pressed: e.pressed,
                            })
                            .await;
                    }
                }
                Err(e) => error!("Peripheral message read error: {:?}", e),
            }
        }
    }
}
