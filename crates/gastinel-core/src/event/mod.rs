//! Low-latency event queue and input types for compositor and shell.

use crate::geom::{Point, Rect, Size};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u8),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    // Pointer
    PointerMotion {
        pos: Point,
        delta: Point,
    },
    PointerButton {
        button: MouseButton,
        pressed: bool,
        pos: Point,
    },
    PointerScroll {
        dx: f32,
        dy: f32,
    },

    // Keyboard
    Key {
        code: u32,
        pressed: bool,
        modifiers: Modifiers,
    },

    // Window events
    WindowDamaged {
        window_id: u32,
        rect: Rect,
    },
    WindowResized {
        window_id: u32,
        size: Size,
    },
    WindowClosed {
        window_id: u32,
    },

    // Shell action shortcuts
    ShellToggleLauncher,
    ShellSwitchWorkspace(u32),
    ShellCloseFocusedWindow,
    ShellNextWindow,

    // Display
    OutputResized {
        width: u32,
        height: u32,
    },
}

/// Circular ring-buffer event queue for zero-allocation enqueueing.
pub struct EventQueue {
    buffer: Vec<Event>,
    head: usize,
    tail: usize,
    count: usize,
}

impl EventQueue {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, event: Event) {
        if self.count == self.buffer.capacity() {
            // Buffer full: drop oldest event to keep queue bounded and prevent OOM
            self.head = (self.head + 1) % self.buffer.capacity();
            self.count -= 1;
        }

        if self.buffer.len() < self.buffer.capacity() {
            self.buffer.push(event);
        } else {
            self.buffer[self.tail] = event;
        }
        self.tail = (self.tail + 1) % self.buffer.capacity();
        self.count += 1;
    }

    pub fn pop(&mut self) -> Option<Event> {
        if self.count == 0 {
            return None;
        }

        let event = self.buffer[self.head];
        self.head = (self.head + 1) % self.buffer.capacity();
        self.count -= 1;
        Some(event)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_queue_ring() {
        let mut queue = EventQueue::with_capacity(3);
        queue.push(Event::ShellToggleLauncher);
        queue.push(Event::ShellSwitchWorkspace(1));
        queue.push(Event::ShellSwitchWorkspace(2));

        assert_eq!(queue.len(), 3);
        // Overflow drops oldest
        queue.push(Event::ShellSwitchWorkspace(3));
        assert_eq!(queue.len(), 3);

        assert_eq!(queue.pop(), Some(Event::ShellSwitchWorkspace(1)));
        assert_eq!(queue.pop(), Some(Event::ShellSwitchWorkspace(2)));
        assert_eq!(queue.pop(), Some(Event::ShellSwitchWorkspace(3)));
        assert_eq!(queue.pop(), None);
    }
}
