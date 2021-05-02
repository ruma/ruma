//! The receiving server will verify the partial `m.room.member` event given in the request body.
//! If valid, the receiving server will issue an invite as per the [Inviting to a room] section
//! before returning a response to this request.
//!
//! [Inviting to a room]: https://matrix.org/docs/spec/server_server/r0.1.4#inviting-to-a-room

pub mod v1;
