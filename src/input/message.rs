use bounded_spsc_queue::Consumer;
use cgmath::*;
use game::*;

// Re-exports.
pub use glutin::MouseButton as CursorButton;
pub use glutin::VirtualKeyCode as Key;

// ////////////////////////////////////////////////////////
// Messages
// ////////////////////////////////////////////////////////

/// These are represented as an enumeration to preserve ordering when stored
/// in a vector and read sequentially.
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum InputFrame {
    // Represents keyboard events.
    KeyPressed(Key),
    KeyReleased(Key),

    // Represents cursor events.
    CursorPressed(CursorButton, Vector2<f32>),
    CursorReleased(CursorButton, Vector2<f32>),
    CursorLeft,
    CursorEntered,
}

// ////////////////////////////////////////////////////////
// Messenger
// ////////////////////////////////////////////////////////

pub struct InputConsumer {
    input_consumer: Consumer<InputFrame>,
}

impl InputConsumer {
    pub fn new(input_consumer: Consumer<InputFrame>) -> InputConsumer {
        InputConsumer { input_consumer: input_consumer }
    }

    pub fn tick<G: Game>(&mut self, game: &mut G) {
        // Frame processing
        loop {
            match self.input_consumer.try_pop() {
                Some(frame) => {
                    game.input(frame);
                },
                None => return,
            }
        }
    }
}