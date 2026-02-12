#![allow(clippy::needless_return)]

extern crate alloc;

use alloc::{string::String, vec, vec::Vec};

use core_io::{Read, Seek, SeekFrom, Write as IoWrite};
use libboard_zynq::{println, sdio, slcr, stdio, timer};
use libconfig::sd_reader;
use libcortex_a9::asm;

const MAGIC0: u8 = b'S';
const MAGIC1: u8 = b'Z';
const VERSION: u8 = 1;
const FW_ID: &[u8] = b"SZL-SD-SVC/1";

const CMD_IDENTIFY: u8 = 0x00;
const CMD_LIST: u8 = 0x01;
const CMD_UPLOAD_BEGIN: u8 = 0x02;
const CMD_UPLOAD_CHUNK: u8 = 0x03;
const CMD_UPLOAD_END: u8 = 0x04;
const CMD_DOWNLOAD: u8 = 0x05;
const CMD_DELETE: u8 = 0x06;
const CMD_REBOOT: u8 = 0x07;

const STATUS_OK: u8 = 0x00;
const STATUS_BAD_FRAME: u8 = 0x01;
const STATUS_BAD_CMD: u8 = 0x02;
const STATUS_BAD_STATE: u8 = 0x03;
const STATUS_INVALID_NAME: u8 = 0x04;
const STATUS_NOT_FOUND: u8 = 0x05;
const STATUS_IO_ERROR: u8 = 0x06;
const STATUS_NO_SD: u8 = 0x07;
const STATUS_NO_SPACE: u8 = 0x08;
const STATUS_INTERNAL: u8 = 0x09;

const FLAG_MORE: u8 = 0x01;

const MAX_FRAME_PAYLOAD: usize = 2048;
const MAX_CHUNK: usize = 1024;
const MAX_DOWNLOAD_CHUNK: usize = 1024;

const READY_SENTINEL: &str = "SZL-SD READY v1; switching to binary protocol";

struct Request {
    cmd: u8,
    seq: u16,
    payload: Vec<u8>,
}

type SdReader = sd_reader::SdReader;
type FileSystem = fatfs::FileSystem<SdReader>;
struct UploadSession {
    id: u16,
    filename: String,
    expected_size: u32,
    received_size: u32,
    crc: u32,
}

struct Service {
    fs: FileSystem,
    next_upload_id: u16,
    upload: Option<UploadSession>,
}

pub fn run() -> ! {
    println!("SZL SD Service starting...");
    println!("UART ready: 1500000");

    let mut service = loop {
        println!("Checking SD card...");
        match mount_fs() {
            Ok(fs) => {
                println!("SD mounted successfully");
                println!("Mode: root-only FAT32 file service (8.3 names)");
                println!("Commands: list/upload/download/delete/reboot");
                println!("{}", READY_SENTINEL);
                for _ in 0..20_000_000 {
                    asm::nop();
                }
                break Service {
                    fs,
                    next_upload_id: 1,
                    upload: None,
                };
            }
            Err(err) => {
                println!("SD mount failed: {}; retrying...", err);
                wait_ms(1000);
            }
        }
    };

    log::set_max_level(log::LevelFilter::Off);

    loop {
        match read_request() {
            Ok(req) => service.handle_request(req),
            Err(()) => {
                let _ = send_response(0, 0, STATUS_BAD_FRAME, &[]);
            }
        }
    }
}

impl Service {
    fn handle_request(&mut self, req: Request) {
        match req.cmd {
            CMD_IDENTIFY => self.cmd_identify(req.seq),
            CMD_LIST => self.cmd_list(req.seq),
            CMD_UPLOAD_BEGIN => self.cmd_upload_begin(req.seq, &req.payload),
            CMD_UPLOAD_CHUNK => self.cmd_upload_chunk(req.seq, &req.payload),
            CMD_UPLOAD_END => self.cmd_upload_end(req.seq, &req.payload),
            CMD_DOWNLOAD => self.cmd_download(req.seq, &req.payload),
            CMD_DELETE => self.cmd_delete(req.seq, &req.payload),
            CMD_REBOOT => self.cmd_reboot(req.seq),
            _ => {
                let _ = send_response(req.cmd, req.seq, STATUS_BAD_CMD, &[]);
            }
        }
    }

    fn cmd_identify(&mut self, seq: u16) {
        let _ = send_response(CMD_IDENTIFY, seq, STATUS_OK, FW_ID);
    }

    fn cmd_list(&mut self, seq: u16) {
        let root = self.fs.root_dir();
        let mut payload = Vec::new();
        payload.extend_from_slice(&0u16.to_le_bytes());

        let mut count: u16 = 0;
        let entries = root.iter();
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => {
                    let _ = send_response(CMD_LIST, seq, STATUS_IO_ERROR, &[]);
                    return;
                }
            };
            if !entry.is_file() {
                continue;
            }
            let name_bytes = entry.short_file_name_as_bytes();
            if name_bytes.len() > u8::MAX as usize {
                continue;
            }

            let need = 1 + name_bytes.len() + 4;
            if payload.len() + need > MAX_FRAME_PAYLOAD {
                let _ = send_response(CMD_LIST, seq, STATUS_NO_SPACE, &[]);
                return;
            }

            payload.push(name_bytes.len() as u8);
            payload.extend_from_slice(name_bytes);
            payload.extend_from_slice(&(entry.len() as u32).to_le_bytes());
            count = count.saturating_add(1);
        }

        payload[0..2].copy_from_slice(&count.to_le_bytes());
        let _ = send_response(CMD_LIST, seq, STATUS_OK, &payload);
    }

    fn cmd_upload_begin(&mut self, seq: u16, payload: &[u8]) {
        // Replace any stale/incomplete session so host can recover by issuing
        // a fresh UPLOAD_BEGIN without requiring reboot.
        self.upload = None;
        let (name, mut pos) = match parse_name(payload) {
            Ok(v) => v,
            Err(status) => {
                let _ = send_response(CMD_UPLOAD_BEGIN, seq, status, &[]);
                return;
            }
        };

        if payload.len() < pos + 5 {
            let _ = send_response(CMD_UPLOAD_BEGIN, seq, STATUS_BAD_FRAME, &[]);
            return;
        }

        let expected_size = u32::from_le_bytes([payload[pos], payload[pos + 1], payload[pos + 2], payload[pos + 3]]);
        pos += 4;
        let overwrite = payload[pos];

        if overwrite != 1 {
            let _ = send_response(CMD_UPLOAD_BEGIN, seq, STATUS_BAD_FRAME, &[]);
            return;
        }

        let root = self.fs.root_dir();
        let mut file = match root.create_file(name.as_str()) {
            Ok(f) => f,
            Err(_) => {
                let _ = send_response(CMD_UPLOAD_BEGIN, seq, STATUS_IO_ERROR, &[]);
                return;
            }
        };

        if file.truncate().is_err() {
            let _ = send_response(CMD_UPLOAD_BEGIN, seq, STATUS_IO_ERROR, &[]);
            return;
        }

        let id = self.next_upload_id;
        self.next_upload_id = self.next_upload_id.wrapping_add(1);
        if self.next_upload_id == 0 {
            self.next_upload_id = 1;
        }

        self.upload = Some(UploadSession {
            id,
            filename: name,
            expected_size,
            received_size: 0,
            crc: 0xFFFF_FFFF,
        });

        let _ = send_response(CMD_UPLOAD_BEGIN, seq, STATUS_OK, &id.to_le_bytes());
    }

    fn cmd_upload_chunk(&mut self, seq: u16, payload: &[u8]) {
        if payload.len() < 8 {
            let _ = send_response(CMD_UPLOAD_CHUNK, seq, STATUS_BAD_FRAME, &[]);
            return;
        }
        let session_id = u16::from_le_bytes([payload[0], payload[1]]);
        let offset = u32::from_le_bytes([payload[2], payload[3], payload[4], payload[5]]);
        let chunk_len = u16::from_le_bytes([payload[6], payload[7]]) as usize;

        if chunk_len > MAX_CHUNK || payload.len() != 8 + chunk_len {
            let _ = send_response(CMD_UPLOAD_CHUNK, seq, STATUS_BAD_FRAME, &[]);
            return;
        }

        let upload = match self.upload.as_mut() {
            Some(u) => u,
            None => {
                let _ = send_response(CMD_UPLOAD_CHUNK, seq, STATUS_BAD_STATE, &[]);
                return;
            }
        };

        if upload.id != session_id || offset != upload.received_size {
            let _ = send_response(CMD_UPLOAD_CHUNK, seq, STATUS_BAD_STATE, &[]);
            return;
        }

        let chunk = &payload[8..];
        let root = self.fs.root_dir();
        let mut file = match root.open_file(upload.filename.as_str()) {
            Ok(f) => f,
            Err(_) => {
                let _ = send_response(CMD_UPLOAD_CHUNK, seq, STATUS_IO_ERROR, &[]);
                return;
            }
        };
        if file.seek(SeekFrom::Start(offset as u64)).is_err() || file.write_all(chunk).is_err() || file.flush().is_err() {
            let _ = send_response(CMD_UPLOAD_CHUNK, seq, STATUS_IO_ERROR, &[]);
            return;
        }

        upload.received_size = upload.received_size.saturating_add(chunk.len() as u32);
        upload.crc = crc32_update(upload.crc, chunk);

        let _ = send_response(CMD_UPLOAD_CHUNK, seq, STATUS_OK, &upload.received_size.to_le_bytes());
    }

    fn cmd_upload_end(&mut self, seq: u16, payload: &[u8]) {
        if payload.len() != 6 {
            let _ = send_response(CMD_UPLOAD_END, seq, STATUS_BAD_FRAME, &[]);
            return;
        }
        let session_id = u16::from_le_bytes([payload[0], payload[1]]);
        let expected_crc = u32::from_le_bytes([payload[2], payload[3], payload[4], payload[5]]);

        let upload = match self.upload.take() {
            Some(u) => u,
            None => {
                let _ = send_response(CMD_UPLOAD_END, seq, STATUS_BAD_STATE, &[]);
                return;
            }
        };

        if upload.id != session_id {
            self.upload = Some(upload);
            let _ = send_response(CMD_UPLOAD_END, seq, STATUS_BAD_STATE, &[]);
            return;
        }

        if upload.received_size != upload.expected_size {
            let _ = send_response(CMD_UPLOAD_END, seq, STATUS_BAD_STATE, &[]);
            return;
        }

        let actual_crc = crc32_finish(upload.crc);
        if expected_crc != actual_crc {
            let _ = send_response(CMD_UPLOAD_END, seq, STATUS_BAD_STATE, &[]);
            return;
        }

        let _ = send_response(CMD_UPLOAD_END, seq, STATUS_OK, &upload.received_size.to_le_bytes());
    }

    fn cmd_download(&mut self, seq: u16, payload: &[u8]) {
        let (name, pos) = match parse_name(payload) {
            Ok(v) => v,
            Err(status) => {
                let _ = send_response(CMD_DOWNLOAD, seq, status, &[]);
                return;
            }
        };
        if pos != payload.len() {
            let _ = send_response(CMD_DOWNLOAD, seq, STATUS_BAD_FRAME, &[]);
            return;
        }

        let root = self.fs.root_dir();
        let mut file = match root.open_file(name.as_str()) {
            Ok(f) => f,
            Err(_) => {
                let _ = send_response(CMD_DOWNLOAD, seq, STATUS_NOT_FOUND, &[]);
                return;
            }
        };

        let mut all = Vec::new();
        if file.read_to_end(&mut all).is_err() {
            let _ = send_response(CMD_DOWNLOAD, seq, STATUS_IO_ERROR, &[]);
            return;
        }
        let file_size = all.len() as u32;
        let file_crc = crc32(all.as_slice());

        let mut offset = 0usize;
        if all.is_empty() {
            let mut meta = Vec::new();
            meta.push(0);
            meta.extend_from_slice(&file_size.to_le_bytes());
            meta.extend_from_slice(&file_crc.to_le_bytes());
            meta.extend_from_slice(&0u32.to_le_bytes());
            let _ = send_response(CMD_DOWNLOAD, seq, STATUS_OK, &meta);
            return;
        }

        while offset < all.len() {
            let remaining = all.len() - offset;
            let chunk_len = core::cmp::min(remaining, MAX_DOWNLOAD_CHUNK);
            let more = if offset + chunk_len < all.len() { FLAG_MORE } else { 0 };
            let mut out = Vec::with_capacity(1 + 4 + 4 + 4 + chunk_len);
            out.push(more);
            out.extend_from_slice(&file_size.to_le_bytes());
            out.extend_from_slice(&file_crc.to_le_bytes());
            out.extend_from_slice(&(offset as u32).to_le_bytes());
            out.extend_from_slice(&all[offset..offset + chunk_len]);
            if send_response(CMD_DOWNLOAD, seq, STATUS_OK, &out).is_err() {
                return;
            }
            offset += chunk_len;
        }
    }

    fn cmd_delete(&mut self, seq: u16, payload: &[u8]) {
        let (name, pos) = match parse_name(payload) {
            Ok(v) => v,
            Err(status) => {
                let _ = send_response(CMD_DELETE, seq, status, &[]);
                return;
            }
        };
        if pos != payload.len() {
            let _ = send_response(CMD_DELETE, seq, STATUS_BAD_FRAME, &[]);
            return;
        }

        let root = self.fs.root_dir();
        match root.remove(name.as_str()) {
            Ok(_) => {
                let _ = send_response(CMD_DELETE, seq, STATUS_OK, &[]);
            }
            Err(_) => {
                let _ = send_response(CMD_DELETE, seq, STATUS_NOT_FOUND, &[]);
            }
        }
    }

    fn cmd_reboot(&mut self, seq: u16) {
        let _ = send_response(CMD_REBOOT, seq, STATUS_OK, &[]);
        wait_ms(50);
        slcr::reboot();
        loop {
            asm::nop();
        }
    }
}

fn parse_name(payload: &[u8]) -> Result<(String, usize), u8> {
    if payload.is_empty() {
        return Err(STATUS_BAD_FRAME);
    }
    let name_len = payload[0] as usize;
    if name_len == 0 || payload.len() < 1 + name_len {
        return Err(STATUS_BAD_FRAME);
    }

    let raw = &payload[1..1 + name_len];
    let mut up = String::new();
    for &b in raw {
        if b.is_ascii_lowercase() {
            up.push((b - b'a' + b'A') as char);
        } else {
            up.push(b as char);
        }
    }

    if !is_valid_83(&up) {
        return Err(STATUS_INVALID_NAME);
    }

    Ok((up, 1 + name_len))
}

fn is_valid_83(name: &str) -> bool {
    if name.is_empty() || name.len() > 12 {
        return false;
    }
    if name.contains('/') || name.contains('\\') || name.contains(':') || name.contains("..") || name.contains(' ') {
        return false;
    }

    let mut parts = name.split('.');
    let base = match parts.next() {
        Some(v) => v,
        None => return false,
    };
    let ext = parts.next();
    if parts.next().is_some() {
        return false;
    }

    if base.is_empty() || base.len() > 8 {
        return false;
    }
    if !base.bytes().all(is_allowed_83_char) {
        return false;
    }

    if let Some(ext) = ext {
        if ext.is_empty() || ext.len() > 3 {
            return false;
        }
        if !ext.bytes().all(is_allowed_83_char) {
            return false;
        }
    }

    true
}

fn is_allowed_83_char(b: u8) -> bool {
    b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_' || b == b'-'
}

fn mount_fs() -> Result<FileSystem, &'static str> {
    let sdio0 = sdio::Sdio::sdio0(true);
    if !sdio0.is_card_inserted() {
        return Err("no SD card");
    }
    let sd = sdio::sd_card::SdCard::from_sdio(sdio0).map_err(|_| "card init failed")?;
    let reader = sd_reader::SdReader::new(sd);
    reader
        .mount_fatfs(sd_reader::PartitionEntry::Entry1)
        .map_err(|_| "FAT mount failed")
}

fn wait_ms(ms: u64) {
    let start = timer::get_ms();
    while timer::get_ms().wrapping_sub(start) < ms {
        asm::nop();
    }
}

fn read_request() -> Result<Request, ()> {
    loop {
        let b0 = uart_read_byte_blocking();
        if b0 != MAGIC0 {
            continue;
        }
        let b1 = uart_read_byte_blocking();
        if b1 != MAGIC1 {
            continue;
        }

        let version = uart_read_byte_blocking();
        if version != VERSION {
            return Err(());
        }

        let cmd = uart_read_byte_blocking();
        let seq = uart_read_u16_le();
        let payload_len = uart_read_u32_le() as usize;
        if payload_len > MAX_FRAME_PAYLOAD {
            return Err(());
        }

        let mut payload = vec![0u8; payload_len];
        for b in payload.iter_mut() {
            *b = uart_read_byte_blocking();
        }
        let recv_crc = uart_read_u32_le();

        let mut crc_data = Vec::with_capacity(1 + 1 + 2 + 4 + payload_len);
        crc_data.push(version);
        crc_data.push(cmd);
        crc_data.extend_from_slice(&seq.to_le_bytes());
        crc_data.extend_from_slice(&(payload_len as u32).to_le_bytes());
        crc_data.extend_from_slice(&payload);
        let expected = crc32(crc_data.as_slice());
        if expected != recv_crc {
            return Err(());
        }

        return Ok(Request { cmd, seq, payload });
    }
}

fn send_response(cmd: u8, seq: u16, status: u8, payload: &[u8]) -> Result<(), ()> {
    if payload.len() + 1 > MAX_FRAME_PAYLOAD {
        return Err(());
    }

    let mut frame = Vec::with_capacity(2 + 1 + 1 + 2 + 4 + 1 + payload.len() + 4);
    frame.push(MAGIC0);
    frame.push(MAGIC1);
    frame.push(VERSION);
    frame.push(cmd);
    frame.extend_from_slice(&seq.to_le_bytes());
    frame.extend_from_slice(&((payload.len() + 1) as u32).to_le_bytes());
    frame.push(status);
    frame.extend_from_slice(payload);

    let crc = crc32(&frame[2..]);
    frame.extend_from_slice(&crc.to_le_bytes());

    uart_write_all(&frame);
    Ok(())
}

fn uart_write_all(data: &[u8]) {
    let mut uart = stdio::get_uart();
    for &b in data {
        uart.write_byte(b);
    }
}

fn uart_read_byte_blocking() -> u8 {
    loop {
        {
            let mut uart = stdio::get_uart();
            if let Ok(b) = uart.read_byte() {
                return b;
            }
        }
        asm::nop();
    }
}

fn uart_read_u16_le() -> u16 {
    let b0 = uart_read_byte_blocking();
    let b1 = uart_read_byte_blocking();
    u16::from_le_bytes([b0, b1])
}

fn uart_read_u32_le() -> u32 {
    let b0 = uart_read_byte_blocking();
    let b1 = uart_read_byte_blocking();
    let b2 = uart_read_byte_blocking();
    let b3 = uart_read_byte_blocking();
    u32::from_le_bytes([b0, b1, b2, b3])
}

fn crc32(bytes: &[u8]) -> u32 {
    let crc = crc32_update(0xFFFF_FFFF, bytes);
    crc32_finish(crc)
}

fn crc32_update(state: u32, bytes: &[u8]) -> u32 {
    let mut crc = state;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = if (crc & 1) != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    crc
}

fn crc32_finish(crc: u32) -> u32 {
    crc ^ 0xFFFF_FFFF
}
