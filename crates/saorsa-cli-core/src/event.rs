//! Event and message system for the TUI framework
//!
//! This module provides the core messaging infrastructure for communication
//! between tabs, panes, and the application coordinator.

use crossterm::event::{KeyEvent, MouseEvent};
use tokio::sync::broadcast;

use crate::error::CoreError;
use crate::pane::{PaneId, Split};
use crate::tab::TabId;

/// Messages that can be sent through the TUI framework
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Message {
    // === Navigation ===
    /// Switch to a specific tab by ID
    SwitchTab(TabId),
    /// Close a specific tab by ID
    CloseTab(TabId),
    /// Switch to the next tab
    NextTab,
    /// Switch to the previous tab
    PrevTab,

    // === Pane Management ===
    /// Split the current pane
    SplitPane {
        /// Direction and ratio for the split
        direction: Split,
    },
    /// Close a specific pane
    ClosePane(PaneId),
    /// Focus a specific pane
    FocusPane(PaneId),
    /// Resize a pane by delta
    ResizePane {
        /// Pane to resize
        pane: PaneId,
        /// Delta to apply (positive = grow, negative = shrink)
        delta: i16,
    },

    // === Global ===
    /// Quit the application
    Quit,
    /// Toggle help display
    ToggleHelp,
    /// Open the command palette
    OpenCommandPalette,

    // === Input ===
    /// Keyboard input event
    Key(KeyEvent),
    /// Mouse input event
    Mouse(MouseEvent),
    /// Terminal resize event
    Resize(u16, u16),

    // === Custom ===
    /// Custom message for tabs/plugins
    Custom {
        /// Message type identifier
        kind: String,
        /// JSON payload
        payload: serde_json::Value,
    },

    // === Batch ===
    /// Multiple messages to process in sequence
    Batch(Vec<Message>),

    // === No-op ===
    /// No operation (used for optional returns)
    #[default]
    None,
}

impl Message {
    /// Creates a custom message with the given kind and payload
    ///
    /// # Arguments
    ///
    /// * `kind` - The message type identifier
    /// * `payload` - JSON value containing the message data
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_cli_core::event::Message;
    /// use serde_json::json;
    ///
    /// let msg = Message::custom("notification", json!({"text": "Hello"}));
    /// ```
    pub fn custom<S: Into<String>>(kind: S, payload: serde_json::Value) -> Self {
        Message::Custom {
            kind: kind.into(),
            payload,
        }
    }

    /// Creates a batch of messages
    ///
    /// # Arguments
    ///
    /// * `messages` - Vector of messages to batch together
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_cli_core::event::Message;
    ///
    /// let batch = Message::batch(vec![Message::NextTab, Message::ToggleHelp]);
    /// ```
    pub fn batch(messages: Vec<Message>) -> Self {
        Message::Batch(messages)
    }

    /// Returns true if this is a no-op message
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_cli_core::event::Message;
    ///
    /// assert!(Message::None.is_none());
    /// assert!(!Message::Quit.is_none());
    /// ```
    pub fn is_none(&self) -> bool {
        matches!(self, Message::None)
    }

    /// Flattens nested batch messages into a single level
    ///
    /// This method recursively flattens any nested `Message::Batch` variants
    /// and removes `Message::None` entries.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_cli_core::event::Message;
    ///
    /// let nested = Message::Batch(vec![
    ///     Message::Quit,
    ///     Message::Batch(vec![Message::NextTab, Message::PrevTab]),
    /// ]);
    /// let flat = nested.flatten();
    /// assert_eq!(flat.len(), 3);
    /// ```
    pub fn flatten(self) -> Vec<Message> {
        match self {
            Message::Batch(msgs) => msgs.into_iter().flat_map(|m| m.flatten()).collect(),
            Message::None => vec![],
            other => vec![other],
        }
    }
}

/// Input events from the terminal
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// Keyboard input
    Key(KeyEvent),
    /// Mouse input
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Tick event for periodic updates
    Tick,
}

impl From<crossterm::event::Event> for InputEvent {
    fn from(event: crossterm::event::Event) -> Self {
        match event {
            crossterm::event::Event::Key(key) => InputEvent::Key(key),
            crossterm::event::Event::Mouse(mouse) => InputEvent::Mouse(mouse),
            crossterm::event::Event::Resize(w, h) => InputEvent::Resize(w, h),
            _ => InputEvent::Tick, // Map other events to tick
        }
    }
}

/// Message bus for broadcasting messages to multiple subscribers
///
/// The `MessageBus` provides a publish-subscribe mechanism for distributing
/// messages throughout the TUI framework. It uses tokio's broadcast channel
/// internally for efficient multi-consumer message delivery.
///
/// # Example
///
/// ```
/// use saorsa_cli_core::event::{Message, MessageBus};
///
/// let bus = MessageBus::new(100);
/// let mut rx = bus.subscribe();
///
/// // Messages can be sent to all subscribers
/// // bus.send(Message::Quit).expect("send should succeed");
/// ```
#[derive(Debug)]
pub struct MessageBus {
    sender: broadcast::Sender<Message>,
}

impl MessageBus {
    /// Creates a new message bus with the specified capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of messages to buffer before older
    ///   messages are dropped for slow receivers
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_cli_core::event::MessageBus;
    /// let bus = MessageBus::new(100);
    /// ```
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        MessageBus { sender }
    }

    /// Subscribe to receive messages from this bus
    ///
    /// Returns a receiver that will receive all messages sent after
    /// the subscription is created.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_cli_core::event::MessageBus;
    /// let bus = MessageBus::new(100);
    /// let mut rx = bus.subscribe();
    /// assert_eq!(bus.subscriber_count(), 1);
    /// ```
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.sender.subscribe()
    }

    /// Send a message to all subscribers
    ///
    /// Returns the number of receivers that received the message.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::EventError` if there are no active subscribers.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_cli_core::event::{Message, MessageBus};
    ///
    /// let bus = MessageBus::new(100);
    /// let _rx = bus.subscribe();
    /// let count = bus.send(Message::Quit).expect("send should succeed");
    /// assert_eq!(count, 1);
    /// ```
    pub fn send(&self, msg: Message) -> Result<usize, CoreError> {
        self.sender
            .send(msg)
            .map_err(|e| CoreError::EventError(format!("failed to send message: {}", e)))
    }

    /// Returns the number of active subscribers
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_cli_core::event::MessageBus;
    ///
    /// let bus = MessageBus::new(100);
    /// assert_eq!(bus.subscriber_count(), 0);
    ///
    /// let _rx1 = bus.subscribe();
    /// assert_eq!(bus.subscriber_count(), 1);
    ///
    /// let _rx2 = bus.subscribe();
    /// assert_eq!(bus.subscriber_count(), 2);
    /// ```
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        MessageBus::new(256)
    }
}

impl Clone for MessageBus {
    fn clone(&self) -> Self {
        MessageBus {
            sender: self.sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_none_is_default() {
        assert!(matches!(Message::default(), Message::None));
    }

    #[test]
    fn test_message_is_none() {
        assert!(Message::None.is_none());
        assert!(!Message::Quit.is_none());
    }

    #[test]
    fn test_message_flatten_single() {
        let msg = Message::Quit;
        let flat = msg.flatten();
        assert_eq!(flat.len(), 1);
        assert!(matches!(flat[0], Message::Quit));
    }

    #[test]
    fn test_message_flatten_batch() {
        let msg = Message::Batch(vec![Message::Quit, Message::NextTab]);
        let flat = msg.flatten();
        assert_eq!(flat.len(), 2);
    }

    #[test]
    fn test_message_flatten_nested_batch() {
        let msg = Message::Batch(vec![
            Message::Quit,
            Message::Batch(vec![Message::NextTab, Message::PrevTab]),
        ]);
        let flat = msg.flatten();
        assert_eq!(flat.len(), 3);
    }

    #[test]
    fn test_message_flatten_none_removed() {
        let msg = Message::Batch(vec![Message::Quit, Message::None, Message::NextTab]);
        let flat = msg.flatten();
        assert_eq!(flat.len(), 2);
    }

    #[test]
    fn test_message_custom() {
        let msg = Message::custom("test", serde_json::json!({"key": "value"}));
        if let Message::Custom { kind, payload } = msg {
            assert_eq!(kind, "test");
            assert_eq!(payload["key"], "value");
        } else {
            panic!("Expected Custom message");
        }
    }

    #[test]
    fn test_message_batch_constructor() {
        let msg = Message::batch(vec![Message::Quit, Message::NextTab]);
        if let Message::Batch(msgs) = msg {
            assert_eq!(msgs.len(), 2);
        } else {
            panic!("Expected Batch message");
        }
    }

    #[test]
    fn test_input_event_from_key() {
        use crossterm::event::{KeyCode, KeyModifiers};
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let event = crossterm::event::Event::Key(key);
        let input: InputEvent = event.into();
        assert!(matches!(input, InputEvent::Key(_)));
    }

    #[test]
    fn test_input_event_from_resize() {
        let event = crossterm::event::Event::Resize(80, 24);
        let input: InputEvent = event.into();
        assert!(matches!(input, InputEvent::Resize(80, 24)));
    }

    #[test]
    fn test_input_event_from_paste() {
        let event = crossterm::event::Event::Paste("hello".to_string());
        let input: InputEvent = event.into();
        assert!(matches!(input, InputEvent::Tick));
    }

    #[test]
    fn test_message_bus_new() {
        let bus = MessageBus::new(100);
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_message_bus_default() {
        let bus = MessageBus::default();
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_message_bus_subscribe() {
        let bus = MessageBus::new(100);
        let _rx = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);
    }

    #[test]
    fn test_message_bus_multiple_subscribers() {
        let bus = MessageBus::new(100);
        let _rx1 = bus.subscribe();
        let _rx2 = bus.subscribe();
        let _rx3 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 3);
    }

    #[tokio::test]
    async fn test_message_bus_send_receive() {
        let bus = MessageBus::new(100);
        let mut rx = bus.subscribe();

        let count = bus.send(Message::Quit).expect("send should succeed");
        assert_eq!(count, 1);

        let received = rx.recv().await.expect("should receive message");
        assert!(matches!(received, Message::Quit));
    }

    #[tokio::test]
    async fn test_message_bus_broadcast_to_multiple() {
        let bus = MessageBus::new(100);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        let count = bus.send(Message::NextTab).expect("send should succeed");
        assert_eq!(count, 2);

        let msg1 = rx1.recv().await.expect("rx1 should receive");
        let msg2 = rx2.recv().await.expect("rx2 should receive");
        assert!(matches!(msg1, Message::NextTab));
        assert!(matches!(msg2, Message::NextTab));
    }

    #[test]
    fn test_message_bus_send_no_subscribers() {
        let bus = MessageBus::new(100);
        let result = bus.send(Message::Quit);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_bus_clone() {
        let bus1 = MessageBus::new(100);
        let _rx = bus1.subscribe();
        let bus2 = bus1.clone();

        // Both buses share the same channel
        assert_eq!(bus2.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn test_message_bus_clone_send() {
        let bus1 = MessageBus::new(100);
        let mut rx = bus1.subscribe();
        let bus2 = bus1.clone();

        // Send from cloned bus
        bus2.send(Message::ToggleHelp).expect("send should succeed");

        let received = rx.recv().await.expect("should receive");
        assert!(matches!(received, Message::ToggleHelp));
    }

    #[test]
    fn test_message_navigation_variants() {
        let switch = Message::SwitchTab(1);
        let close = Message::CloseTab(2);
        let next = Message::NextTab;
        let prev = Message::PrevTab;

        assert!(matches!(switch, Message::SwitchTab(1)));
        assert!(matches!(close, Message::CloseTab(2)));
        assert!(matches!(next, Message::NextTab));
        assert!(matches!(prev, Message::PrevTab));
    }

    #[test]
    fn test_message_pane_variants() {
        let split = Message::SplitPane {
            direction: Split::Horizontal(50),
        };
        let close = Message::ClosePane(1);
        let focus = Message::FocusPane(2);
        let resize = Message::ResizePane { pane: 3, delta: 10 };

        assert!(matches!(
            split,
            Message::SplitPane {
                direction: Split::Horizontal(50)
            }
        ));
        assert!(matches!(close, Message::ClosePane(1)));
        assert!(matches!(focus, Message::FocusPane(2)));
        assert!(matches!(resize, Message::ResizePane { pane: 3, delta: 10 }));
    }

    #[test]
    fn test_message_global_variants() {
        let quit = Message::Quit;
        let help = Message::ToggleHelp;
        let cmd = Message::OpenCommandPalette;

        assert!(matches!(quit, Message::Quit));
        assert!(matches!(help, Message::ToggleHelp));
        assert!(matches!(cmd, Message::OpenCommandPalette));
    }

    #[test]
    fn test_message_resize() {
        let msg = Message::Resize(120, 40);
        assert!(matches!(msg, Message::Resize(120, 40)));
    }

    #[test]
    fn test_input_event_equality() {
        use crossterm::event::{KeyCode, KeyModifiers};
        let key1 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let key2 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        let event1 = InputEvent::Key(key1);
        let event2 = InputEvent::Key(key2);

        assert_eq!(event1, event2);
    }

    #[test]
    fn test_input_event_tick() {
        let tick = InputEvent::Tick;
        assert!(matches!(tick, InputEvent::Tick));
    }
}
