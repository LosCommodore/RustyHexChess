# Reminder on the type state pattern
I decided to use the type state patten for the game state machine.
- This is maybe a bit heavy but a good exercise

Some AI generated text as a reminder for the pattern:

## The Type-State Pattern in Rust
The Type-State Pattern uses Rust's affine type system (ownership and move semantics) and generics to represent the states of a state machine as distinct compile-time types. It enforces valid state transitions and prevents illegal operations at compile time.

### Core Concepts

* States as Types: Each state is represented by a unique zero-sized struct (struct StateA;).
* State-Specific Methods: Methods are implemented only for specific type instances (impl Subsystem<StateA>).
* Consuming Transitions: Transition methods take self by value. This moves and consumes the old state, making it impossible to reuse.
* Compile-Time Safety: Attempting to call a method not allowed in the current state results in a compilation error.

### Minimal Blueprint

// 1. Define the statespub struct Disconnected;pub struct Connected;
// 2. Define the generic containerpub struct Connection<State> {
    id: u32,
    state: State,
}
// 3. Methods available in ALL statesimpl<State> Connection<State> {
    pub fn id(&self) -> u32 { self.id }
}
// 4. Methods available ONLY in 'Disconnected'impl Connection<Disconnected> {
    pub fn new(id: u32) -> Self {
        Self { id, state: Disconnected }
    }

    // Transition: Consumes Disconnected, returns Connected
    pub fn connect(self) -> Connection<Connected> {
        Connection {
            id: self.id,
            state: Connected,
        }
    }
}
// 5. Methods available ONLY in 'Connected'impl Connection<Connected> {
    pub fn send_data(&self, data: &[u8]) {
        // Transmission logic here
    }

    // Transition: Consumes Connected, returns Disconnected
    pub fn disconnect(self) -> Connection<Disconnected> {
        Connection {
            id: self.id,
            state: Disconnected,
        }
    }
}

### Key Design Patterns## Handling Dynamic Transitions
If a state transition depends on runtime data and can result in multiple target states, wrap the resulting type-states inside a standard enum:

pub enum NextState {
    Stay(Connection<Connected>),
    Terminated(Connection<Disconnected>),
}

### Shared Fields Access
To avoid boilerplate when accessing fields common to all states, implement a shared Trait or expose getter methods on the generic impl<State> Container<State> block.

### Advantages & Trade-offs

* Pro: Zero runtime overhead (zero-sized structs are optimized away).
* Pro: Eliminates runtime checks like if state != State::Connected.
* Con: Can lead to code duplication across different impl blocks if not designed carefully.
* Con: Increased API surface and slightly more complex compiler error messages for users.

------------------------------
Would you like to explore how to combine this pattern with traits to reduce code duplication for shared fields, or should we look at how to handle errors during state transitions cleanly?

