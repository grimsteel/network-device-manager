/**
 * Typed fetch client for the rspc-axum backend.
 *
 * rspc-axum HTTP protocol:
 *   GET  /rspc/<path>?input=<json>   → query
 *   POST /rspc/<path>                → mutation, body: JSON input
 *
 * Response envelope:
 *   { jsonrpc: "2.0", id: null,
 *     result: { type: "response", data: <value> }
 *             | { type: "error",   data: { code, message } } }
 *
 * Convention: pass id: 0 when creating a new entity; the server ignores it
 * and returns the real database-assigned id in the response.
 */

import type {
  Device, Group, GroupDeviceInput,
  AccessPoint, Interface, SyncResult,
  PfsenseConfig, DhcpLease,
} from './bindings';

const BASE = '/rspc';

export class ApiError extends Error {
  code: number;
  constructor(code: number, message: string) {
    super(message);
    this.code = code;
    this.name = 'ApiError';
  }
}

async function query<T>(path: string, input?: unknown): Promise<T> {
  let url = `${BASE}/${path}`;
  if (input !== undefined) {
    url += `?input=${encodeURIComponent(JSON.stringify(input))}`;
  }
  const resp = await fetch(url);
  const body = await resp.json();
  if (body?.result?.type === 'error') {
    const err = body.result.data ?? {};
    throw new ApiError(err.code ?? 500, err.message ?? 'Unknown error');
  }
  return body?.result?.data as T;
}

async function mutate<T>(path: string, input?: unknown): Promise<T> {
  const resp = await fetch(`${BASE}/${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input ?? null),
  });
  const body = await resp.json();
  if (body?.result?.type === 'error') {
    const err = body.result.data ?? {};
    throw new ApiError(err.code ?? 500, err.message ?? 'Unknown error');
  }
  return body?.result?.data as T;
}

export const api = {
  devices: {
    list: (): Promise<Device[]> => query('devices.list'),
    get: (id: number): Promise<Device> => query('devices.get', id),
    create: (input: Device): Promise<Device> => mutate('devices.create', input),
    update: (input: Device): Promise<Device> => mutate('devices.update', input),
    delete: (id: number): Promise<void> => mutate('devices.delete', id),
  },

  groups: {
    list: (): Promise<Group[]> => query('groups.list'),
    get: (id: number): Promise<Group> => query('groups.get', id),
    create: (input: Group): Promise<Group> => mutate('groups.create', input),
    update: (input: Group): Promise<Group> => mutate('groups.update', input),
    delete: (id: number): Promise<void> => mutate('groups.delete', id),
    addDevice: (input: GroupDeviceInput): Promise<Group> => mutate('groups.addDevice', input),
    removeDevice: (input: GroupDeviceInput): Promise<Group> => mutate('groups.removeDevice', input),
  },

  accessPoints: {
    list: (): Promise<AccessPoint[]> => query('accessPoints.list'),
    get: (id: number): Promise<AccessPoint> => query('accessPoints.get', id),
    create: (input: AccessPoint): Promise<AccessPoint> => mutate('accessPoints.create', input),
    update: (input: AccessPoint): Promise<AccessPoint> => mutate('accessPoints.update', input),
    delete: (id: number): Promise<void> => mutate('accessPoints.delete', id),
    addInterface: (input: Interface): Promise<Interface> =>
      mutate('accessPoints.addInterface', input),
    updateInterface: (input: Interface): Promise<Interface> =>
      mutate('accessPoints.updateInterface', input),
    removeInterface: (id: number): Promise<void> =>
      mutate('accessPoints.removeInterface', id),
    syncInterface: (id: number): Promise<SyncResult> =>
      mutate('accessPoints.syncInterface', id),
  },

  pfsense: {
    listDhcpLeases: (cfg: PfsenseConfig): Promise<DhcpLease[]> =>
      mutate('pfsense.listDhcpLeases', cfg),
  },
};
