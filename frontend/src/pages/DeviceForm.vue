<template>
  <div class="max-w-xl">
    <h1 class="text-2xl font-semibold text-gray-800 mb-6">
      {{ isNew ? 'Add Device' : 'Edit Device' }}
    </h1>

    <form @submit.prevent="submit" class="bg-white rounded-lg border border-gray-200 p-6 space-y-4">
      <FormField label="Name" v-model="form.name" required />
      <FormField label="Description" v-model="form.description" />
      <FormField label="Network" v-model="form.network" placeholder="e.g. LAN" />
      <FormField label="MAC Address">
        <div class="flex gap-2">
          <input
            v-model="form.mac_address"
            required
            placeholder="AA:BB:CC:DD:EE:FF"
            class="form-input flex-1"
          />
          <button
            type="button"
            @click="showImport = true"
            class="px-3 py-2 text-sm border border-gray-300 rounded-md hover:bg-gray-50"
          >
            Import from pfSense
          </button>
        </div>
      </FormField>
      <FormField label="IP Address (optional)" v-model="form.ip_address" placeholder="192.168.1.x" />

      <div v-if="submitError" class="text-red-600 text-sm">{{ submitError }}</div>

      <div class="flex gap-3 pt-2">
        <button
          type="submit"
          :disabled="saving"
          class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm hover:bg-indigo-700 disabled:opacity-50"
        >
          {{ saving ? 'Saving…' : 'Save' }}
        </button>
        <RouterLink to="/devices" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">
          Cancel
        </RouterLink>
      </div>
    </form>

    <!-- pfSense import modal -->
    <div v-if="showImport" class="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl p-6 w-full max-w-lg">
        <h2 class="text-lg font-semibold mb-4">Import from pfSense DHCP</h2>
        <div class="space-y-3 mb-4">
          <FormField label="pfSense Base URL" v-model="pf.base_url" placeholder="https://192.168.1.1" />
          <FormField label="API Key" v-model="pf.api_key" type="password" />
          <button
            @click="fetchLeases"
            :disabled="pfLoading"
            class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm hover:bg-indigo-700 disabled:opacity-50"
          >
            {{ pfLoading ? 'Fetching…' : 'Fetch Leases' }}
          </button>
          <div v-if="pfError" class="text-red-600 text-sm">{{ pfError }}</div>
        </div>

        <div v-if="leases.length > 0" class="max-h-64 overflow-y-auto border border-gray-200 rounded">
          <button
            v-for="lease in leases"
            :key="lease.mac"
            @click="importLease(lease)"
            class="w-full text-left px-4 py-2 hover:bg-indigo-50 border-b border-gray-100 last:border-0"
          >
            <div class="font-mono text-sm">{{ lease.mac }}</div>
            <div class="text-xs text-gray-500">
              {{ lease.ip }} {{ lease.hostname ? `— ${lease.hostname}` : '' }}
            </div>
          </button>
        </div>

        <button @click="showImport = false" class="mt-4 text-sm text-gray-500 hover:text-gray-700">
          Close
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { api } from '../rspc';
import type { DhcpLease } from '../bindings';
import FormField from '../components/FormField.vue';

const router = useRouter();
const route = useRoute();

const id = route.params.id ? Number(route.params.id) : null;
const isNew = id === null;

const form = reactive({ name: '', description: '', network: '', mac_address: '', ip_address: '' });
const saving = ref(false);
const submitError = ref('');

const showImport = ref(false);
const pf = reactive({ base_url: '', api_key: '' });
const pfLoading = ref(false);
const pfError = ref('');
const leases = ref<DhcpLease[]>([]);

onMounted(async () => {
  if (!isNew) {
    const device = await api.devices.get(id!);
    form.name = device.name;
    form.description = device.description;
    form.network = device.network;
    form.mac_address = device.mac_address;
    form.ip_address = device.ip_address ?? '';
  }
});

async function submit() {
  saving.value = true;
  submitError.value = '';
  try {
    const payload = {
      id: id ?? 0,
      name: form.name,
      description: form.description,
      network: form.network,
      mac_address: form.mac_address,
      ip_address: form.ip_address || null,
    };
    if (isNew) {
      await api.devices.create(payload);
    } else {
      await api.devices.update(payload);
    }
    router.push('/devices');
  } catch (e: unknown) {
    submitError.value = e instanceof Error ? e.message : 'Save failed';
  } finally {
    saving.value = false;
  }
}

async function fetchLeases() {
  pfLoading.value = true;
  pfError.value = '';
  leases.value = [];
  try {
    leases.value = await api.pfsense.listDhcpLeases({ base_url: pf.base_url, api_key: pf.api_key });
  } catch (e: unknown) {
    pfError.value = e instanceof Error ? e.message : 'Failed to fetch leases';
  } finally {
    pfLoading.value = false;
  }
}

function importLease(lease: DhcpLease) {
  form.mac_address = lease.mac;
  form.ip_address = lease.ip;
  if (!form.name && lease.hostname) form.name = lease.hostname;
  showImport.value = false;
}
</script>
