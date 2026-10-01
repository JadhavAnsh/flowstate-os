# Persistent voice transcription widget

The notch widget represents the current voice turn. Holding Control+Option
reveals it and starts capture. Partial and final transcription appear in the
widget. Releasing the keys ends capture, but the widget stays visible during
generation and until the last spoken sentence finishes. It then collapses and
clears its transient transcript, response, error, and expansion state.

Conversation messages remain available in the dashboard. The widget does not
retain a copy of the event history. Events and completion callbacks from an
interrupted turn must not update or dismiss a newer turn.

## Acceptance checks

- A held shortcut is delivered after the widget listeners are ready.
- Releasing the shortcut and moving the pointer away cannot conceal an active turn.
- Partial and final input text render in the widget.
- Model completion does not collapse the widget before native speech completion.
- Completion of the last utterance automatically collapses the widget.
- Reopening starts with an empty, compact widget; late events stay ignored.
- Cancelling or starting a new turn invalidates old playback callbacks.
- No-input, capture failure, permission handoff, and model failure release busy state.

## Validation

Pending focused native shortcut tests, speech queue tests, browser lifecycle
regression checks, and a rebuilt macOS app.
