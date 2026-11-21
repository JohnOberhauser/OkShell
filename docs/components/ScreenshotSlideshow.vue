<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { withBase } from 'vitepress'

const props = withDefaults(
  defineProps<{
    images: { src: string; alt?: string }[]
    interval?: number
  }>(),
  { interval: 4000 }
)

const index = ref(0)
const paused = ref(false)
let timer: ReturnType<typeof setInterval> | undefined

const count = computed(() => props.images.length)
const trackStyle = computed(() => ({
  transform: `translateX(-${index.value * 100}%)`,
}))

function go(i: number) {
  index.value = (i + count.value) % count.value
}
const next = () => go(index.value + 1)
const prev = () => go(index.value - 1)

function start() {
  stop()
  timer = setInterval(() => {
    if (!paused.value && !document.hidden) next()
  }, props.interval)
}
function stop() {
  if (timer) clearInterval(timer)
  timer = undefined
}

onMounted(() => {
  // Respect users who've asked for less motion: no autoplay, manual controls still work.
  if (!window.matchMedia('(prefers-reduced-motion: reduce)').matches) start()
})
onBeforeUnmount(stop)
</script>

<template>
  <div
    class="slideshow"
    @mouseenter="paused = true"
    @mouseleave="paused = false"
    @focusin="paused = true"
    @focusout="paused = false"
  >
    <div class="viewport">
      <div class="track" :style="trackStyle">
        <img
          v-for="(img, i) in images"
          :key="img.src"
          :src="withBase(img.src)"
          :alt="img.alt ?? `Screenshot ${i + 1}`"
          :loading="i === 0 ? 'eager' : 'lazy'"
          class="slide"
        />
      </div>

      <button class="nav prev" aria-label="Previous screenshot" @click="prev">‹</button>
      <button class="nav next" aria-label="Next screenshot" @click="next">›</button>
    </div>

    <div class="dots">
      <button
        v-for="(_, i) in images"
        :key="i"
        :class="['dot', { active: i === index }]"
        :aria-label="`Go to screenshot ${i + 1}`"
        @click="go(i)"
      />
    </div>
  </div>
</template>

<style scoped>
.slideshow {
  width: 100%;
  margin-top: 2rem;
}

.viewport {
  position: relative;
  overflow: hidden;
  border-radius: 8px;
  background: var(--vp-c-bg-soft);
}

.track {
  display: flex;
  transition: transform 0.6s ease;
}

.slide {
  flex: 0 0 100%;
  width: 100%;
  aspect-ratio: 16 / 9;
  object-fit: cover;
  display: block;
}

.nav {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: none;
  background: rgba(0, 0, 0, 0.45);
  color: #fff;
  font-size: 1.6rem;
  line-height: 1;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s;
}
.viewport:hover .nav,
.nav:focus-visible {
  opacity: 1;
}
.prev { left: 12px; }
.next { right: 12px; }

.dots {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-top: 12px;
}
.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  border: none;
  padding: 0;
  background: var(--vp-c-divider);
  cursor: pointer;
  transition: background 0.2s, transform 0.2s;
}
.dot.active {
  background: var(--vp-c-brand-1);
  transform: scale(1.3);
}

@media (prefers-reduced-motion: reduce) {
  .track { transition: none; }
}
</style>
