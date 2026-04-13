<template>
  <span
    :class="[
      'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium',
      needsSync
        ? 'bg-amber-100 text-amber-700'
        : lastSyncedAt
          ? 'bg-green-100 text-green-700'
          : 'bg-gray-100 text-gray-500',
    ]"
  >
    <span
      :class="[
        'w-1.5 h-1.5 rounded-full mr-1',
        needsSync ? 'bg-amber-500' : lastSyncedAt ? 'bg-green-500' : 'bg-gray-400',
      ]"
    />
    {{ label }}
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  needsSync: boolean;
  lastSyncedAt: number | null;
}>();

const label = computed(() => {
  if (props.needsSync) return 'Needs sync';
  if (props.lastSyncedAt) {
    const d = new Date(props.lastSyncedAt * 1000);
    return `Synced ${d.toLocaleString()}`;
  }
  return 'Never synced';
});
</script>
