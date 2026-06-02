<script setup>
import { useRouter, useRoute } from 'vue-router'
import { computed, reactive } from 'vue'
import { auth } from './store'
import { toast, toastState } from './toast'
import { api } from './api'
import Modal from './components/Modal.vue'

const router = useRouter()
const route = useRoute()

// Hide the top bar on auth pages and the public preview page.
const bareLayout = computed(() =>
  ['login', 'register', 'preview'].includes(route.name)
)

const pw = reactive({ open: false, old: '', neo: '', error: '' })
async function changePw() {
  pw.error = ''
  try {
    await api.changePassword(pw.old, pw.neo)
    pw.open = false
    pw.old = ''
    pw.neo = ''
    toast('密码已修改')
  } catch (e) {
    pw.error = e.message
  }
}

function logout() {
  auth.clear()
  router.push({ name: 'login' })
}
</script>

<template>
  <header v-if="!bareLayout" class="topbar">
    <span class="brand">📦 DeployAnyFile</span>
    <router-link class="nav-link" :to="{ name: 'home' }">我的文件</router-link>
    <router-link v-if="auth.isAdmin" class="nav-link" :to="{ name: 'admin' }">用户管理</router-link>
    <span class="spacer"></span>
    <span class="muted">{{ auth.user?.username }}<span v-if="auth.isAdmin"> · 管理员</span></span>
    <button class="btn sm" @click="pw.open = true">修改密码</button>
    <button class="btn sm" @click="logout">退出</button>
  </header>

  <router-view />

  <!-- Change password (available on every page via the top bar) -->
  <Modal v-if="pw.open" title="修改密码" @close="pw.open = false">
    <div class="field"><label>原密码</label><input class="input" type="password" v-model="pw.old" /></div>
    <div class="field"><label>新密码</label><input class="input" type="password" v-model="pw.neo" placeholder="至少 6 位" /></div>
    <p v-if="pw.error" class="error-text">{{ pw.error }}</p>
    <template #footer>
      <button class="btn" @click="pw.open = false">取消</button>
      <button class="btn primary" @click="changePw">保存</button>
    </template>
  </Modal>

  <transition name="fade">
    <div v-if="toastState.msg" class="toast" :class="{ err: toastState.type === 'err' }">
      {{ toastState.msg }}
    </div>
  </transition>
</template>

<style>
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
