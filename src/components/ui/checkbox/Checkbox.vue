<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import { CheckboxRoot, type CheckboxRootProps } from "reka-ui"
import { cn } from "@/lib/utils"

interface Props extends CheckboxRootProps {
  class?: HTMLAttributes["class"]
}

const props = defineProps<Props>()
const emits = defineEmits<{
  "update:checked": [value: boolean]
}>()
</script>

<template>
  <CheckboxRoot
    data-slot="checkbox"
    :class="cn(
      'peer size-4 shrink-0 rounded-md border-2 border-subtle bg-elevated transition-all duration-120',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-bg-elevated',
      'disabled:cursor-not-allowed disabled:opacity-50',
      'data-[state=checked]:bg-accent data-[state=checked]:border-accent',
      'data-[state=checked]:text-primary',
      props.class
    )"
    :checked="props.checked"
    :default-checked="props.defaultChecked"
    :disabled="props.disabled"
    :name="props.name"
    :required="props.required"
    :value="props.value"
    :id="props.id"
    @update:checked="emits('update:checked', $event)"
  >
    <span 
      class="flex items-center justify-center text-current size-4 transition-transform duration-120"
      :class="{
        'scale-100': props.checked,
        'scale-0': !props.checked
      }"
    >
      <span class="icon-[lucide--check] size-3"></span>
    </span>
  </CheckboxRoot>
</template>