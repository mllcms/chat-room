<script setup lang="ts">
import { onMounted, ref } from 'vue';
const props = withDefaults(defineProps<{
    name: string,
    width?: number
    height?: number
    radius?: string
}>(), {
    width: 40,
    height: 40,
    radius: "50%"
})

const dom = ref<any>();
onMounted(() => {
    const canvas = dom.value
    const firstChar = props.name.charAt(0);
    canvas.width = props.width;
    canvas.height = props.height;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#409EFF';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = '#fff';
    ctx.font = `bold ${props.height / 2}px sans-serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(firstChar, canvas.width / 2, canvas.height / 2);
})
</script>
<template>
    <canvas ref="dom" :style="{ borderRadius: radius }"></canvas>
</template>