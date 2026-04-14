<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-semibold text-gray-800">Groups</h1>
      <RouterLink
        to="/groups/new"
        class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm hover:bg-indigo-700"
      >
        Add Group
      </RouterLink>
    </div>

    <div v-if="loading" class="text-gray-500">Loading…</div>
    <div v-else-if="error" class="text-red-600">{{ error }}</div>
    <div v-else-if="groups.length === 0" class="text-gray-500">No groups yet.</div>
    <div v-else class="grid gap-3">
      <div
        v-for="g in groups"
        :key="g.id"
        class="bg-white rounded-lg border border-gray-200 px-4 py-3 flex items-center justify-between"
      >
        <div>
          <RouterLink :to="`/groups/${g.id}`" class="font-medium text-gray-800 hover:text-indigo-600">
            {{ g.name }}
          </RouterLink>
          <p v-if="g.description" class="text-sm text-gray-500 mt-0.5">{{ g.description }}</p>
        </div>
        <button @click="remove(g.id)" class="text-red-500 hover:text-red-700 text-sm">Delete</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { api } from '../rspc';
import type { Group } from '../bindings';

const groups = ref<Group[]>([]);
const loading = ref(true);
const error = ref('');

async function load() {
  loading.value = true;
  error.value = '';
  try {
    groups.value = await api.groups.list();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load';
  } finally {
    loading.value = false;
  }
}

async function remove(id: number) {
  if (!confirm('Delete this group?')) return;
  try {
    await api.groups.delete(id);
    await load();
  } catch (e: unknown) {
    alert(e instanceof Error ? e.message : 'Delete failed');
  }
}

onMounted(load);
</script>
