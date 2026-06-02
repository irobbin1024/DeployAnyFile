<script setup>
import { ref, reactive, onMounted } from 'vue'
import { api } from '../api'
import { auth } from '../store'
import { toast } from '../toast'
import { fmtDate } from '../format'
import Modal from '../components/Modal.vue'

const users = ref([])
const loading = ref(false)

const create = reactive({ open: false, username: '', password: '', is_admin: false, error: '' })
const reset = reactive({ open: false, id: null, username: '', password: '', error: '' })

async function load() {
  loading.value = true
  try {
    users.value = await api.listUsers()
  } catch (e) {
    toast(e.message, 'err')
  } finally {
    loading.value = false
  }
}

async function submitCreate() {
  create.error = ''
  try {
    await api.createUser(create.username.trim(), create.password, create.is_admin)
    create.open = false
    create.username = ''
    create.password = ''
    create.is_admin = false
    toast('用户已创建')
    await load()
  } catch (e) {
    create.error = e.message
  }
}

function openReset(u) {
  reset.open = true
  reset.id = u.id
  reset.username = u.username
  reset.password = ''
  reset.error = ''
}
async function submitReset() {
  reset.error = ''
  try {
    await api.resetUserPassword(reset.id, reset.password)
    reset.open = false
    toast('密码已重置')
  } catch (e) {
    reset.error = e.message
  }
}

async function removeUser(u) {
  if (!confirm(`确认删除用户「${u.username}」？其所有文件也会被删除。`)) return
  try {
    await api.deleteUser(u.id)
    toast('用户已删除')
    await load()
  } catch (e) {
    toast(e.message, 'err')
  }
}

onMounted(load)
</script>

<template>
  <div class="container">
    <div class="toolbar" style="margin-top:0">
      <h2 style="margin:0; flex:1">用户管理</h2>
      <button class="btn primary" @click="create.open = true">+ 新建用户</button>
    </div>

    <div v-if="loading" class="spin">加载中…</div>
    <table v-else class="files-table">
      <thead>
        <tr>
          <th style="width:60px">ID</th>
          <th>用户名</th>
          <th style="width:100px">角色</th>
          <th style="width:90px">文件数</th>
          <th style="width:160px">注册时间</th>
          <th style="width:180px">操作</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="u in users" :key="u.id">
          <td class="muted" data-label="ID">{{ u.id }}</td>
          <td class="fname" data-label="用户名">{{ u.username }}</td>
          <td data-label="角色">
            <span class="tag" :class="u.is_admin ? 'html' : 'text'">{{ u.is_admin ? '管理员' : '普通用户' }}</span>
          </td>
          <td class="muted" data-label="文件数">{{ u.file_count }}</td>
          <td class="muted" data-label="注册时间">{{ fmtDate(u.created_at) }}</td>
          <td class="cell-actions" data-label="操作">
            <div class="row-actions">
              <button class="btn sm" @click="openReset(u)">重置密码</button>
              <button
                class="btn sm danger"
                :disabled="u.id === auth.user?.id"
                :title="u.id === auth.user?.id ? '不能删除自己' : '删除'"
                @click="removeUser(u)"
              >删除</button>
            </div>
          </td>
        </tr>
      </tbody>
    </table>

    <!-- Create user -->
    <Modal v-if="create.open" title="新建用户" @close="create.open = false">
      <div class="field"><label>用户名</label><input class="input" v-model="create.username" placeholder="3-32 位" /></div>
      <div class="field"><label>密码</label><input class="input" type="password" v-model="create.password" placeholder="至少 6 位" /></div>
      <label style="display:flex; align-items:center; gap:8px; font-weight:400; cursor:pointer">
        <input type="checkbox" v-model="create.is_admin" /> 设为管理员
      </label>
      <p v-if="create.error" class="error-text">{{ create.error }}</p>
      <template #footer>
        <button class="btn" @click="create.open = false">取消</button>
        <button class="btn primary" @click="submitCreate">创建</button>
      </template>
    </Modal>

    <!-- Reset password -->
    <Modal v-if="reset.open" :title="`重置「${reset.username}」的密码`" @close="reset.open = false">
      <div class="field"><label>新密码</label><input class="input" type="password" v-model="reset.password" placeholder="至少 6 位" /></div>
      <p v-if="reset.error" class="error-text">{{ reset.error }}</p>
      <template #footer>
        <button class="btn" @click="reset.open = false">取消</button>
        <button class="btn primary" @click="submitReset">重置</button>
      </template>
    </Modal>
  </div>
</template>
