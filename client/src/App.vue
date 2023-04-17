<script setup lang="ts">
import { computed, h, reactive, ref } from "vue";
import { uuid } from "./utils";
import { ElMessage, ElNotification } from "element-plus";
import listItem from "@/components/list-item.vue";
import { Search } from "@element-plus/icons-vue";
import { createData, createMsgCache, type Data, type Message, type MsgCache, type User } from "@/types"
import msgWindow from "@/components/msg-window.vue"

const show = ref(true);
const user = reactive<User>({
  id: uuid(),
  name: "",
});
const search = ref<string>("");
const list = ref<User[]>([]);
const listShow = computed(() => {
  if (search) {
    return list.value.filter((item) =>
      item.name.toLowerCase().includes(search.value.toLowerCase())
    );
  }
  return list.value;
});

let websocket: WebSocket | null = null;
const login = async () => {
  if (!user.name) {
    ElMessage.error("用户名不能为空");
    return;
  }

  if (!show.value) return
  show.value = false;

  init_websocket()
};

function init_websocket() {
  websocket = new WebSocket(`ws://${window.location.hostname}/ws`);
  websocket.onopen = function () {
    this.send(JSON.stringify(createData("login", "", user)));
  };

  websocket.onmessage = function (event) {
    const data: Data = JSON.parse(event.data);
    const pub = cache["pubilc"] ||= []

    switch (data.type) {
      case "error":
        ElMessage.error(data.msg);
        break;
      case "login":
      case "logout":
      case "system": {
        ElNotification({
          title: "系统消息",
          message: h("span", { style: "color: #337ecc" }, data.msg),
        });
        if (data.list) {
          list.value = data.list;
        }
        if (data.type == "logout") {
          Reflect.deleteProperty(cache, data.target.id)
        }
        pub.push(createMsgCache("center", data.target, data.msg))
        break
      }
      case "public": {
        if (data.target.id != user.id) {
          pub.push(createMsgCache("left", data.target, data.msg))
        }
        break
      }
      case "private":
        const arr = cache[data.target.id] ||= []
        arr.push(createMsgCache("left", data.target, data.msg))
        break
      default: {
        console.log(event.data);
      }
    }
  };
  websocket.onerror = function () {
    if (this.readyState == this.CLOSED) return
    ElMessage.error("系统错误!!!请尝试刷新");
  };
  websocket.onclose = function () {
    ElMessage.error("与服务器断开连接!!!请尝试刷新")
  }
}

const pub = reactive<User>({
  id: "pubilc",
  name: "公共聊天室",
});
const msg = reactive<Message>({
  type: "public",
  target: pub,
  msg: "",
});

function change(user: User) {
  if (user == pub) {
    msg.type = "public";
  } else {
    msg.type = "private";
  }
  msg.target = user;
}

const cache = reactive<Record<string, MsgCache[]>>({});
function send() {
  if (!msg.msg) return;
  if (!websocket) return ElMessage.error("没有 websocket 连接")

  // 连接断开
  if (websocket.readyState == websocket.CLOSED) {
    init_websocket()
    return ElMessage.warning("服务重连中 请稍后")
  }

  const arr = cache[msg.target.id] ||= []
  arr.push(createMsgCache("right", user, msg.msg))

  websocket.send(JSON.stringify(msg))
  msg.msg = "";
}
</script>

<template>
  <el-dialog v-model="show" title="用户登录" width="max(20vw, 300px)" center :show-close="false" :close-on-click-modal="false"
    :close-on-press-escape="false">
    <el-input v-model="user.name" placeholder="用户名" @keyup.enter="login" />
    <template #footer>
      <span class="dialog-footer">
        <el-button type="primary" @click="login"> 加入聊天室 </el-button>
      </span>
    </template>
  </el-dialog>
  <div v-if="!show" class="main-container">
    <!-- 搜索 -->
    <div class="search">
      <el-input v-model="search" class="w-50 m-2" placeholder="搜索用户" :prefix-icon="Search" />
    </div>
    <!-- 标题 -->
    <div class="title">
      <h3>{{ msg.target.name }}</h3>
    </div>
    <!-- 用户列表 -->
    <div class="list">
      <list-item :name="pub.name" :class="{ active: msg.target == pub }" @click="change(pub)" />
      <list-item v-for="user in listShow" :name="user.name" :class="{ active: msg.target == user }" @click="change(user)"
        :key="user.id" />
    </div>
    <!-- 消息窗口 -->
    <div class="window">
      <msg-window :msgs="cache[msg.target.id] || []" />
      <div class="input">
        <el-input v-model="msg.msg" @keyup.enter="send" :placeholder="`发送到 ${msg.target.name}`" />
        <el-button type="primary" @click="send">发送</el-button>
      </div>
    </div>
  </div>
</template>
<style>
* {
  padding: 0;
  margin: 0;
}

#app {
  width: 100vw;
  height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  overflow: hidden;
}

.active {
  background-color: rgba(230, 240, 255, 0.9);
}

.main-container {
  width: 1000px;
  height: 700px;
  display: grid;
  grid-template-columns: 200px 1fr;
  grid-template-rows: 40px 1fr;
  border-radius: 10px;
  box-shadow: 0px 0px 10px #a2a2a2;
  overflow: hidden;
}

.search {
  display: flex;
  padding: 6px 6px;
  background-color: pink;
}

.title {
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #29395a;
  color: #fff;
}

.list {
  display: grid;
  grid-template-rows: repeat(auto-fill, minmax(60px, 1fr));
  background-color: pink;
  padding: 0 6px;
  gap: 5px;
  overflow: auto;
}

.window {
  width: 100%;
  display: grid;
  padding: 10px 0;
  background-color: #fafbfc;
  grid-template-rows: 1fr;
  overflow: auto;
  box-sizing: border-box;
}

.input {
  display: flex;
  gap: 10px;
  margin-top: 10px;
  padding: 0 10px;
}
</style>
