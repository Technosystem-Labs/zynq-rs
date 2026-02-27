#!/usr/bin/env python3
import argparse
import binascii
import os
import serial
import struct
import subprocess
import sys
import time

MAGIC = b"SZ"
VERSION = 1
MAX_CHUNK = 1024

CMD_IDENTIFY = 0x00
CMD_LIST = 0x01
CMD_UPLOAD_BEGIN = 0x02
CMD_UPLOAD_CHUNK = 0x03
CMD_UPLOAD_END = 0x04
CMD_DOWNLOAD = 0x05
CMD_DELETE = 0x06
CMD_REBOOT = 0x07
CMD_MKDIR = 0x08
CMD_RMDIR = 0x09
CMD_FORMAT = 0x0A

FLAG_MORE = 0x01
READY_SENTINEL = b"SZL-SD READY v1; switching to binary protocol"
FW_ID = b"SZL-SD-SVC/1"
FIXED_BAUD = 1500000


class ProtoError(Exception):
    pass


def crc32(data: bytes) -> int:
    return binascii.crc32(data) & 0xFFFFFFFF


def read_until_ready(ser: serial.Serial, timeout: float):
    deadline = time.time() + timeout
    line = bytearray()
    while time.time() < deadline:
        b = ser.read(1)
        if not b:
            continue
        line.extend(b)
        if b == b"\n":
            text = line.decode("utf-8", errors="replace").rstrip("\r\n")
            print(text)
            if READY_SENTINEL.decode() in text:
                return True
            line.clear()
    return False


def write_frame(ser: serial.Serial, cmd: int, seq: int, payload: bytes):
    hdr = struct.pack("<BBHI", VERSION, cmd, seq, len(payload))
    crc = crc32(hdr + payload)
    frame = MAGIC + hdr + payload + struct.pack("<I", crc)
    ser.write(frame)
    ser.flush()


def read_exact(ser: serial.Serial, n: int) -> bytes:
    data = bytearray()
    while len(data) < n:
        chunk = ser.read(n - len(data))
        if not chunk:
            raise TimeoutError(f"Timed out reading {n} bytes")
        data.extend(chunk)
    return bytes(data)


def read_frame(ser: serial.Serial):
    while True:
        b0 = read_exact(ser, 1)
        if b0 != MAGIC[:1]:
            continue
        b1 = read_exact(ser, 1)
        if b1 != MAGIC[1:]:
            continue

        head = read_exact(ser, 8)
        version, cmd, seq, payload_len = struct.unpack("<BBHI", head)
        if version != VERSION:
            raise ProtoError(f"Unexpected protocol version: {version}")
        payload = read_exact(ser, payload_len)
        recv_crc, = struct.unpack("<I", read_exact(ser, 4))
        calc = crc32(head + payload)
        if recv_crc != calc:
            raise ProtoError("CRC mismatch")
        if len(payload) < 1:
            raise ProtoError("Empty response payload")
        status = payload[0]
        return cmd, seq, status, payload[1:]


def expect_status_ok(cmd, status):
    if status != 0:
        raise ProtoError(f"Command 0x{cmd:02x} failed, status={status}")


def identify_fw(ser, seq: int) -> bytes:
    write_frame(ser, CMD_IDENTIFY, seq, b"")
    rcmd, rseq, status, payload = read_frame(ser)
    if (rcmd, rseq) != (CMD_IDENTIFY, seq):
        raise ProtoError("Mismatched IDENTIFY response")
    expect_status_ok(rcmd, status)
    return payload


READY_DELAY = 4.0


def ensure_firmware(ser, seq: int) -> bytes:
    read_until_ready(ser, READY_DELAY)
    return identify_fw(ser, seq)


def _validate_83_component(component: str):
    """Validate a single 8.3 name component (already uppercased)."""
    if len(component) == 0 or len(component) > 12:
        raise ValueError(f"Name component {component!r} must be 1..12 chars")
    if " " in component or "\\" in component or ":" in component or ".." in component:
        raise ValueError(f"Name component {component!r} contains invalid characters")
    parts = component.split(".")
    if len(parts) > 2:
        raise ValueError(f"Name component {component!r} must be 8.3")
    base = parts[0]
    ext = parts[1] if len(parts) == 2 else ""
    if len(base) == 0 or len(base) > 8:
        raise ValueError(f"Base name {base!r} must be 1..8 chars")
    if ext and len(ext) > 3:
        raise ValueError(f"Extension {ext!r} must be 1..3 chars")
    allowed = set("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-")
    if not set(base).issubset(allowed):
        raise ValueError(f"Base {base!r} contains invalid characters")
    if ext and not set(ext).issubset(allowed):
        raise ValueError(f"Extension {ext!r} contains invalid characters")


def validate_path(path: str) -> str:
    """Validate a remote path (one or more 8.3 components separated by '/').

    Returns the uppercased path.
    """
    if not path:
        raise ValueError("Remote path must not be empty")
    up = path.upper()
    if up.startswith("/") or up.endswith("/") or "//" in up:
        raise ValueError("Remote path must not have leading/trailing/double slashes")
    if len(up) > 255:
        raise ValueError("Remote path too long (max 255 bytes)")
    for component in up.split("/"):
        _validate_83_component(component)
    return up


def cmd_list(ser, seq, path=""):
    if path:
        remote = validate_path(path)
        encoded = remote.encode("ascii")
        payload = bytes([len(encoded)]) + encoded
    else:
        payload = b""
    write_frame(ser, CMD_LIST, seq, payload)
    rcmd, rseq, status, payload = read_frame(ser)
    if rcmd != CMD_LIST or rseq != seq:
        raise ProtoError("Mismatched response")
    expect_status_ok(rcmd, status)
    if len(payload) < 2:
        raise ProtoError("LIST payload too short")
    count, = struct.unpack_from("<H", payload, 0)
    off = 2
    entries = []
    for _ in range(count):
        if off + 2 > len(payload):
            raise ProtoError("LIST malformed")
        type_byte = payload[off]
        nlen = payload[off + 1]
        off += 2
        if off + nlen + 4 > len(payload):
            raise ProtoError("LIST malformed")
        name = payload[off:off + nlen].decode("ascii", errors="replace")
        off += nlen
        size, = struct.unpack_from("<I", payload, off)
        off += 4
        entries.append((type_byte, name, size))
    for type_byte, name, size in entries:
        if type_byte == 0x01:
            print(f"{name}/")
        else:
            print(f"{name}\t{size}")


def cmd_upload(ser, seq, local_path, remote_name):
    remote = validate_path(remote_name)
    with open(local_path, "rb") as f:
        data = f.read()
    total_size = len(data)
    total_crc = crc32(data)

    payload = bytes([len(remote)]) + remote.encode("ascii") + struct.pack("<IB", total_size, 1)
    write_frame(ser, CMD_UPLOAD_BEGIN, seq, payload)
    rcmd, rseq, status, rp = read_frame(ser)
    if (rcmd, rseq) != (CMD_UPLOAD_BEGIN, seq):
        raise ProtoError("Mismatched UPLOAD_BEGIN response")
    expect_status_ok(rcmd, status)
    if len(rp) != 2:
        raise ProtoError("UPLOAD_BEGIN payload malformed")
    session_id, = struct.unpack("<H", rp)

    offset = 0
    start_time = time.time()

    def print_progress(done: int):
        elapsed = max(time.time() - start_time, 1e-6)
        speed = done / elapsed
        if total_size > 0:
            pct = (done * 100.0) / total_size
            remaining = max(total_size - done, 0)
            eta = remaining / speed if speed > 0 else 0.0
        else:
            pct = 100.0
            eta = 0.0
        print(
            f"\rUploading {remote}: {done}/{total_size} ({pct:5.1f}%)  "
            f"{speed/1024.0:7.1f} KiB/s  ETA {eta:5.1f}s",
            end="",
            flush=True,
        )

    while offset < total_size:
        chunk = data[offset:offset + MAX_CHUNK]
        payload = struct.pack("<HIH", session_id, offset, len(chunk)) + chunk
        write_frame(ser, CMD_UPLOAD_CHUNK, seq, payload)
        rcmd, rseq, status, rp = read_frame(ser)
        if (rcmd, rseq) != (CMD_UPLOAD_CHUNK, seq):
            raise ProtoError("Mismatched UPLOAD_CHUNK response")
        expect_status_ok(rcmd, status)
        if len(rp) != 4:
            raise ProtoError("UPLOAD_CHUNK payload malformed")
        next_off, = struct.unpack("<I", rp)
        if next_off != offset + len(chunk):
            raise ProtoError("Unexpected next offset")
        offset = next_off
        print_progress(offset)

    payload = struct.pack("<HI", session_id, total_crc)
    write_frame(ser, CMD_UPLOAD_END, seq, payload)
    rcmd, rseq, status, _ = read_frame(ser)
    if (rcmd, rseq) != (CMD_UPLOAD_END, seq):
        raise ProtoError("Mismatched UPLOAD_END response")
    expect_status_ok(rcmd, status)
    if total_size == 0:
        print_progress(0)
    print()
    print(f"Uploaded {total_size} bytes to {remote}")


def cmd_download(ser, seq, remote_name, local_path):
    remote = validate_path(remote_name)
    payload = bytes([len(remote)]) + remote.encode("ascii")
    write_frame(ser, CMD_DOWNLOAD, seq, payload)

    out = bytearray()
    expected_size = None
    expected_crc = None

    while True:
        rcmd, rseq, status, rp = read_frame(ser)
        if (rcmd, rseq) != (CMD_DOWNLOAD, seq):
            raise ProtoError("Mismatched DOWNLOAD response")
        expect_status_ok(rcmd, status)
        if len(rp) < 13:
            raise ProtoError("DOWNLOAD payload malformed")
        flags = rp[0]
        size, = struct.unpack_from("<I", rp, 1)
        file_crc, = struct.unpack_from("<I", rp, 5)
        offset, = struct.unpack_from("<I", rp, 9)
        data = rp[13:]

        if expected_size is None:
            expected_size = size
            expected_crc = file_crc

        if offset != len(out):
            raise ProtoError("Unexpected download offset")
        out.extend(data)

        if (flags & FLAG_MORE) == 0:
            break

    if expected_size != len(out):
        raise ProtoError(f"Size mismatch: expected {expected_size}, got {len(out)}")
    actual_crc = crc32(out)
    if actual_crc != expected_crc:
        raise ProtoError(f"CRC mismatch: expected 0x{expected_crc:08x}, got 0x{actual_crc:08x}")

    with open(local_path, "wb") as f:
        f.write(out)
    print(f"Downloaded {len(out)} bytes from {remote} to {local_path}")


def cmd_delete(ser, seq, remote_name):
    remote = validate_path(remote_name)
    payload = bytes([len(remote)]) + remote.encode("ascii")
    write_frame(ser, CMD_DELETE, seq, payload)
    rcmd, rseq, status, _ = read_frame(ser)
    if (rcmd, rseq) != (CMD_DELETE, seq):
        raise ProtoError("Mismatched DELETE response")
    expect_status_ok(rcmd, status)
    print(f"Deleted {remote}")


def cmd_reboot(ser, seq):
    write_frame(ser, CMD_REBOOT, seq, b"")
    rcmd, rseq, status, _ = read_frame(ser)
    if (rcmd, rseq) != (CMD_REBOOT, seq):
        raise ProtoError("Mismatched REBOOT response")
    expect_status_ok(rcmd, status)
    print("Reboot command acknowledged")


def cmd_mkdir(ser, seq, path):
    remote = validate_path(path)
    encoded = remote.encode("ascii")
    payload = bytes([len(encoded)]) + encoded
    write_frame(ser, CMD_MKDIR, seq, payload)
    rcmd, rseq, status, _ = read_frame(ser)
    if (rcmd, rseq) != (CMD_MKDIR, seq):
        raise ProtoError("Mismatched MKDIR response")
    expect_status_ok(rcmd, status)
    print(f"Created directory {remote}")


def cmd_rmdir(ser, seq, path):
    remote = validate_path(path)
    encoded = remote.encode("ascii")
    payload = bytes([len(encoded)]) + encoded
    write_frame(ser, CMD_RMDIR, seq, payload)
    rcmd, rseq, status, _ = read_frame(ser)
    if (rcmd, rseq) != (CMD_RMDIR, seq):
        raise ProtoError("Mismatched RMDIR response")
    expect_status_ok(rcmd, status)
    print(f"Removed directory {remote}")


def cmd_format(ser, seq, yes: bool):
    if not yes:
        answer = input("This will ERASE the entire SD card. Type 'yes' to confirm: ")
        if answer.strip().lower() != "yes":
            print("Aborted.")
            return
    write_frame(ser, CMD_FORMAT, seq, b"")
    rcmd, rseq, status, _ = read_frame(ser)
    if (rcmd, rseq) != (CMD_FORMAT, seq):
        raise ProtoError("Mismatched FORMAT response")
    expect_status_ok(rcmd, status)
    print("Format complete. SD card remounting...")


def cmd_load(args):
    elf = args.elf or os.environ.get("SZL_SDFS_DEFAULT_ELF")
    search_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "openocd")
    if "SZL_SDFS_OPENOCD_SEARCH_PATH" in os.environ:
        search_path = os.environ["SZL_SDFS_OPENOCD_SEARCH_PATH"]
    if not os.path.isdir(search_path):
        raise ValueError(f"OpenOCD search path {search_path} does not exist")
    if not elf or not os.path.isfile(elf):
        raise ValueError(
            "No ELF image for load. Either run via 'nix run .#szl-sdfs-cli load' "
            "(uses Nix-built artifact from szl-sdfs) or pass --elf /path/to/image.elf"
        )
    openocd_cmd = [
        "openocd",
        "-f",
        f"{args.target}.cfg",
        "-s",
        search_path,
        "-c",
        f"reset init; load_image {elf}; resume {args.resume_addr}; shutdown",
    ]
    subprocess.run(openocd_cmd, check=True)

    # Verify load succeeded by identifying the device over serial (device may reboot first)
    verify_timeout = 10.0
    with serial.Serial(args.port, FIXED_BAUD, timeout=1.0) as ser:
        if not read_until_ready(ser, verify_timeout):
            raise ProtoError("Device did not become ready after load (identify verification failed)")
        ident = identify_fw(ser, 1)
        if ident != FW_ID:
            raise ProtoError(f"Identify verification failed: unexpected firmware id {ident!r}")
    print("Load verified successfully (identify OK)")


def main():
    parser = argparse.ArgumentParser(description="SZL serial SD service client")
    parser.add_argument("--port", default="/dev/serial/by-id/usb-FTDI_Quad_RS232-HS-if02-port0", help="Serial port path (default: /dev/serial/by-id/usb-FTDI_Quad_RS232-HS-if02-port0)")

    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("identify")

    ls = sub.add_parser("list")
    ls.add_argument("path", nargs="?", default="", help="Remote directory path to list (default: root)")

    up = sub.add_parser("upload")
    up.add_argument("local")
    up.add_argument("remote")

    dl = sub.add_parser("download")
    dl.add_argument("remote")
    dl.add_argument("local")

    rm = sub.add_parser("delete")
    rm.add_argument("remote")

    sub.add_parser("reboot")

    mk = sub.add_parser("mkdir")
    mk.add_argument("path")

    rd = sub.add_parser("rmdir")
    rd.add_argument("path")

    fmt = sub.add_parser("format")
    fmt.add_argument("--yes", "-y", action="store_true", help="Skip confirmation prompt")

    ld = sub.add_parser("load")
    ld.add_argument("--target", default="kasli_soc", help="Target board")
    ld.add_argument("--elf", help="use custom ELF image")
    ld.add_argument("--resume-addr", default="0x20", help="Resume address")

    args = parser.parse_args()

    if args.cmd == "load":
        cmd_load(args)
        return

    if not args.port:
        parser.error("--port is required for serial commands")

    seq = 1
    with serial.Serial(args.port, FIXED_BAUD, timeout=1.0) as ser:
        ident = ensure_firmware(ser, seq)
        if ident != FW_ID:
            raise ProtoError(f"Unexpected firmware id: {ident!r}")

        if args.cmd == "identify":
            print(ident.decode("ascii", errors="replace"))
        elif args.cmd == "list":
            cmd_list(ser, seq, args.path)
        elif args.cmd == "upload":
            cmd_upload(ser, seq, args.local, args.remote)
        elif args.cmd == "download":
            cmd_download(ser, seq, args.remote, args.local)
        elif args.cmd == "delete":
            cmd_delete(ser, seq, args.remote)
        elif args.cmd == "reboot":
            cmd_reboot(ser, seq)
        elif args.cmd == "mkdir":
            cmd_mkdir(ser, seq, args.path)
        elif args.cmd == "rmdir":
            cmd_rmdir(ser, seq, args.path)
        elif args.cmd == "format":
            cmd_format(ser, seq, args.yes)
        else:
            raise ValueError("Unknown command")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
