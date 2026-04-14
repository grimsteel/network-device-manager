<template>
  <nav class="bg-white border-b border-gray-200 shadow-sm">
    <div class="max-w-5xl mx-auto px-4 flex items-center gap-6 h-14">
      <span class="font-semibold text-gray-800">Network Device Manager</span>
      <RouterLink
        v-for="link in links"
        :key="link.to"
        :to="link.to"
        class="text-sm text-gray-600 hover:text-gray-900 transition-colors"
        active-class="text-indigo-600 font-medium"
      >
        {{ link.label }}
      </RouterLink>
    </div>
  </nav>
</template>

<script setup lang="ts">
const links = [
  { to: '/devices', label: 'Devices' },
  { to: '/groups', label: 'Groups' },
  { to: '/access-points', label: 'Access Points' },
];
</script>
