<script setup lang="ts">
import type { MsgCache } from '@/types';
import portrait from "@/components/portrait.vue"
import { nextTick, onMounted, ref, watch } from 'vue';

const props = defineProps<{
    msgs: MsgCache[]
}>()
const msgWindow = ref<HTMLElement>()
watch(() => props.msgs, () => {
    nextTick(() => {
        msgWindow.value?.scrollTo(0, msgWindow.value.scrollHeight)
    })
});
watch(() => props.msgs.length, () => {
    const dom = msgWindow.value
    if (!dom || dom.scrollHeight - dom.scrollTop > 700) {
        return
    }
    nextTick(() => {
        msgWindow.value?.scrollTo(0, msgWindow.value.scrollHeight)
    })
})

</script>
<template>
    <div class="msg-window" ref="msgWindow">
        <template v-for="item in msgs">
            <div class="msg-left" v-if="item.position == 'left'">
                <portrait :name="item.target.name" class="portrait" />
                <small>{{ item.tiem }}</small>
                <p class="msg-p">{{ item.msg }}</p>
            </div>
            <div class="msg-center" v-if="item.position == 'center'">
                <small>系统消息：{{ item.msg }}</small>
            </div>
            <div class="msg-right" v-if="item.position == 'right'">
                <small>{{ item.tiem }}</small>
                <portrait :name="item.target.name" class="portrait" />
                <p class="msg-p">{{ item.msg }}</p>
            </div>
        </template>
    </div>
</template>
<style>
.msg-window {
    display: flex;
    flex-flow: column;
    row-gap: 14px;
    width: 100%;
    height: 100%;
    overflow: auto;
    padding: 0 10px 30px;
    box-sizing: border-box;
}

.msg-window::-webkit-scrollbar {
    width: 6px;
}

.msg-window::-webkit-scrollbar-thumb {
    background: rgba(175, 180, 185, 0.8);
    border-radius: 4px;
}


.msg-window small {
    font-size: 13px;
    color: #8F9AA7;
}

.msg-left {
    display: grid;
    grid-template-rows: 20px 1fr;
    grid-template-columns: 50px 1fr;
}

.msg-center {
    display: flex;
    justify-content: center;
}

.msg-right {
    display: grid;
    grid-template-rows: 20px 1fr;
    grid-template-columns: 1fr 50px;
    margin-left: auto;
    justify-items: end;
}

.msg-p {
    width: fit-content;
    max-width: 500px;
    background-color: #F9D5B7;
    padding: 12px;
    border-radius: 6px;
    word-wrap: break-word;
}

.portrait {
    grid-row: span 2;
    margin-top: 22px;
}
</style>