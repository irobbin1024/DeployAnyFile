<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '../api'
import { auth } from '../store'
import { toast } from '../toast'

const router = useRouter()
const username = ref('')
const password = ref('')
const password2 = ref('')
const error = ref('')
const loading = ref(false)

async function submit() {
  error.value = ''
  if (password.value !== password2.value) {
    error.value = '两次输入的密码不一致'
    return
  }
  loading.value = true
  try {
    const { token, user } = await api.register(username.value.trim(), password.value)
    auth.setSession(token, user)
    toast('注册成功')
    router.push({ name: 'home' })
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
      <h1>创建账号</h1>
      <p class="sub">用户名 3-32 位，密码至少 6 位</p>
      <form @submit.prevent="submit">
        <div class="field">
          <label>用户名</label>
          <input class="input" v-model="username" placeholder="字母、数字、_ 或 -" />
        </div>
        <div class="field">
          <label>密码</label>
          <input class="input" type="password" v-model="password" placeholder="至少 6 位" />
        </div>
        <div class="field">
          <label>确认密码</label>
          <input class="input" type="password" v-model="password2" placeholder="再次输入密码" />
        </div>
        <p v-if="error" class="error-text">{{ error }}</p>
        <button class="btn primary" style="width:100%; justify-content:center; margin-top:6px" :disabled="loading">
          {{ loading ? '注册中…' : '注册' }}
        </button>
      </form>
      <p class="muted" style="text-align:center; margin-top:16px">
        已有账号？<router-link :to="{ name: 'login' }">登录</router-link>
      </p>
    </div>
  </div>
</template>
