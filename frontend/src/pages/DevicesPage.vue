<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-semibold text-gray-800">Devices</h1>
      <RouterLink
        to="/devices/new"
        class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm hover:bg-indigo-700"
      >
        Add Device
      </RouterLink>
    </div>

    <div v-if="loading" class="text-gray-500">Loading…</div>
    <div v-else-if="error" class="text-red-600">{{ error }}</div>
    <div v-else-if="devices.length === 0" class="text-gray-500">No devices yet.</div>
    <div v-else class="bg-white rounded-lg border border-gray-200 overflow-hidden">
      <table class="w-full text-sm">
        <thead class="bg-gray-50 text-left text-gray-500 uppercase text-xs">
          <tr>
            <th class="px-4 py-3">Name</th>
            <th class="px-4 py-3">Network</th>
            <th class="px-4 py-3">MAC Address</th>
            <th class="px-4 py-3">IP Address</th>
            <th class="px-4 py-3 text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="d in devices" :key="d.id" class="hover:bg-gray-50">
            <td class="px-4 py-3 font-medium text-gray-800">
              <RouterLink :to="`/devices/${d.id}`" class="hover:text-indigo-600">
                {{ d.name }}
              </RouterLink>
            </td>
            <td class="px-4 py-3 text-gray-600">{{ d.network || '—' }}</td>
            <td class="px-4 py-3 font-mono text-gray-600">{{ d.mac_address }}</td>
            <td class="px-4 py-3 text-gray-600">{{ d.ip_address || '—' }}</td>
            <td class="px-4 py-3 text-right">
              <button
                @click="remove(d.id)"
                class="text-red-500 hover:text-red-700 text-xs"
              >
                Delete
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { api } from '../rspc';
import type { Device } from '../bindings';

const devices = ref<Device[]>([]);
const loading = ref(true);
const error = ref('');

async function load() {
  loading.value = true;
  error.value = '';
  try {
    devices.value = await api.devices.list();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load devices';
  } finally {
    loading.value = false;
  }
}

async function remove(id: number) {
  if (!confirm('Delete this device?')) return;
  try {
    await api.devices.delete(id);
    await load();
  } catch (e: unknown) {
    alert(e instanceof Error ? e.message : 'Delete failed');
  }
}

onMounted(load);
</script>
