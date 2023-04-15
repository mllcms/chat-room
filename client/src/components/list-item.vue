<script setup lang="ts">
import { withDefaults } from 'vue';
import portrait from "@/components/portrait.vue"

withDefaults(defineProps<{
    name: string,
    width?: number
    height?: number
}>(), {
    width: 40,
    height: 40
})

</script>
<template>
    <div class="list-item">
        <portrait :name="name" />
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
    background-color: rgba(237, 238, 238, .8);
}

.list-item p {
    padding-left: 10px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>