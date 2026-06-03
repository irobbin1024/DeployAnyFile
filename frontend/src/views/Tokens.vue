<script setup>
import { ref, reactive, onMounted } from 'vue'
import { api } from '../api'
import { toast } from '../toast'
import { fmtDate } from '../format'
import Modal from '../components/Modal.vue'

const tokens = ref([])
const loading = ref(false)

const create = reactive({ open: false, name: '', error: '', creating: false })
const created = ref(null) // { name, token } shown once after creation
const del = reactive({ open: false, id: null, name: '' })

async function load() {
  loading.value = true
  try {
    tokens.value = await api.listTokens()
  } catch (e) {
    toast(e.message, 'err')
  } finally {
    loading.value = false
  }
}

async function submitCreate() {
  create.error = ''
  create.creating = true
  try {
    const res = await api.createToken(create.name.trim())
    create.open = false
    create.name = ''
    created.value = res // contains full token
    await load()
  } catch (e) {
    create.error = e.message
  } finally {
    create.creating = false
  }
}

async function copyToken(t) {
  try {
    await navigator.clipboard.writeText(t)
    toast('令牌已复制')
  } catch {
    toast('复制失败，请手动选择复制')
  }
}

function askDelete(t) {
  del.open = true
  del.id = t.id
  del.name = t.name
}
async function confirmDelete() {
  try {
    await api.deleteToken(del.id)
    del.open = false
    toast('令牌已吊销')
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
      <h2 style="margin:0; flex:1">API 令牌</h2>
      <button class="btn primary" @click="create.open = true">+ 新建令牌</button>
    </div>
    <p class="muted" style="margin-top:-4px">
      令牌用于脚本上传文件（仅限上传，无法删除文件或管理用户）。在请求头携带
      <code>Authorization: Bearer &lt;令牌&gt;</code> 即可。
    </p>

    <!-- one-time token reveal -->
    <div v-if="created" class="panel" style="border-color:var(--ok); margin-bottom:18px">
      <strong style="color:var(--ok)">✓ 令牌「{{ created.name }}」已创建</strong>
      <p class="muted" style="margin:6px 0">请立即复制保存，<b>关闭后将无法再次查看完整令牌</b>。</p>
      <div class="link-box">
        <code>{{ created.token }}</code>
        <button class="btn sm" @click="copyToken(created.token)">复制</button>
      </div>
      <button class="btn sm ghost" style="margin-top:10px" @click="created = null">我已保存，关闭</button>
    </div>

    <div v-if="loading" class="spin">加载中…</div>
    <div v-else-if="!tokens.length" class="empty">还没有令牌，点右上角新建一个。</div>
    <table v-else class="files-table">
      <thead>
        <tr>
          <th>名称</th>
          <th style="width:160px">令牌</th>
          <th style="width:160px">创建时间</th>
          <th style="width:160px">最近使用</th>
          <th style="width:90px">操作</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="t in tokens" :key="t.id">
          <td class="fname" data-label="名称">{{ t.name }}</td>
          <td class="muted" data-label="令牌"><code>{{ t.token_prefix }}</code></td>
          <td class="muted" data-label="创建时间">{{ fmtDate(t.created_at) }}</td>
          <td class="muted" data-label="最近使用">{{ t.last_used_at ? fmtDate(t.last_used_at) : '从未' }}</td>
          <td class="cell-actions" data-label="操作">
            <div class="row-actions">
              <button class="btn sm danger-text" @click="askDelete(t)">🗑️ 吊销</button>
            </div>
          </td>
        </tr>
      </tbody>
    </table>

    <!-- create -->
    <Modal v-if="create.open" title="新建 API 令牌" @close="create.open = false">
      <div class="field">
        <label>令牌名称</label>
        <input class="input" v-model="create.name" placeholder="例如：AI 上传脚本" @keyup.enter="submitCreate" />
      </div>
      <p class="muted">起个好记的名字，方便日后识别和吊销。</p>
      <p v-if="create.error" class="error-text">{{ create.error }}</p>
      <template #footer>
        <button class="btn" @click="create.open = false">取消</button>
        <button class="btn primary" :disabled="create.creating" @click="submitCreate">
          {{ create.creating ? '创建中…' : '创建' }}
        </button>
      </template>
    </Modal>

    <!-- delete confirm -->
    <Modal v-if="del.open" title="吊销令牌" @close="del.open = false">
      <p>确定吊销令牌 <strong>「{{ del.name }}」</strong> 吗？</p>
      <p class="muted">吊销后，使用该令牌的脚本将立即无法上传。此操作不可恢复。</p>
      <template #footer>
        <button class="btn" @click="del.open = false">取消</button>
        <button class="btn danger" @click="confirmDelete">吊销</button>
      </template>
    </Modal>
  </div>
</template>
