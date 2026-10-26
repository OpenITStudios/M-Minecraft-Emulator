use m_bedrock_virtual_clients_core::{ClientId, VirtualClients};

#[test]
fn native_client_lifecycle_is_manual() {
    let app = VirtualClients;
    assert_eq!(
        app.open(ClientId::Native)
            .expect_err("Native open belongs to the host")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );
    assert_eq!(
        app.stop(Some(ClientId::Native))
            .expect_err("Native stop belongs to the host")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );
}

#[test]
fn start_rejects_invalid_client_count() {
    let app = VirtualClients;
    let error = app.start(0).expect_err("zero clients must be rejected");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn native_client_cannot_use_vm_clean_state_actions() {
    let app = VirtualClients;

    assert_eq!(
        app.suspend(Some(ClientId::Native))
            .expect_err("native suspend must be external/manual")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );

    assert_eq!(
        app.restart(ClientId::Native)
            .expect_err("native restart must be external/manual")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );

    assert_eq!(
        app.set_ready(ClientId::Native)
            .expect_err("native client has no QA_READY snapshot")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );

    assert_eq!(
        app.reset(ClientId::Native)
            .expect_err("native client has no VM reset")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );
}

#[test]
fn virtual_count_contract_is_one_to_three() {
    let app = VirtualClients;

    assert_eq!(
        app.start(0)
            .expect_err("zero Virtual instances must be rejected")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );
    assert_eq!(
        app.start(4)
            .expect_err("there are only three Virtual instances")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );
    assert_eq!(
        app.resources(0)
            .expect_err("zero Virtual instances must be rejected")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );
    assert_eq!(
        app.resources(4)
            .expect_err("there are only three Virtual instances")
            .kind(),
        std::io::ErrorKind::InvalidInput
    );
}

#[test]
fn virtual_names_are_stable_and_exclude_native() {
    assert_eq!(ClientId::Native.as_str(), "Native");
    assert_eq!(ClientId::Virtual01.as_str(), "Virtual-01");
    assert_eq!(ClientId::Virtual02.as_str(), "Virtual-02");
    assert_eq!(ClientId::Virtual03.as_str(), "Virtual-03");
    assert_eq!(
        ClientId::VIRTUAL,
        [
            ClientId::Virtual01,
            ClientId::Virtual02,
            ClientId::Virtual03,
        ]
    );
}
