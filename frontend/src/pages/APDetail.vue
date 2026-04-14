<template>
  <div class="max-w-2xl">
    <h1 class="text-2xl font-semibold text-gray-800 mb-6">
      {{ isNew ? 'New Access Point' : 'Edit Access Point' }}
    </h1>

    <!-- AP metadata form -->
    <form @submit.prevent="saveAP" class="bg-white rounded-lg border border-gray-200 p-6 space-y-4 mb-6">
      <FormField label="Name" v-model="form.name" required />
      <FormField label="Host" v-model="form.host" required placeholder="192.168.1.10" />
      <FormField label="Port">
        <input v-model.number="form.port" type="number" min="1" max="65535" class="form-input" />
      </FormField>
      <div v-if="saveError" class="text-red-600 text-sm">{{ saveError }}</div>
      <div class="flex gap-3">
        <button
          type="submit"
          :disabled="saving"
          class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm hover:bg-indigo-700 disabled:opacity-50"
        >
          {{ saving ? 'Saving…' : 'Save' }}
        </button>
        <RouterLink to="/access-points" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">
          Back
        </RouterLink>
      </div>
    </form>

    <!-- Interfaces (existing AP only) -->
    <div v-if="!isNew && ap" class="bg-white rounded-lg border border-gray-200 p-6">
      <h2 class="text-lg font-medium text-gray-800 mb-4">Interfaces</h2>

      <div v-if="ap.interfaces.length === 0" class="text-sm text-gray-500 mb-4">
        No interfaces configured yet.
      </div>
      <div v-else class="space-y-3 mb-6">
        <div
          v-for="iface in ap.interfaces"
          :key="iface.id"
          class="border border-gray-200 rounded-lg p-4"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 space-y-2">
              <div class="flex items-center gap-3">
                <input
                  v-model="editForms[iface.id].iface_name"
                  class="form-input text-sm font-mono w-40"
                  placeholder="wlan0"
                />
                <select v-model="editForms[iface.id].group_id" class="form-input text-sm flex-1">
                  <option :value="null">No group</option>
                  <option v-for="g in groups" :key="g.id" :value="g.id">{{ g.name }}</option>
                </select>
                <button
                  @click="saveIface(iface.id)"
                  class="px-3 py-1.5 text-xs bg-gray-100 hover:bg-gray-200 rounded"
                >
                  Save
                </button>
              </div>
              <div class="flex items-center gap-3">
                <SyncBadge :needs-sync="iface.needs_sync" :last-synced-at="iface.last_synced_at" />
                <button
                  @click="sync(iface.id)"
                  :disabled="syncing === iface.id"
                  class="text-xs text-indigo-600 hover:text-indigo-800 disabled:opacity-50"
                >
                  {{ syncing === iface.id ? 'Syncing…' : 'Sync Now' }}
                </button>
                <span
                  v-if="syncResults[iface.id]"
                  :class="syncResults[iface.id]!.success ? 'text-green-600' : 'text-red-600'"
                  class="text-xs"
                >
                  {{ syncResults[iface.id]!.message }}
                </span>
              </div>
            </div>
            <button
              @click="removeIface(iface.id)"
              class="text-red-500 hover:text-red-700 text-xs mt-1"
            >
              Remove
            </button>
          </div>
        </div>
      </div>

      <!-- Add new interface -->
      <div class="border-t border-gray-100 pt-4">
        <h3 class="text-sm font-medium text-gray-700 mb-3">Add Interface</h3>
        <div class="flex gap-2">
          <input v-model="newIface.iface_name" placeholder="wlan0" class="form-input text-sm w-32" />
          <select v-model="newIface.group_id" class="form-input text-sm flex-1">
            <option :value="null">No group</option>
            <option v-for="g in groups" :key="g.id" :value="g.id">{{ g.name }}</option>
          </select>
          <button
            @click="addIface"
            :disabled="!newIface.iface_name"
            class="bg-indigo-600 text-white px-3 py-2 rounded-md text-sm hover:bg-indigo-700 disabled:opacity-50"
          >
            Add
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { api } from '../rspc';
import type { AccessPoint, Group, SyncResult } from '../bindings';
import FormField from '../components/FormField.vue';
import SyncBadge from '../components/SyncBadge.vue';

const router = useRouter();
const route = useRoute();

const id = route.params.id ? Number(route.params.id) : null;
const isNew = id === null;

const form = reactive({ name: '', host: '', port: 8765 });
const saving = ref(false);
const saveError = ref('');

const ap = ref<AccessPoint | null>(null);
const groups = ref<Group[]>([]);

// Per-interface inline edit forms keyed by interface id
const editForms = reactive<Record<number, { iface_name: string; group_id: number | null }>>({});

const newIface = reactive({ iface_name: '', group_id: null as number | null });
const syncing = ref<number | null>(null);
const syncResults = reactive<Record<number, SyncResult | undefined>>({});

onMounted(async () => {
  groups.value = await api.groups.list();
  if (!isNew) {
    ap.value = await api.accessPoints.get(id!);
    form.name = ap.value.name;
    form.host = ap.value.host;
    form.port = ap.value.port;
    rebuildEditForms();
  }
});

watch(ap, () => rebuildEditForms());

function rebuildEditForms() {
  for (const iface of ap.value?.interfaces ?? []) {
    if (!editForms[iface.id]) {
      editForms[iface.id] = { iface_name: iface.iface_name, group_id: iface.group_id };
    }
  }
}

async function saveAP() {
  saving.value = true;
  saveError.value = '';
  try {
    if (isNew) {
      const created = await api.accessPoints.create({ id: 0, name: form.name, host: form.host, port: form.port, interfaces: [] });
      router.push(`/access-points/${created.id}`);
    } else {
      await api.accessPoints.update({ id: id!, name: form.name, host: form.host, port: form.port, interfaces: [] });
    }
  } catch (e: unknown) {
    saveError.value = e instanceof Error ? e.message : 'Save failed';
  } finally {
    saving.value = false;
  }
}

async function addIface() {
  if (!newIface.iface_name) return;
  await api.accessPoints.addInterface({
    id: 0,
    ap_id: id!,
    iface_name: newIface.iface_name,
    group_id: newIface.group_id,
    last_synced_at: null,
    needs_sync: false,
  });
  ap.value = await api.accessPoints.get(id!);
  newIface.iface_name = '';
  newIface.group_id = null;
}

async function saveIface(ifaceId: number) {
  const ef = editForms[ifaceId];
  const current = ap.value!.interfaces.find(i => i.id === ifaceId)!;
  await api.accessPoints.updateInterface({
    id: ifaceId,
    ap_id: current.ap_id,
    iface_name: ef.iface_name,
    group_id: ef.group_id,
    last_synced_at: current.last_synced_at,
    needs_sync: current.needs_sync,
  });
  ap.value = await api.accessPoints.get(id!);
}

async function removeIface(ifaceId: number) {
  if (!confirm('Remove this interface?')) return;
  await api.accessPoints.removeInterface(ifaceId);
  ap.value = await api.accessPoints.get(id!);
}

async function sync(ifaceId: number) {
  syncing.value = ifaceId;
  try {
    const result = await api.accessPoints.syncInterface(ifaceId);
    syncResults[ifaceId] = result;
    ap.value = await api.accessPoints.get(id!);
  } catch (e: unknown) {
    syncResults[ifaceId] = {
      interface_id: ifaceId,
      success: false,
      message: e instanceof Error ? e.message : 'Sync failed',
      mac_count: 0,
    };
  } finally {
    syncing.value = null;
  }
}
</script>
