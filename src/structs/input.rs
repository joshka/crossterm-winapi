//! This module provides a few structs to wrap common input struts to a rusty interface
//!
//! Types like:
//! - `KEY_EVENT_RECORD`
//! - `MOUSE_EVENT_RECORD`
//! - `ControlKeyState`
//! - `ButtonState`
//! - `EventFlags`
//! - `InputEventType`
//! - `INPUT_RECORD`

use windows::Win32::System::Console::{
    FOCUS_EVENT, FOCUS_EVENT_RECORD, FROM_LEFT_1ST_BUTTON_PRESSED, FROM_LEFT_2ND_BUTTON_PRESSED,
    FROM_LEFT_3RD_BUTTON_PRESSED, FROM_LEFT_4TH_BUTTON_PRESSED, INPUT_RECORD, KEY_EVENT,
    KEY_EVENT_RECORD, MENU_EVENT, MENU_EVENT_RECORD, MOUSE_EVENT, MOUSE_EVENT_RECORD,
    RIGHTMOST_BUTTON_PRESSED, WINDOW_BUFFER_SIZE_EVENT, WINDOW_BUFFER_SIZE_RECORD,
};

use super::Coord;
use crate::ScreenBuffer;

/// A [keyboard input event](https://docs.microsoft.com/en-us/windows/console/key-event-record-str).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyEventRecord {
    /// If the key is pressed, this member is true. Otherwise, this member is
    /// false (the key is released).
    pub key_down: bool,
    /// The repeat count, which indicates that a key is being held down.
    /// For example, when a key is held down, you might get five events with
    /// this member equal to 1, one event with this member equal to 5, or
    /// multiple events with this member greater than or equal to 1.
    pub repeat_count: u16,
    /// A virtual-key code that identifies the given key in a
    /// device-independent manner.
    pub virtual_key_code: u16,
    /// The virtual scan code of the given key that represents the
    /// device-dependent value generated by the keyboard hardware.
    pub virtual_scan_code: u16,
    /// The translated Unicode character (as a WCHAR, or utf-16 value)
    pub u_char: u16,
    /// The state of the control keys.
    pub control_key_state: ControlKeyState,
}

impl KeyEventRecord {
    /// Convert a `KEY_EVENT_RECORD` to KeyEventRecord. This function is private
    /// because the `KEY_EVENT_RECORD` has several union fields for characters
    /// (u8 vs u16) that we always interpret as u16. We always use the wide
    /// versions of windows API calls to support this.
    #[inline]
    fn from_winapi(record: &KEY_EVENT_RECORD) -> Self {
        KeyEventRecord {
            key_down: record.bKeyDown.as_bool(),
            repeat_count: record.wRepeatCount,
            virtual_key_code: record.wVirtualKeyCode,
            virtual_scan_code: record.wVirtualScanCode,
            u_char: unsafe { record.uChar.UnicodeChar },
            control_key_state: ControlKeyState(record.dwControlKeyState),
        }
    }
}

/// A [mouse input event](https://docs.microsoft.com/en-us/windows/console/mouse-event-record-str).
#[derive(PartialEq, Debug, Copy, Clone, Eq)]
pub struct MouseEvent {
    /// The position of the mouse when the event occurred in cell coordinates.
    pub mouse_position: Coord,
    /// The state of the mouse's buttons.
    pub button_state: ButtonState,
    /// The state of the control keys.
    pub control_key_state: ControlKeyState,
    /// What type of mouse event it is.
    pub event_flags: EventFlags,
}

impl From<MOUSE_EVENT_RECORD> for MouseEvent {
    #[inline]
    fn from(event: MOUSE_EVENT_RECORD) -> Self {
        MouseEvent {
            mouse_position: event.dwMousePosition.into(),
            button_state: event.dwButtonState.into(),
            control_key_state: ControlKeyState(event.dwControlKeyState),
            event_flags: event.dwEventFlags.into(),
        }
    }
}

/// The status of the mouse buttons.
/// The least significant bit corresponds to the leftmost mouse button.
/// The next least significant bit corresponds to the rightmost mouse button.
/// The next bit indicates the next-to-leftmost mouse button.
/// The bits then correspond left to right to the mouse buttons.
/// A bit is 1 if the button was pressed.
///
/// The state can be one of the following:
///
/// ```
/// # enum __ {
/// Release = 0x0000,
/// /// The leftmost mouse button.
/// FromLeft1stButtonPressed = 0x0001,
/// /// The second button from the left.
/// FromLeft2ndButtonPressed = 0x0004,
/// /// The third button from the left.
/// FromLeft3rdButtonPressed = 0x0008,
/// /// The fourth button from the left.
/// FromLeft4thButtonPressed = 0x0010,
/// /// The rightmost mouse button.
/// RightmostButtonPressed = 0x0002,
/// /// This button state is not recognized.
/// Unknown = 0x0021,
/// /// The wheel was rotated backward, toward the user; this will only be activated for `MOUSE_WHEELED ` from `dwEventFlags`
/// Negative = 0x0020,
/// # }
/// ```
///
/// [Ms Docs](https://docs.microsoft.com/en-us/windows/console/mouse-event-record-str#members)
#[derive(PartialEq, Debug, Copy, Clone, Eq)]
pub struct ButtonState {
    state: i32,
}

impl From<u32> for ButtonState {
    #[inline]
    fn from(event: u32) -> Self {
        let state = event as i32;
        ButtonState { state }
    }
}

impl ButtonState {
    /// Get whether no buttons are being pressed.
    pub fn release_button(&self) -> bool {
        self.state == 0
    }

    /// Returns whether the left button was pressed.
    pub fn left_button(&self) -> bool {
        self.state as u32 & FROM_LEFT_1ST_BUTTON_PRESSED != 0
    }

    /// Returns whether the right button was pressed.
    pub fn right_button(&self) -> bool {
        self.state as u32
            & (RIGHTMOST_BUTTON_PRESSED
                | FROM_LEFT_3RD_BUTTON_PRESSED
                | FROM_LEFT_4TH_BUTTON_PRESSED)
            != 0
    }

    /// Returns whether the right button was pressed.
    pub fn middle_button(&self) -> bool {
        self.state as u32 & FROM_LEFT_2ND_BUTTON_PRESSED != 0
    }

    /// Returns whether there is a down scroll.
    pub fn scroll_down(&self) -> bool {
        self.state < 0
    }

    /// Returns whether there is a up scroll.
    pub fn scroll_up(&self) -> bool {
        self.state > 0
    }

    /// Returns the raw state.
    pub fn state(&self) -> i32 {
        self.state
    }
}

/// The state of the control keys.
///
/// This is a bitmask of the following values.
///
/// | Description | Value |
/// | --- | --- |
/// | The right alt key is pressed | `0x0001` |
/// | The left alt key is pressed | `x0002` |
/// | The right control key is pressed | `0x0004` |
/// | The left control key is pressed | `x0008` |
/// | The shift key is pressed | `0x0010` |
/// | The num lock light is on | `0x0020` |
/// | The scroll lock light is on | `0x0040` |
/// | The caps lock light is on | `0x0080` |
/// | The key is [enhanced](https://docs.microsoft.com/en-us/windows/console/key-event-record-str#remarks) | `0x0100` |
#[derive(PartialEq, Debug, Copy, Clone, Eq)]
pub struct ControlKeyState(u32);

impl ControlKeyState {
    /// Whether the control key has a state.
    pub fn has_state(&self, state: u32) -> bool {
        (state & self.0) != 0
    }
}

/// The type of mouse event.
/// If this value is zero, it indicates a mouse button being pressed or released.
/// Otherwise, this member is one of the following values.
///
/// [Ms Docs](https://docs.microsoft.com/en-us/windows/console/mouse-event-record-str#members)
#[derive(PartialEq, Debug, Copy, Clone, Eq)]
pub enum EventFlags {
    PressOrRelease = 0x0000,
    /// The second click (button press) of a double-click occurred. The first click is returned as a regular button-press event.
    DoubleClick = 0x0002,
    /// The horizontal mouse wheel was moved.
    MouseHwheeled = 0x0008,
    /// If the high word of the dwButtonState member contains a positive value, the wheel was rotated to the right. Otherwise, the wheel was rotated to the left.
    MouseMoved = 0x0001,
    /// A change in mouse position occurred.
    /// The vertical mouse wheel was moved, if the high word of the dwButtonState member contains a positive value, the wheel was rotated forward, away from the user.
    /// Otherwise, the wheel was rotated backward, toward the user.
    MouseWheeled = 0x0004,
    // This button state is not recognized.
    Unknown = 0x0021,
}

// TODO: Replace with TryFrom.
impl From<u32> for EventFlags {
    fn from(event: u32) -> Self {
        match event {
            0x0000 => EventFlags::PressOrRelease,
            0x0002 => EventFlags::DoubleClick,
            0x0008 => EventFlags::MouseHwheeled,
            0x0001 => EventFlags::MouseMoved,
            0x0004 => EventFlags::MouseWheeled,
            _ => EventFlags::Unknown,
        }
    }
}

/// The [size of console screen
/// buffer](https://docs.microsoft.com/en-us/windows/console/window-buffer-size-record-str).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowBufferSizeRecord {
    pub size: Coord,
}

impl From<WINDOW_BUFFER_SIZE_RECORD> for WindowBufferSizeRecord {
    #[inline]
    fn from(record: WINDOW_BUFFER_SIZE_RECORD) -> Self {
        WindowBufferSizeRecord {
            size: record.dwSize.into(),
        }
    }
}

/// A [focus event](https://docs.microsoft.com/en-us/windows/console/focus-event-record-str). This
/// is used only internally by Windows and should be ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusEventRecord {
    /// Reserved; do not use.
    pub set_focus: bool,
}

impl From<FOCUS_EVENT_RECORD> for FocusEventRecord {
    #[inline]
    fn from(record: FOCUS_EVENT_RECORD) -> Self {
        FocusEventRecord {
            set_focus: record.bSetFocus.as_bool(),
        }
    }
}

/// A [menu event](https://docs.microsoft.com/en-us/windows/console/menu-event-record-str). This is
/// used only internally by Windows and should be ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuEventRecord {
    /// Reserved; do not use.
    pub command_id: u32,
}

impl From<MENU_EVENT_RECORD> for MenuEventRecord {
    #[inline]
    fn from(record: MENU_EVENT_RECORD) -> Self {
        MenuEventRecord {
            command_id: record.dwCommandId,
        }
    }
}

/// An [input event](https://docs.microsoft.com/en-us/windows/console/input-record-str).
///
/// These records can be read from the input buffer by using the `ReadConsoleInput`
/// or `PeekConsoleInput` function, or written to the input buffer by using the
/// `WriteConsoleInput` function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputRecord {
    /// A keyboard event occurred.
    KeyEvent(KeyEventRecord),
    /// The mouse was moved or a mouse button was pressed.
    MouseEvent(MouseEvent),
    /// A console screen buffer was resized.
    WindowBufferSizeEvent(WindowBufferSizeRecord),
    /// A focus event occured. This is used only internally by Windows and should be ignored.
    FocusEvent(FocusEventRecord),
    /// A menu event occurred. This is used only internally by Windows and should be ignored.
    MenuEvent(MenuEventRecord),
}

impl From<INPUT_RECORD> for InputRecord {
    #[inline]
    fn from(record: INPUT_RECORD) -> Self {
        match record.EventType as u32 {
            KEY_EVENT => InputRecord::KeyEvent(KeyEventRecord::from_winapi(unsafe {
                &record.Event.KeyEvent
            })),
            MOUSE_EVENT => InputRecord::MouseEvent(unsafe { record.Event.MouseEvent }.into()),
            WINDOW_BUFFER_SIZE_EVENT => InputRecord::WindowBufferSizeEvent({
                let mut buffer =
                    unsafe { WindowBufferSizeRecord::from(record.Event.WindowBufferSizeEvent) };
                let window = ScreenBuffer::current().unwrap().info().unwrap();
                let screen_size = window.terminal_size();

                buffer.size.y = screen_size.height;
                buffer.size.x = screen_size.width;

                buffer
            }),
            FOCUS_EVENT => InputRecord::FocusEvent(unsafe { record.Event.FocusEvent }.into()),
            MENU_EVENT => InputRecord::MenuEvent(unsafe { record.Event.MenuEvent }.into()),
            code => panic!("Unexpected INPUT_RECORD EventType: {}", code),
        }
    }
}
