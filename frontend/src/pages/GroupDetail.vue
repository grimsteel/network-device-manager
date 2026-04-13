<template>
  <div class="max-w-2xl">
    <h1 class="text-2xl font-semibold text-gray-800 mb-6">
      {{ isNew ? 'New Group' : 'Edit Group' }}
    </h1>

    <!-- Group metadata form -->
    <form @submit.prevent="saveGroup" class="bg-white rounded-lg border border-gray-200 p-6 space-y-4 mb-6">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Name</label>
        <input v-model="form.name" required class="input" />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Description</label>
        <input v-model="form.description" class="input" />
      </div>
      <div v-if="saveError" class="text-red-600 text-sm">{{ saveError }}</div>
      <div class="flex gap-3">
        <button type="submit" :disabled="saving" class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm hover:bg-indigo-700 disabled:opacity-50">
          {{ saving ? 'Saving…' : 'Save' }}
        </button>
        <RouterLink to="/groups" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">
          Back to Groups
        </RouterLink>
      </div>
    </form>

    <!-- Device membership (only for existing groups) -->
    <div v-if="!isNew && group" class="bg-white rounded-lg border border-gray-200 p-6">
      <h2 class="text-lg font-medium text-gray-800 mb-4">Devices in this Group</h2>

      <!-- Current members -->
      <div v-if="group.devices.length === 0" class="text-sm text-gray-500 mb-4">
        No devices assigned yet.
      </div>
      <ul v-else class="divide-y divide-gray-100 mb-4">
        <li
          v-for="d in group.devices"
          :key="d.id"
          class="flex items-center justify-between py-2"
        >
          <div>
            <span class="text-sm font-medium text-gray-800">{{ d.name }}</span>
            <span class="text-xs text-gray-400 font-mono ml-2">{{ d.mac_address }}</span>
          </div>
          <button
            @click="removeDevice(d.id)"
            class="text-red-500 hover:text-red-700 text-xs"
          >
            Remove
          </button>
        </li>
      </ul>

      <!-- Add device -->
      <div v-if="availableDevices.length > 0" class="flex gap-2">
        <select v-model="selectedDeviceId" class="input flex-1 text-sm">
          <option disabled value="">Select a device to add…</option>
          <option v-for="d in availableDevices" :key="d.id" :value="d.id">
            {{ d.name }} ({{ d.mac_address }})
          </option>
        </select>
        <button
          @click="addDevice"
          :disabled="!selectedDeviceId"
          class="bg-indigo-600 text-white px-3 py-2 rounded-md text-sm hover:bg-indigo-700 disabled:opacity-50"
        >
          Add
        </button>
      </div>
      <div v-else-if="allDevices.length > 0" class="text-sm text-gray-400">
        All devices are already in this group.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { api } from '../rspc';
import type { GroupWithDevices, Device } from '../bindings';

const router = useRouter();
const route = useRoute();

const id = route.params.id ? Number(route.params.id) : null;
const isNew = id === null;

const form = reactive({ name: '', description: '' });
const saving = ref(false);
const saveError = ref('');

const group = ref<GroupWithDevices | null>(null);
const allDevices = ref<Device[]>([]);
const selectedDeviceId = ref<number | ''>('');

const availableDevices = computed(() =>
  allDevices.value.filter(d => !group.value?.devices.some(gd => gd.id === d.id))
);

onMounted(async () => {
  allDevices.value = await api.devices.list();
  if (!isNew) {
    group.value = await api.groups.get(id!);
    form.name = group.value.name;
    form.description = group.value.description;
  }
});

async function saveGroup() {
  saving.value = true;
  saveError.value = '';
  try {
    if (isNew) {
      const created = await api.groups.create({ name: form.name, description: form.description });
      router.push(`/groups/${created.id}`);
    } else {
      await api.groups.update({ id: id!, name: form.name, description: form.description });
    }
  } catch (e: unknown) {
    saveError.value = e instanceof Error ? e.message : 'Save failed';
  } finally {
    saving.value = false;
  }
}

async function addDevice() {
  if (!selectedDeviceId.value) return;
  group.value = await api.groups.addDevice({
    group_id: id!,
    device_id: Number(selectedDeviceId.value),
  });
  selectedDeviceId.value = '';
}

async function removeDevice(deviceId: number) {
  group.value = await api.groups.removeDevice({ group_id: id!, device_id: deviceId });
}
</script>

<style scoped>
.input {
  @apply w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500;
}
</style>
