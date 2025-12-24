<script setup lang="ts">
import { ref, computed } from 'vue'
import { useData } from 'vitepress'

const { page } = useData()
const copied = ref(false)
let copyTimeout: ReturnType<typeof setTimeout> | null = null

const rawMarkdown = computed(() => (page.value as any).rawMarkdown)

const shouldShow = computed(() => {
  return rawMarkdown.value && rawMarkdown.value.length > 0
})

async function copyToClipboard() {
  if (!rawMarkdown.value) return

  try {
    await navigator.clipboard.writeText(rawMarkdown.value)
    copied.value = true

    if (copyTimeout) {
      clearTimeout(copyTimeout)
    }

    copyTimeout = setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    fallbackCopy(rawMarkdown.value)
  }
}

function fallbackCopy(text: string) {
  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.style.position = 'fixed'
  textarea.style.opacity = '0'
  document.body.appendChild(textarea)
  textarea.select()

  try {
    document.execCommand('copy')
    copied.value = true
    copyTimeout = setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }

  document.body.removeChild(textarea)
}
</script>

<template>
  <button
    v-if="shouldShow"
    class="copy-markdown-btn"
    :class="{ copied }"
    @click="copyToClipboard"
    :title="copied ? 'Copied!' : 'Copy page as Markdown'"
    :aria-label="copied ? 'Copied to clipboard' : 'Copy page as Markdown'"
  >
    <svg
      v-if="!copied"
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
    </svg>

    <svg
      v-else
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <polyline points="20 6 9 17 4 12"></polyline>
    </svg>

    <span class="copy-markdown-text">
      {{ copied ? 'Copied!' : 'Copy as Markdown' }}
    </span>
  </button>
</template>

<style scoped>
.copy-markdown-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid var(--vp-c-border);
  border-radius: 6px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-2);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-bottom: 16px;
}

.copy-markdown-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
}

.copy-markdown-btn.copied {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
}

.copy-markdown-btn:focus-visible {
  outline: 2px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.copy-markdown-text {
  line-height: 1;
}

@media (max-width: 640px) {
  .copy-markdown-text {
    display: none;
  }

  .copy-markdown-btn {
    padding: 8px;
  }
}
</style>
