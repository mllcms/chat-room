<script setup lang="ts">
import { ref, watch, withDefaults } from 'vue'
import portrait from '@/components/portrait.vue'

const props = withDefaults(
    defineProps<{
        name: string
        unread: number
        width?: number
        height?: number
    }>(),
    {
        width: 40,
        height: 40,
    }
)

const unread = ref(0)
let timer = 0;
watch(() => props.unread, nv => {
    clearTimeout(timer)
    timer = setTimeout(() => {
        unread.value = nv
    }, 100)
})

</script>
<template>
    <div class="list-item">
        <portrait :name="name" :unread="unread" />
        <p>{{ name }}</p>
    </div>
</template>
<style>
.list-item {
    display: flex;
    width: 100%;
    height: 100%;
    align-items: center;
    box-sizing: border-box;
    border-radius: 4px;
    padding: 6px 10px;
}

.list-item:hover {
    cursor: pointer;
    background-color: rgba(237, 238, 238, 0.8);
}

.list-item p {
    padding-left: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
