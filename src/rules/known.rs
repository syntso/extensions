pub const APIS: &[&str] = &["0.1", "0.2"];

pub const CALLS: &[&str] = &[
    "core.ping",
    "project.list",
    "resource.get",
    "resource.list",
    "resource.put",
    "resource.remove",
    "session.list",
    "sftp.list",
    "sftp.mkdir",
    "sftp.read",
    "sftp.remove",
    "sftp.rename",
    "sftp.stat",
    "sftp.write",
    "ssh.connections",
    "ssh.exec",
    "ssh.warm",
    "tunnel.close",
    "tunnel.list",
    "tunnel.open",
];

pub const EVENTS: &[&str] = &["journal", "session", "resource", "approval"];

pub const FILE_ACCESS: &[&str] = &["read", "read-write"];
