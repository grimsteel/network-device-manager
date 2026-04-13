# AP Daemon Binary Protocol

The backend sends MAC-address filter lists to each access point over a TCP socket.
The daemon running on the AP applies the list to the named wireless interface
(typically via `iptables` or the driver's MAC filter table).

---

## Transport

Plain TCP. The backend opens a short-lived connection to `host:port` (default **8765**),
sends a single message, and closes the connection.
The daemon does not send a reply (fire-and-forget).

---

## Message Format

All multi-byte integers are **big-endian**.

### Header (8 bytes, fixed)

| Offset | Size | Field         | Value |
|--------|------|---------------|-------|
| 0      | 2    | magic         | `0x4E 0x44` ("ND") |
| 2      | 1    | version       | `0x01` |
| 3      | 1    | msg\_type     | see table below |
| 4      | 4    | payload\_len  | `u32` – byte length of the payload |

### Message Types

| msg\_type | Name          | Direction   |
|-----------|---------------|-------------|
| `0x01`    | SetMacFilter  | server → AP |

### SetMacFilter Payload

| Field        | Type                    | Description |
|--------------|-------------------------|-------------|
| iface\_len   | `u8`                    | byte length of `iface_name` |
| iface\_name  | `u8[iface_len]`         | UTF-8 wireless interface name |
| mac\_count   | `u16`                   | number of MAC addresses |
| macs         | `[u8; 6][mac_count]`    | raw MAC addresses (each 6 bytes, no separators) |

### Full Message Layout

```
 0        1        2        3        4        5        6        7
┌────────────────┬────────┬──────────────────────────────────────┐
│  magic: 4E 44  │  ver   │ msg_type  │     payload_len (u32)    │
└────────────────┴────────┴──────────────────────────────────────┘
│ iface_len (u8) │ iface_name (N bytes)                           │
│ mac_count (u16)│ mac[0] (6 bytes)  │ mac[1] (6 bytes)  │ …    │
```

---

## Example

Set `wlan0` to allow exactly two devices (`AA:BB:CC:DD:EE:FF`, `11:22:33:44:55:66`):

```
4E 44 01 01          -- magic "ND", version 1, SetMacFilter
00 00 00 11          -- payload_len = 17
05                   -- iface_len = 5
77 6C 61 6E 30       -- "wlan0"
00 02                -- mac_count = 2
AA BB CC DD EE FF    -- MAC 1
11 22 33 44 55 66    -- MAC 2
```

---

## Daemon Implementation Notes

The daemon is **not included** in this repository. A conforming implementation should:

1. Listen on a configurable TCP port (default 8765).
2. Parse the 8-byte header; reject messages where `magic ≠ 0x4E 0x44` or `version > 0x01`.
3. For `SetMacFilter`, parse `iface_name` and the MAC list.
4. Apply the allowlist to the named interface (e.g., via `hostapd_cli` or `iw dev <iface> set mac_acl …`).
5. Close the connection.

Error handling: if the payload is malformed (e.g., truncated), the daemon should
log the error and close the connection without crashing.

---

## Versioning

The `version` byte allows future backward-compatible extensions.
Clients set `version = 0x01`; a daemon **must** accept any version ≤ its own.
New message types may be added without changing the version byte.
