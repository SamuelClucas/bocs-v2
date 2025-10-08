pub mod world;
pub mod root_stage;
pub mod state;
pub mod app_event_dispatch;

// Traits
pub trait Stage<T>: Sized {
    /// Instantiates all child nodes managed by this Stage
    fn new(payload: T) ->  Result<Self, Box<dyn std::error::Error>>; 

}

// Structs


// Enums
