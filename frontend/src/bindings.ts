// Hand-written TypeScript types that mirror the Rust models in backend/src/models/

export interface Device {
  id: number;
  name: string;
  description: string;
  network: string;
  mac_address: string;
  ip_address: string | null;
}

export interface CreateDevice {
  name: string;
  description: string;
  network: string;
  mac_address: string;
  ip_address: string | null;
}

export interface UpdateDevice {
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
}

export interface GroupWithDevices {
  id: number;
  name: string;
  description: string;
  devices: Device[];
}

export interface CreateGroup {
  name: string;
  description: string;
}

export interface UpdateGroup {
  id: number;
  name: string;
  description: string;
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
}

export interface Interface {
  id: number;
  ap_id: number;
  iface_name: string;
  group_id: number | null;
  last_synced_at: number | null;
  needs_sync: boolean;
}

export interface AccessPointWithInterfaces {
  id: number;
  name: string;
  host: string;
  port: number;
  interfaces: Interface[];
}

export interface CreateAccessPoint {
  name: string;
  host: string;
  port: number;
}

export interface UpdateAccessPoint {
  id: number;
  name: string;
  host: string;
  port: number;
}

export interface CreateInterface {
  ap_id: number;
  iface_name: string;
  group_id: number | null;
}

export interface UpdateInterface {
  id: number;
  iface_name: string;
  group_id: number | null;
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
