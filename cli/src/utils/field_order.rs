pub struct FieldOrder;

impl FieldOrder {
    pub fn get_order() -> &'static [&'static str] {
        &[
            "name",
            "dockerComposeFile",
            "service",
            "workspaceFolder",
            "containerEnv",
            "forwardPorts",
            "postCreateCommand",
            "postStartCommand",
            "postAttachCommand",
            "remoteUser",
            "customizations",
        ]
    }
}
