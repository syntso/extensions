use super::{EVENTS, FILE_ACCESS};
use crate::manifest::Manifest;

const NEWER_API: &str = "0.2";

pub fn reach(manifest: &Manifest) -> Vec<String> {
    let mut problems = Vec::new();

    problems.extend(api(manifest));
    problems.extend(hosts(manifest));
    problems.extend(secrets(manifest));
    problems.extend(documents(manifest));
    problems.extend(events_and_files(manifest));
    problems.extend(own_actions(manifest));

    problems
}

fn api(manifest: &Manifest) -> Option<String> {
    let newer = manifest.capabilities.uses_api_0_2() || !manifest.contributes.is_empty();

    (newer && manifest.api != NEWER_API).then(|| {
        "contributes and the capabilities http, secrets, documents, events, timers and files \
         need api 0.2"
            .to_owned()
    })
}

fn hosts(manifest: &Manifest) -> Vec<String> {
    manifest
        .capabilities
        .http
        .iter()
        .filter(|host| !is_host(host))
        .map(|host| format!("http host {host} is not a lowercase host with an optional :port"))
        .collect()
}

fn secrets(manifest: &Manifest) -> Vec<String> {
    let capabilities = &manifest.capabilities;

    capabilities
        .secrets
        .iter()
        .flat_map(|(secret, hosts)| {
            let unlisted = hosts
                .iter()
                .filter(|host| !capabilities.http.contains(host))
                .map(move |host| {
                    format!("secret {secret} goes to {host}, which http does not list")
                });
            let nowhere = hosts
                .is_empty()
                .then(|| format!("secret {secret} names no host"));

            unlisted.chain(nowhere)
        })
        .collect()
}

fn documents(manifest: &Manifest) -> Option<String> {
    let typed = !manifest.contributes.document.is_empty();

    (manifest.capabilities.documents != typed)
        .then(|| "documents = true and contributes.document come together".to_owned())
}

fn events_and_files(manifest: &Manifest) -> Vec<String> {
    let capabilities = &manifest.capabilities;

    let events = capabilities
        .events
        .iter()
        .filter(|event| !EVENTS.contains(&event.as_str()))
        .map(|event| format!("event {event} is not one of {}", EVENTS.join(", ")));
    let files = capabilities
        .files
        .iter()
        .filter(|access| !FILE_ACCESS.contains(&access.as_str()))
        .map(|access| format!("files = {access:?} is not read or read-write"));

    events.chain(files).collect()
}

fn own_actions(manifest: &Manifest) -> Vec<String> {
    let prefix = format!("{}.", manifest.id);

    manifest
        .contributes
        .actions()
        .filter(|action| !action.starts_with(&prefix))
        .map(|action| format!("contributes names {action}, not one of its own actions"))
        .collect()
}

fn is_host(entry: &str) -> bool {
    let (host, port) = entry.rsplit_once(':').unwrap_or((entry, "443"));

    !host.is_empty()
        && host.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || ".-".contains(character)
        })
        && port.parse::<u16>().is_ok_and(|port| port > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(text: &str) -> Manifest {
        let base = "id = \"board\"\nname = \"Board\"\nversion = \"0.2.0\"\napi = \"0.2\"\n\
                    description = \"d\"\nrepository = \"https://example.com\"\n";

        toml::from_str(&format!("{base}{text}")).expect("manifest")
    }

    #[test]
    fn accepts_every_capability_and_contribution() {
        let manifest = manifest(
            "[capabilities]\nhttp = [\"grafana.example.com\", \"127.0.0.1:3000\"]\n\
             secrets = { \"grafana/token\" = [\"grafana.example.com\"] }\ndocuments = true\n\
             events = [\"journal\"]\ntimers = true\nfiles = \"read\"\n\n\
             [[contributes.document]]\ntype = \"diagram\"\ntitle = \"Diagram\"\nsuffix = \"uml\"\n\n\
             [[contributes.command]]\ntitle = \"Refresh\"\naction = \"board.refresh\"\n",
        );

        assert!(reach(&manifest).is_empty(), "{:?}", reach(&manifest));
    }

    #[test]
    fn finds_every_kind_of_problem() {
        let manifest = manifest(
            "[capabilities]\nhttp = [\"https://x\"]\nsecrets = { token = [\"elsewhere\"] }\n\
             documents = true\nevents = [\"keystrokes\"]\nfiles = \"write\"\n\n\
             [[contributes.command]]\ntitle = \"Wipe\"\naction = \"ssh.exec\"\n",
        );

        assert_eq!(reach(&manifest).len(), 6, "{:?}", reach(&manifest));
    }

    #[test]
    fn api_0_1_cannot_contribute() {
        let mut manifest =
            manifest("[[contributes.command]]\ntitle = \"Go\"\naction = \"board.go\"\n");
        manifest.api = "0.1".to_owned();

        assert_eq!(reach(&manifest).len(), 1);
    }
}
