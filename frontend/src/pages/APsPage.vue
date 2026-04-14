<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-semibold text-gray-800">Access Points</h1>
      <RouterLink
        to="/access-points/new"
        class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm hover:bg-indigo-700"
      >
        Add Access Point
      </RouterLink>
    </div>

    <div v-if="loading" class="text-gray-500">Loading…</div>
    <div v-else-if="error" class="text-red-600">{{ error }}</div>
    <div v-else-if="aps.length === 0" class="text-gray-500">No access points yet.</div>
    <div v-else class="grid gap-3">
      <div
        v-for="ap in aps"
        :key="ap.id"
        class="bg-white rounded-lg border border-gray-200 px-4 py-3 flex items-center justify-between"
      >
        <div>
          <RouterLink
            :to="`/access-points/${ap.id}`"
            class="font-medium text-gray-800 hover:text-indigo-600"
          >
            {{ ap.name }}
          </RouterLink>
          <p class="text-sm text-gray-500 font-mono mt-0.5">{{ ap.host }}:{{ ap.port }}</p>
        </div>
        <button @click="remove(ap.id)" class="text-red-500 hover:text-red-700 text-sm">Delete</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { api } from '../rspc';
import type { AccessPoint } from '../bindings';

const aps = ref<AccessPoint[]>([]);
const loading = ref(true);
const error = ref('');

async function load() {
  loading.value = true;
  error.value = '';
  try {
    aps.value = await api.accessPoints.list();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load';
  } finally {
    loading.value = false;
  }
}

async function remove(id: number) {
  if (!confirm('Delete this access point?')) return;
  try {
    await api.accessPoints.delete(id);
    await load();
  } catch (e: unknown) {
    alert(e instanceof Error ? e.message : 'Delete failed');
  }
}

onMounted(load);
</script>
