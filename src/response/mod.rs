//! The typed shapes an agent can return.

pub mod agent_response;
pub mod command_response;
pub mod general_response;
pub mod response_type;

pub use agent_response::AgentResponse;
pub use command_response::CommandResponse;
pub use general_response::GeneralResponse;
pub use response_type::ResponseType;
