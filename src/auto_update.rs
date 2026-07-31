use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

pub struct ReleaseInfo {
    version: String,
}

pub fn install_update() -> crate::Result<()> {
    // TODO: download into localappdata/updates dir
    let url = "https://github.com/snubwoody/mukwa/releases/download/v0.1.0-alpha.4/Mukwa-x86_64-Setup.exe";
    let response = ureq::get(url).call()?;
    let mut reader = response.into_body().into_reader();

    // TODO: stream directly instead of loading into memory
    let mut buffer: Vec<u8> = Vec::new();

    reader.read_to_end(&mut buffer)?;
    let path = PathBuf::from("Mukwa-Update.exe");
    fs::write(&path, &buffer)?;
    // TODO: output logs to log dir
    // TODO: test installing the same version and previous versions?
    // TODO: zed had /update=true
    // DOC: brings popup when not installed
    let absolute_path = path.canonicalize()?;
    dbg!(&absolute_path);
    let output = Command::new(absolute_path).arg("/verysilent").output()?;
    assert!(output.status.success());
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn update() -> crate::Result<()> {
        install_update()?;
        Ok(())
    }
}
