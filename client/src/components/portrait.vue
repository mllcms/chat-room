<script setup lang="ts">
import { onMounted, ref } from 'vue'
const props = withDefaults(
  defineProps<{
    name: string
    width?: number
    height?: number
    radius?: string
    unread?: number
  }>(),
  {
    width: 40,
    height: 40,
    radius: '10%',
  }
)

const dom = ref<any>()
onMounted(() => {
  const canvas = dom.value
  const firstChar = props.name.charAt(0)
  canvas.width = props.width
  canvas.height = props.height
  const ctx = canvas.getContext('2d')
  ctx.fillStyle = '#409EFF'
  ctx.fillRect(0, 0, canvas.width, canvas.height)
  ctx.fillStyle = '#fff'
  ctx.font = `bold ${props.height / 2}px sans-serif`
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.fillText(firstChar, canvas.width / 2, canvas.height / 2)
})
</script>
<template>
  <div class="portrait">
    <canvas ref="dom" :style="{ borderRadius: radius }"></canvas>
    <div class="unread" v-if="unread">{{ unread < 99 ? unread : '99+' }}</div>
  </div>
</template>

<style>
.portrait {
  width: max-content;
  height: max-content;
  position: relative;
}

.portrait .unread {
  text-align: center;
  padding: 1px 6px;
  font-size: 13px;
  color: white;
  border-radius: 6px;
  background-color: red;
  position: absolute;
  top: 0;
  right: 0;
  transform: translate(50%, -30%);
}
</style>
