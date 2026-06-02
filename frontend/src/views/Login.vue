<script setup>
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { api } from '../api'
import { auth } from '../store'
import { toast } from '../toast'

const router = useRouter()
const route = useRoute()
const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

async function submit() {
  error.value = ''
  loading.value = true
  try {
    const { token, user } = await api.login(username.value.trim(), password.value)
    auth.setSession(token, user)
    toast('登录成功')
    router.push(route.query.redirect || { name: 'home' })
  } catch (e) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="auth-wrap">
    <div class="auth-card panel">
      <h1>📦 DeployAnyFile</h1>
      <p class="sub">登录后即可上传与分享你的媒体文件</p>
      <form @submit.prevent="submit">
        <div class="field">
          <label>用户名</label>
          <input class="input" v-model="username" autocomplete="username" placeholder="请输入用户名" />
        </div>
        <div class="field">
          <label>密码</label>
          <input class="input" type="password" v-model="password" autocomplete="current-password" placeholder="请输入密码" />
        </div>
        <p v-if="error" class="error-text">{{ error }}</p>
        <button class="btn primary" style="width:100%; justify-content:center; margin-top:6px" :disabled="loading">
          {{ loading ? '登录中…' : '登录' }}
        </button>
      </form>
      <p class="muted" style="text-align:center; margin-top:16px">
        还没有账号？<router-link :to="{ name: 'register' }">注册</router-link>
      </p>
    </div>
  </div>
</template>
