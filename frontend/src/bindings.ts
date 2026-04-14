// Hand-written TypeScript types that mirror the Rust models in backend/src/models/

export interface Device {
  id: number;
  name: string;
  description: string;
  network: string;
  mac_address: string;
  ip_address: string | null;
}

export interface Group {
  id: number;
  name: string;
  description: string;
  devices: Device[];
}

export interface GroupDeviceInput {
  group_id: number;
  device_id: number;
}

export interface AccessPoint {
  id: number;
  name: string;
  host: string;
  port: number;
  interfaces: Interface[];
}

export interface Interface {
  id: number;
  ap_id: number;
  iface_name: string;
  group_id: number | null;
  last_synced_at: number | null;
  needs_sync: boolean;
}

export interface SyncResult {
  interface_id: number;
  success: boolean;
  message: string;
  mac_count: number;
}

export interface PfsenseConfig {
  base_url: string;
  api_key: string;
}

export interface DhcpLease {
  mac: string;
  ip: string;
  hostname: string;
  description: string;
}
