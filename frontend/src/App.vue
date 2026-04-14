<template>
  <div class="min-h-screen bg-gray-50">
    <NavBar />
    <main class="max-w-5xl mx-auto px-4 py-6">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import NavBar from './components/NavBar.vue';
</script>
