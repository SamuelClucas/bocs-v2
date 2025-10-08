pub mod world;
pub mod backend;
pub mod state;
pub mod app_event_dispatch;

// Traits
pub trait Stage<T>: Sized {
    /// Instantiates all child nodes managed by this Stage
    fn new(payload: T) ->  Result<Self, Box<dyn std::error::Error>>; 

}

// Structs


// Enums
