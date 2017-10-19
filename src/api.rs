macro_rules! endpoint {
    // No reexports besides `Request` and `Response`.
    ($(#[$attr:meta])+ [$($outer_mod:ident),*], $inner_mod:ident) => {
        endpoint!($(#[$attr])+ [$($outer_mod),*], $inner_mod, []);
    };

    // No imports from super.
    ($(#[$attr:meta])+ [$($outer_mod:ident),*], $inner_mod:ident, [$($import:ident),*]) => {
        endpoint!($(#[$attr])+ [$($outer_mod),*], $inner_mod, [$($import),*], []);
    };

    // Explicit case.
    (
        $(#[$attr:meta])+
        [$($outer_mod:ident),*],
        $inner_mod:ident,
        [$($import:ident),*],
        [$($super_import:ident),*]
    ) => {
        #[$($attr)+]
        pub mod $inner_mod {
            use futures::Future;
            use hyper::client::Connect;
            use ruma_client_api::$($outer_mod::)*$inner_mod::Endpoint;
            $(use super::$super_import;)*
            pub use ruma_client_api::$($outer_mod::)*$inner_mod::{
                Request,
                Response,
                $($import),*
            };

            use {Client, Error};

            /// Make a request to this API endpoint.
            pub fn call<C>(
                client: Client<C>,
                request: Request,
            ) -> impl Future<Item = Response, Error = Error>
            where
                C: Connect,
            {
                client.request::<Endpoint>(request)
            }
        }
    };
}

/// Endpoints for the r0.x.x versions of the client API specification.
pub mod r0 {
    /// Account registration and management.
    pub mod account {
        endpoint!(
            /// Change the password for an account on this homeserver.
            [r0, account],
            change_password
        );

        endpoint!(
            /// Deactivate the user's account, removing all ability for the user to log in again.
            [r0, account],
            deactivate
        );

        endpoint!(
            /// Register for an account on this homeserver.
            [r0, account],
            register,
            [AuthenticationData, RegistrationKind]
        );

        endpoint!(
            /// Request a password change token by email.
            [r0, account],
            request_password_change_token
        );

        endpoint!(
            /// Request an account registration token by email.
            [r0, account],
            request_register_token
        );
    }

    /// Room aliases.
    pub mod alias {
        endpoint!(
            /// Create a new mapping from a room alias to a room ID.
            [r0, alias],
            create_alias
        );

        endpoint!(
            /// Remove a mapping from a room alias to a room ID.
            [r0, alias],
            delete_alias
        );

        endpoint!(
            /// Resolve a room alias to the corresponding room ID.
            [r0, alias],
            get_alias
        );
    }

    /// Client configuration.
    pub mod config {
        endpoint!(
            /// Set account data for the user.
            [r0, config],
            set_global_account_data
        );

        endpoint!(
            /// Set account data scoped to a room for the user.
            [r0, config],
            set_room_account_data
        );
    }

    /// Account contact information.
    pub mod contact {
        endpoint!(
            /// Add contact information to the user's account.
            [r0, contact],
            create_contact,
            [ThreePidCredentials]
        );

        endpoint!(
            /// Get a list of the third party identifiers that the homeserver has associated with the user's account.
            [r0, contact],
            get_contacts,
            [Medium, ThirdPartyIdentifier]
        );

        endpoint!(
            /// Request an email address verification token by email.
            [r0, contact],
            request_contact_verification_token
        );
    }

    /// Event context.
    pub mod context {
        endpoint!(
            /// Get a number of events that happened just before and after a given event.
            [r0, context],
            get_context
        );
    }

    /// The public room directory.
    pub mod directory {
        endpoint!(
            /// Get a number of events that happened just before and after a given event.
            [r0, directory],
            get_public_rooms,
            [PublicRoomsChunk]
        );
    }

    /// Event filters.
    pub mod filter {
        pub use ruma_client_api::r0::filter::{
            EventFormat,
            Filter,
            FilterDefinition,
            RoomEventFilter,
            RoomFilter,
        };

        endpoint!(
            /// Create a new filter.
            [r0, filter],
            create_filter
        );

        endpoint!(
            /// Get a filter.
            [r0, filter],
            get_filter
        );
    }

    /// Media repository.
    pub mod media {
        endpoint!(
            /// Upload media to the media repository.
            [r0, media],
            create_content
        );

        endpoint!(
            /// Download media from the media repository.
            [r0, media],
            get_content
        );

        endpoint!(
            /// Download a thumbnail image  for the media in the media repository.
            [r0, media],
            get_content_thumbnail,
            [Method]
        );
    }

    /// Room membership.
    pub mod membership {
        pub use ruma_client_api::r0::membership::ThirdPartySigned;

        endpoint!(
            /// Ban a user from a room.
            [r0, membership],
            ban_user
        );

        endpoint!(
            /// Permanently forget a room.
            [r0, membership],
            forget_room
        );

        endpoint!(
            /// Invite a user to a room.
            [r0, membership],
            invite_user
        );

        endpoint!(
            /// Join a room using its ID.
            [r0, membership],
            join_room_by_id
        );

        endpoint!(
            /// Join a room using its ID or an alias.
            [r0, membership],
            join_room_by_id_or_alias
        );

        endpoint!(
            /// Kick a user from a room.
            [r0, membership],
            kick_user
        );

        endpoint!(
            /// Leave a room.
            [r0, membership],
            leave_room
        );

        endpoint!(
            /// Unban a user from a room.
            [r0, membership],
            unban_user
        );
    }

    /// User presence.
    pub mod presence {
        endpoint!(
            /// Get a user's presence state.
            [r0, presence],
            get_presence
        );

        endpoint!(
            /// Get a list of presence events for users on the presence subscription list.
            [r0, presence],
            get_subscribed_presences
        );

        endpoint!(
            /// Set a user's presence state.
            [r0, presence],
            set_presence
        );

        endpoint!(
            /// Add or remove users from the presence subscription list.
            [r0, presence],
            update_presence_subscriptions
        );
    }

    /// User profiles.
    pub mod profile {
        endpoint!(
            /// Get the URL for a user's avatar.
            [r0, profile],
            get_avatar_url
        );

        endpoint!(
            /// Get a user's display name.
            [r0, profile],
            get_display_name
        );

        endpoint!(
            /// Get a user's full profile.
            [r0, profile],
            get_profile
        );

        endpoint!(
            /// Set the URL to the user's avatar.
            [r0, profile],
            set_avatar_url
        );

        endpoint!(
            /// Set the user's display name.
            [r0, profile],
            set_display_name
        );
    }

    /// Push notifications.
    pub mod push {
    }

    /// Event receipts.
    pub mod receipt {
        endpoint!(
            /// Update a receipt marker to point to a given event.
            [r0, receipt],
            create_receipt,
            [ReceiptType]
        );
    }

    /// Event redaction.
    pub mod redact {
        endpoint!(
            /// Redact an event from a room.
            [r0, redact],
            redact_event
        );
    }

    /// Room creation.
    pub mod room {
        endpoint!(
            /// Create a room.
            [r0, room],
            create_room,
            [CreationContent, RoomPreset, Visibility]
        );
    }

    /// Event searches.
    pub mod search {
        endpoint!(
            /// Search for events.
            [r0, search],
            search_events,
            [
                Categories,
                Criteria,
                EventContext,
                EventContextResult,
                Grouping,
                Groupings,
                ResultCategories,
                ResultGroup,
                RoomEventResults,
                SearchResult,
                UserProfile,
                GroupingKey,
                OrderBy,
                SearchKeys
            ]
        );
    }

    /// Sending events.
    pub mod send {
        endpoint!(
            /// Send a message to a room.
            [r0, send],
            send_message_event
        );

        endpoint!(
            /// Send a state event with an empty state key.
            [r0, send],
            send_state_event_for_empty_key
        );

        endpoint!(
            /// Send a state event with a particular state key.
            [r0, send],
            send_state_event_for_key
        );
    }

    /// Server administration.
    pub mod server {
        endpoint!(
            /// Get administrative information about a user.
            [r0, server],
            get_user_info,
            [ConnectionInfo, DeviceInfo, SessionInfo]
        );
    }

    /// User session management.
    pub mod session {
        endpoint!(
            /// Log in to an account, creating an access token.
            [r0, session],
            login,
            [LoginType, Medium]
        );

        endpoint!(
            /// Log out of an account by invalidating the access token.
            [r0, session],
            logout
        );
    }

    /// Getting and synchronizing events.
    pub mod sync {
        endpoint!(
            /// Get the list of members for a room.
            [r0, sync],
            get_member_events
        );

        endpoint!(
            /// Get message and state events for a room.
            [r0, sync],
            get_message_events,
            [Direction]
        );

        endpoint!(
            /// Get the state events for the current state of a room.
            [r0, sync],
            get_state_events
        );

        endpoint!(
            /// Get a particular state event with an empty state key for a room.
            [r0, sync],
            get_state_events_for_empty_key
        );

        endpoint!(
            /// Get a particular state event with a particular state key for a room.
            [r0, sync],
            get_state_events_for_key
        );

        endpoint!(
            /// Synchronize the client's state with the latest state on the homeserver.
            [r0, sync],
            sync_events,
            [
                AccountData,
                Ephemeral,
                Filter,
                InviteState,
                InvitedRoom,
                JoinedRoom,
                LeftRoom,
                Presence,
                Rooms,
                SetPresence,
                State,
                Timeline,
                UnreadNotificationsCount
            ]
        );
    }

    /// Tagging rooms.
    pub mod tag {
        endpoint!(
            /// Create a tag on a room.
            [r0, tag],
            create_tag
        );

        endpoint!(
            /// Delete a tag on a room.
            [r0, tag],
            delete_tag
        );

        endpoint!(
            /// Get the user's tags for a room.
            [r0, tag],
            get_tags
        );
    }

    /// Typing notifications.
    pub mod typing {
        endpoint!(
            /// Indicate that the user is currently typing.
            [r0, typing],
            create_typing_event
        );
    }

    /// Voice over IP.
    pub mod voip {
        endpoint!(
            /// Get credentials for initiating voice over IP calls via a TURN server.
            [r0, voip],
            get_turn_server_info
        );
    }
}

/// Endpoints that cannot change with new versions of the Matrix specification.
pub mod unversioned {
    endpoint!(
        /// Get the versions of the specification supported by this homeserver.
        [unversioned],
        get_supported_versions
    );
}
