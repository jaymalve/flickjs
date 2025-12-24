<script setup lang="ts">
import { computed } from 'vue'
import DefaultTheme from 'vitepress/theme'
import CopyMarkdownButton from './components/CopyMarkdownButton.vue'
import { useData } from 'vitepress'

const { Layout } = DefaultTheme
const { page, frontmatter } = useData()

const showCopyButton = computed(() => {
  const layout = frontmatter.value.layout
  const isNotFound = page.value.isNotFound

  if (isNotFound) return false
  if (layout === 'home') return false
  if (layout === 'page') return false
  if (layout && layout !== 'doc') return false

  return true
})
</script>

<template>
  <Layout>
    <template #doc-before>
      <CopyMarkdownButton v-if="showCopyButton" />
    </template>
  </Layout>
</template>
