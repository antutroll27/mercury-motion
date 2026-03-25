<script setup lang="ts">
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
</script>

<template>
  <header class="h-12 bg-cosmos-card border-b border-cosmos-border flex items-center px-4 gap-6">
    <!-- Logo -->
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 rounded-full bg-crimson"></div>
      <span class="font-serif text-sm tracking-wide text-varden">Mercury Motion</span>
    </div>

    <!-- Timecode -->
    <div class="font-mono text-lg tracking-widest text-text-primary tabular-nums">
      {{ store.currentTimecode }}
    </div>

    <!-- Frame Counter -->
    <div class="font-mono text-xs text-text-muted uppercase tracking-widest">
      Frame {{ store.currentFrame }} / {{ store.totalFrames }}
    </div>

    <div class="flex-1" />

    <!-- Scene Info -->
    <div class="font-mono text-xs text-text-muted uppercase tracking-widest">
      {{ store.scene.meta.width }}&times;{{ store.scene.meta.height }} &middot; {{ store.scene.meta.fps }}fps
    </div>

    <!-- Export Button -->
    <button class="px-4 py-1.5 bg-crimson text-varden text-xs font-mono uppercase tracking-widest rounded hover:bg-gochujang transition-colors">
      Export
    </button>
  </header>
</template>
