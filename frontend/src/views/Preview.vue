<script setup>
import { ref, reactive, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { marked } from 'marked'
import { api } from '../api'
import { auth } from '../store'
import { toast } from '../toast'
import { fmtSize, fmtDate, catLabel, shareUrl } from '../format'
import Modal from '../components/Modal.vue'

const route = useRoute()
const router = useRouter()

const meta = ref(null)
const loading = ref(true)
const error = ref('')
const panelOpen = ref(false)

const mediaSrc = ref('')
const textContent = ref('')
const mdHtml = ref('')

async function fetchRaw(url) {
  const headers = {}
  if (auth.token) headers['Authorization'] = `Bearer ${auth.token}`
  const res = await fetch(url, { headers })
  if (!res.ok) throw new Error('内容加载失败')
  return res
}

async function loadContent(m) {
  const cat = m.category
  if (cat === 'text') {
    textContent.value = await (await fetchRaw(m.raw_url)).text()
  } else if (cat === 'markdown') {
    mdHtml.value = marked.parse(await (await fetchRaw(m.raw_url)).text())
  } else if (['image', 'video', 'audio', 'html'].includes(cat)) {
    if (m.is_shared) {
      mediaSrc.value = m.raw_url
    } else {
      mediaSrc.value = URL.createObjectURL(await (await fetchRaw(m.raw_url)).blob())
    }
  }
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const m = await api.publicMeta(route.params.slug)
    meta.value = m
    await loadContent(m)
  } catch (e) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

const isOwner = computed(() => meta.value?.is_owner)

async function toggleShare() {
  try {
    const res = await api.setShareOne(meta.value.id, !meta.value.is_shared)
    meta.value.is_shared = res.is_shared
    toast(res.is_shared ? '已开启分享' : '已取消分享')
  } catch (e) {
    toast(e.message, 'err')
  }
}

const del = reactive({ open: false })
async function confirmDelete() {
  try {
    await api.deleteFiles([meta.value.id])
    del.open = false
    toast('已删除')
    router.push({ name: 'home' })
  } catch (e) {
    toast(e.message, 'err')
  }
}

const slugEdit = reactive({ open: false, value: '', error: '' })
function openSlug() {
  slugEdit.open = true
  slugEdit.value = meta.value.slug
  slugEdit.error = ''
}
async function saveSlug() {
  slugEdit.error = ''
  try {
    const res = await api.updateSlug(meta.value.id, slugEdit.value.trim())
    slugEdit.open = false
    toast('链接地址已更新')
    meta.value.slug = res.slug
    meta.value.raw_url = `/raw/${res.slug}`
    router.replace({ name: 'preview', params: { slug: res.slug } })
  } catch (e) {
    slugEdit.error = e.message
  }
}

const stats = ref(null)
const statsOpen = ref(false)
async function openStats() {
  statsOpen.value = true
  stats.value = null
  try {
    stats.value = await api.fileStats(meta.value.id)
  } catch (e) {
    toast(e.message, 'err')
    statsOpen.value = false
  }
}

async function copyLink() {
  try {
    await navigator.clipboard.writeText(shareUrl(meta.value.slug))
    toast('链接已复制')
  } catch {
    toast(shareUrl(meta.value.slug))
  }
}
function download() {
  window.open(meta.value.raw_url, '_blank')
}

onMounted(load)
</script>

<template>
  <div v-if="loading" class="spin">加载中…</div>

  <div v-else-if="error" class="preview-wrap">
    <div class="empty panel">
      <h2>😕 {{ error }}</h2>
      <p class="muted">该链接可能不存在，或分享已被关闭。</p>
      <router-link class="btn primary" :to="{ name: 'home' }" style="margin-top:12px">返回首页</router-link>
    </div>
  </div>

  <div v-else-if="meta" class="preview-page">
    <!-- Top: info + control panel (collapsible) -->
    <transition name="slide">
      <div v-show="panelOpen" class="preview-panel">
        <div class="preview-panel-inner">
          <div class="preview-head">
            <div>
              <h2 style="margin:0 0 4px">{{ meta.original_name }}</h2>
              <div class="muted" style="font-size:13px">
                <span class="tag" :class="meta.category">{{ catLabel(meta.category) }}</span>
                · {{ fmtSize(meta.size) }} · 上传于 {{ fmtDate(meta.created_at) }}
                <span v-if="meta.owner"> · 来自 {{ meta.owner }}</span>
              </div>
            </div>
            <div class="row-actions">
              <router-link class="btn sm" :to="{ name: 'home' }">← 我的文件</router-link>
              <button class="btn sm" @click="copyLink">🔗 复制链接</button>
              <button class="btn sm" @click="download">⬇ 下载</button>
            </div>
          </div>

          <div v-if="isOwner" class="owner-controls">
            <strong>管理面板</strong>
            <span :class="meta.is_shared ? 'badge-on' : 'badge-off'">
              {{ meta.is_shared ? '● 分享中' : '○ 已关闭' }}
            </span>
            <span class="spacer"></span>
            <button class="btn sm" @click="toggleShare">{{ meta.is_shared ? '取消分享' : '开启分享' }}</button>
            <button class="btn sm" @click="openSlug">✏️ 改链接</button>
            <button class="btn sm" @click="openStats">📊 分享数据</button>
            <button class="btn sm danger" @click="del.open = true">🗑️ 删除</button>
          </div>
        </div>
      </div>
    </transition>

    <!-- Handle (拉手) -->
    <div class="panel-handle" @click="panelOpen = !panelOpen" :title="panelOpen ? '收起信息面板' : '展开信息面板'">
      <span class="grip"></span>
    </div>

    <!-- Bottom: full preview area -->
    <div class="preview-stage">
      <img v-if="meta.category === 'image'" :src="mediaSrc" :alt="meta.original_name" />
      <video v-else-if="meta.category === 'video'" :src="mediaSrc" controls />
      <audio v-else-if="meta.category === 'audio'" :src="mediaSrc" controls />
      <iframe v-else-if="meta.category === 'html'" :src="mediaSrc" sandbox="allow-scripts allow-same-origin allow-popups allow-forms" />
      <div v-else-if="meta.category === 'markdown'" class="markdown-body" v-html="mdHtml"></div>
      <pre v-else-if="meta.category === 'text'" class="preview-text">{{ textContent }}</pre>
      <div v-else class="empty">
        <p>该文件类型不支持在线预览。</p>
        <button class="btn primary" @click="download">下载文件</button>
      </div>
    </div>

    <!-- Owner: edit slug -->
    <Modal v-if="slugEdit.open" title="修改链接地址" @close="slugEdit.open = false">
      <label>链接地址</label>
      <div class="link-box">
        <span class="muted">{{ shareUrl('') }}</span>
        <input class="input" v-model="slugEdit.value" style="flex:1" />
      </div>
      <p v-if="slugEdit.error" class="error-text">{{ slugEdit.error }}</p>
      <template #footer>
        <button class="btn" @click="slugEdit.open = false">取消</button>
        <button class="btn primary" @click="saveSlug">保存</button>
      </template>
    </Modal>

    <!-- Owner: delete confirm -->
    <Modal v-if="del.open" title="确认删除" @close="del.open = false">
      <p>确定要删除 <strong>「{{ meta.original_name }}」</strong> 吗？</p>
      <p class="muted">文件及其分享数据将被一并移除，此操作不可恢复。</p>
      <template #footer>
        <button class="btn" @click="del.open = false">取消</button>
        <button class="btn danger" @click="confirmDelete">删除</button>
      </template>
    </Modal>

    <!-- Owner: stats -->
    <Modal v-if="statsOpen" title="分享详情" @close="statsOpen = false">
      <div v-if="!stats" class="spin">加载中…</div>
      <template v-else>
        <div class="stat-grid">
          <div class="stat-box"><div class="num">{{ stats.total_views }}</div><div class="lbl">总浏览</div></div>
          <div class="stat-box"><div class="num">{{ stats.unique_ips }}</div><div class="lbl">独立 IP</div></div>
          <div class="stat-box"><div class="num">{{ stats.is_shared ? '开' : '关' }}</div><div class="lbl">分享状态</div></div>
        </div>
        <p class="muted">分享创建于：{{ fmtDate(stats.share_created_at) }}</p>
        <h4 style="margin:8px 0">访问记录</h4>
        <div v-if="!stats.visits.length" class="muted">暂无访问记录</div>
        <div v-else class="visit-list">
          <table>
            <thead><tr><th>IP</th><th>时间</th><th>设备</th></tr></thead>
            <tbody>
              <tr v-for="(v, i) in stats.visits" :key="i">
                <td>{{ v.ip }}</td>
                <td>{{ fmtDate(v.visited_at) }}</td>
                <td :title="v.user_agent" style="max-width:180px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap">{{ v.user_agent || '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </Modal>
  </div>
</template>
