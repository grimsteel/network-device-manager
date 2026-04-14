<template>
  <div>
    <label class="block text-sm font-medium text-gray-700 mb-1">{{ label }}</label>
    <slot>
      <input
        v-bind="$attrs"
        :value="modelValue ?? ''"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        class="form-input"
      />
    </slot>
  </div>
</template>

<script setup lang="ts">
defineOptions({ inheritAttrs: false });

defineProps<{
  label: string;
  modelValue?: string | null;
}>();

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();
</script>
