#![no_std]
extern crate alloc;

use alloc::{string::{FromUtf8Error, String},
            vec::Vec};
use core::fmt;

use core_io::{self as io, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use libboard_zynq::sdio;

pub mod bootgen;
pub mod net_settings;
pub mod sd_reader;

#[derive(Debug)]
pub enum Error {
    SdError(sdio::sd_card::CardInitializationError),
    IoError(io::Error),
    Utf8Error(FromUtf8Error),
    KeyNotFoundError(String),
    NoConfig,
}

pub type Result<T> = core::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SdError(error) => write!(f, "SD error: {}", error),
            Error::IoError(error) => write!(f, "I/O error: {}", error),
            Error::Utf8Error(error) => write!(f, "UTF-8 error: {}", error),
            Error::KeyNotFoundError(name) => write!(f, "Configuration key `{}` not found", name),
            Error::NoConfig => write!(f, "Configuration not present"),
        }
    }
}

impl From<sdio::sd_card::CardInitializationError> for Error {
    fn from(error: sdio::sd_card::CardInitializationError) -> Self {
        Error::SdError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::Utf8Error(error)
    }
}

fn parse_config(key: &str, buffer: &mut Vec<u8>, file: fatfs::File<sd_reader::SdReader>) -> Result<()> {
    let prefix = [key, "="].concat().to_ascii_lowercase();
    for line in BufReader::new(file).lines() {
        let line = line?.to_ascii_lowercase();
        if line.starts_with(&prefix) {
            buffer.extend(line[prefix.len()..].as_bytes());
            return Ok(());
        }
    }
    Err(Error::KeyNotFoundError(key.into()))
}

type FileSystem = fatfs::FileSystem<sd_reader::SdReader>;
static mut FS: Option<FileSystem> = None;

#[allow(static_mut_refs)]
pub fn get_filesystem() -> &'static Option<FileSystem> {
    unsafe { &FS }
}

const NEWLINE: &[u8] = b"\n";

pub fn init() -> Result<()> {
    let sdio = sdio::Sdio::sdio0(true);
    if !sdio.is_card_inserted() {
        Err(sdio::sd_card::CardInitializationError::NoCardInserted)?;
    }
    let sd = sdio::sd_card::SdCard::from_sdio(sdio)?;
    let reader = sd_reader::SdReader::new(sd);

    let fs = reader.mount_fatfs(sd_reader::PartitionEntry::Entry1)?;

    unsafe {
        FS = Some(fs);
    }
    Ok(())
}

pub fn from_fs(fs: Option<FileSystem>) {
    unsafe { FS = fs }
}

pub fn read(key: &str) -> Result<Vec<u8>> {
    if let Some(fs) = get_filesystem() {
        let root_dir = fs.root_dir();
        let mut buffer: Vec<u8> = Vec::new();
        match root_dir.open_file(&["/CONFIG/", key, ".BIN"].concat()) {
            Ok(mut f) => f.read_to_end(&mut buffer).map(|_| ())?,
            Err(_) => match root_dir.open_file("/CONFIG.TXT") {
                Ok(f) => parse_config(key, &mut buffer, f)?,
                Err(_) => return Err(Error::KeyNotFoundError(key.into())),
            },
        };
        Ok(buffer)
    } else {
        Err(Error::NoConfig)
    }
}

pub fn read_str(key: &str) -> Result<String> {
    Ok(String::from_utf8(read(key)?)?)
}

pub fn remove(key: &str) -> Result<()> {
    if let Some(fs) = get_filesystem() {
        let root_dir = fs.root_dir();
        match root_dir.remove(&["/CONFIG/", key, ".BIN"].concat()) {
            Ok(()) => Ok(()),
            Err(_) => {
                let prefix = [key, "="].concat().to_ascii_lowercase();
                match root_dir.create_file("/CONFIG.TXT") {
                    Ok(mut f) => {
                        let mut buffer = String::new();
                        f.read_to_string(&mut buffer)?;
                        f.seek(SeekFrom::Start(0))?;
                        f.truncate()?;
                        for line in buffer.lines() {
                            if line.len() > 0 && !line.to_ascii_lowercase().starts_with(&prefix) {
                                f.write(line.as_bytes())?;
                                f.write(NEWLINE)?;
                            }
                        }
                        Ok(())
                    }
                    Err(_) => Err(Error::KeyNotFoundError(key.into())),
                }
            }
        }
    } else {
        Err(Error::NoConfig)
    }
}

pub fn write(key: &str, value: Vec<u8>) -> Result<()> {
    if get_filesystem().is_none() {
        return Err(Error::NoConfig);
    }
    let fs = get_filesystem().as_ref().unwrap();
    let root_dir = fs.root_dir();
    let is_str = value.len() <= 100 && value.is_ascii() && !value.contains(&b'\n');
    if key == "boot" {
        let mut f = root_dir.create_file("/BOOT.BIN")?;
        f.truncate()?;
        f.write_all(&value)?;
        drop(f);
    } else {
        let _ = remove(key);
        if is_str {
            let mut f = root_dir.create_file("/CONFIG.TXT")?;
            f.seek(SeekFrom::End(0))?;
            write!(f, "{}={}\n", key, String::from_utf8(value).unwrap())?;
        } else {
            let dir = root_dir.create_dir("/CONFIG")?;
            let mut f = dir.create_file(&[key, ".BIN"].concat())?;
            f.write_all(&value)?;
        }
    }
    Ok(())
}
