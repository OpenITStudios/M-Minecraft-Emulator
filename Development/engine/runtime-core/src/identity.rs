use crate::client::IdentityState;

fn identity_components(value: &str) -> Option<(&str, &str)> {
    let (uuid, mac) = value.split_once('|')?;
    let (uuid, mac) = (uuid.trim(), mac.trim());
    (!uuid.is_empty() && !mac.is_empty() && !mac.contains('|')).then_some((uuid, mac))
}

pub(crate) fn classify_identity_state<'a>(
    current: Option<&'a str>,
    peers: impl IntoIterator<Item = Option<&'a str>>,
) -> IdentityState {
    let Some((uuid, mac)) = current.and_then(identity_components) else {
        return IdentityState::Unknown;
    };

    for peer in peers {
        let Some((peer_uuid, peer_mac)) = peer.and_then(identity_components) else {
            return IdentityState::Unknown;
        };
        if uuid.eq_ignore_ascii_case(peer_uuid) || mac.eq_ignore_ascii_case(peer_mac) {
            return IdentityState::Duplicate;
        }
    }

    IdentityState::Unique
}

#[cfg(test)]
mod tests {
    use super::classify_identity_state;
    use crate::client::IdentityState;

    #[test]
    fn identity_collision_policy_is_fail_closed() {
        assert_eq!(
            classify_identity_state(Some("uuid-a|mac-a"), [Some("uuid-b|mac-b")]),
            IdentityState::Unique
        );
        assert_eq!(
            classify_identity_state(Some("uuid-a|mac-a"), [Some("uuid-a|mac-b")]),
            IdentityState::Duplicate
        );
        assert_eq!(
            classify_identity_state(Some("uuid-a|mac-a"), [Some("uuid-b|mac-a")]),
            IdentityState::Duplicate
        );
        assert_eq!(
            classify_identity_state(Some("uuid-a|mac-a"), [None]),
            IdentityState::Unknown
        );
        assert_eq!(
            classify_identity_state(None, [Some("uuid-b|mac-b")]),
            IdentityState::Unknown
        );
    }
}
