<script setup>
import { ref, reactive, onMounted, computed } from 'vue'
import { api } from '../api'
import { toast } from '../toast'
import { fmtSize, fmtDate, CATEGORIES, catLabel, shareUrl } from '../format'
import Modal from '../components/Modal.vue'

const list = reactive({ items: [], total: 0, page: 1, page_size: 12 })
const filters = reactive({ category: 'all', search: '' })
const loading = ref(false)
const site = ref(null)

const selected = ref(new Set())
const recentIds = ref(new Set()) // files just uploaded this session
const fileInput = ref(null)
const dragOver = ref(false)
const uploading = ref(false)
const useCustomSlug = ref(false)
const customSlug = ref('')

const totalPages = computed(() => Math.max(1, Math.ceil(list.total / list.page_size)))
const allChecked = computed(
  () => list.items.length > 0 && list.items.every((f) => selected.value.has(f.id))
)

// Recently-uploaded files float to the top as a highlighted group, separated by a gap.
const displayRows = computed(() => {
  const recent = list.items.filter((f) => recentIds.value.has(f.id))
  const others = list.items.filter((f) => !recentIds.value.has(f.id))
  const rows = recent.map((f) => ({ f, recent: true }))
  if (recent.length && others.length) rows.push({ gap: true })
  others.forEach((f) => rows.push({ f, recent: false }))
  return rows
})

async function load() {
  loading.value = true
  try {
    const data = await api.listFiles({
      category: filters.category === 'all' ? '' : filters.category,
      search: filters.search.trim(),
      page: list.page,
      page_size: list.page_size,
    })
    list.items = data.items
    list.total = data.total
    selected.value = new Set()
    loadStats()
  } catch (e) {
    toast(e.message, 'err')
  } finally {
    loading.value = false
  }
}

async function loadStats() {
  try {
    site.value = await api.siteStats()
  } catch (e) {
    /* non-critical */
  }
}

function doSearch() {
  list.page = 1
  load()
}
function goPage(p) {
  if (p < 1 || p > totalPages.value) return
  list.page = p
  load()
}

// ---- selection ----
function toggle(id) {
  const s = new Set(selected.value)
  s.has(id) ? s.delete(id) : s.add(id)
  selected.value = s
}
function toggleAll() {
  selected.value = allChecked.value ? new Set() : new Set(list.items.map((f) => f.id))
}
const selectedIds = computed(() => [...selected.value])

// ---- upload ----
function pickFile() {
  fileInput.value.click()
}
function onFileChange(e) {
  const f = e.target.files[0]
  if (f) uploadFile(f)
  e.target.value = ''
}
function onDrop(e) {
  dragOver.value = false
  const f = e.dataTransfer.files[0]
  if (f) uploadFile(f)
}
async function uploadFile(file) {
  if (uploading.value) return
  uploading.value = true
  try {
    const fd = new FormData()
    if (useCustomSlug.value && customSlug.value.trim()) fd.append('slug', customSlug.value.trim())
    fd.append('file', file)
    const res = await api.upload(fd)
    toast('上传成功，链接已复制到剪贴板')
    customSlug.value = ''
    navigator.clipboard?.writeText(shareUrl(res.slug)).catch(() => {})
    filters.category = 'all'
    filters.search = ''
    list.page = 1
    await load()
    recentIds.value = new Set([...recentIds.value, res.id])
  } catch (e) {
    toast(e.message, 'err')
  } finally {
    uploading.value = false
  }
}

// ---- row actions ----
async function copyLink(slug) {
  try {
    await navigator.clipboard.writeText(shareUrl(slug))
    toast('链接已复制')
  } catch {
    toast(shareUrl(slug))
  }
}
async function bulkShare(isShared) {
  if (!selectedIds.value.length) return
  try {
    await api.setShareBulk(selectedIds.value, isShared)
    toast(isShared ? '已开启分享' : '已取消分享')
    await load()
  } catch (e) {
    toast(e.message, 'err')
  }
}
async function toggleShareOne(f) {
  try {
    await api.setShareOne(f.id, !f.is_shared)
    f.is_shared = !f.is_shared
    toast(f.is_shared ? '已开启分享' : '已取消分享')
    loadStats()
  } catch (e) {
    toast(e.message, 'err')
  }
}

// ---- delete (modal confirm) ----
const del = reactive({ open: false, ids: [], label: '' })
function askDeleteOne(f) {
  del.ids = [f.id]
  del.label = `「${f.original_name}」`
  del.open = true
}
function askDeleteBulk() {
  if (!selectedIds.value.length) return
  del.ids = [...selectedIds.value]
  del.label = `选中的 ${selectedIds.value.length} 个文件`
  del.open = true
}
async function confirmDelete() {
  try {
    const count = del.ids.length
    await api.deleteFiles(del.ids)
    del.open = false
    toast('已删除')
    if (list.items.length === count && list.page > 1) list.page--
    await load()
  } catch (e) {
    toast(e.message, 'err')
  }
}

// ---- stats modal ----
const stats = ref(null)
const statsOpen = ref(false)
async function openStats(f) {
  statsOpen.value = true
  stats.value = null
  try {
    stats.value = await api.fileStats(f.id)
  } catch (e) {
    toast(e.message, 'err')
    statsOpen.value = false
  }
}

// ---- edit slug modal ----
const slugEdit = reactive({ open: false, id: null, value: '', error: '' })
function openSlugEdit(f) {
  slugEdit.open = true
  slugEdit.id = f.id
  slugEdit.value = f.slug
  slugEdit.error = ''
}
async function saveSlug() {
  slugEdit.error = ''
  try {
    const res = await api.updateSlug(slugEdit.id, slugEdit.value.trim())
    const item = list.items.find((x) => x.id === slugEdit.id)
    if (item) item.slug = res.slug
    slugEdit.open = false
    toast('链接地址已更新')
  } catch (e) {
    slugEdit.error = e.message
  }
}

onMounted(load)
</script>

<template>
  <div class="container">
    <!-- Upload -->
    <input ref="fileInput" type="file" hidden @change="onFileChange" />
    <div
      class="upload-zone"
      :class="{ drag: dragOver }"
      @click="pickFile"
      @dragover.prevent="dragOver = true"
      @dragleave.prevent="dragOver = false"
      @drop.prevent="onDrop"
    >
      <div class="icon">
        <svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 5v14M5 12l7-7 7 7" />
        </svg>
      </div>
      <h2>{{ uploading ? '上传中…' : '点击或拖拽文件到此处上传' }}</h2>
      <p>支持图片、音视频、HTML、Markdown、文本等媒体文件</p>
    </div>

    <div style="margin-top:12px" @click.stop>
      <label style="display:flex; align-items:center; gap:8px; font-weight:400; cursor:pointer">
        <input type="checkbox" v-model="useCustomSlug" />
        自定义链接地址（短链）
      </label>
      <div v-if="useCustomSlug" class="link-box" style="margin-top:8px; max-width:460px">
        <span class="muted">{{ shareUrl('') }}</span>
        <input class="input" v-model="customSlug" placeholder="my-link" style="flex:1" />
      </div>
    </div>

    <!-- Toolbar -->
    <div class="toolbar">
      <select class="select" v-model="filters.category" @change="doSearch">
        <option v-for="c in CATEGORIES" :key="c.value" :value="c.value">{{ c.label }}</option>
      </select>
      <input class="input grow" v-model="filters.search" placeholder="搜索文件名…" @keyup.enter="doSearch" />
      <button class="btn" @click="doSearch">搜索</button>
    </div>

    <!-- Bulk actions -->
    <div v-if="selectedIds.length" class="toolbar" style="margin-top:0">
      <span class="muted">已选 {{ selectedIds.length }} 项</span>
      <button class="btn sm" @click="bulkShare(false)">取消分享</button>
      <button class="btn sm" @click="bulkShare(true)">开启分享</button>
      <button class="btn sm danger" @click="askDeleteBulk">删除</button>
    </div>

    <!-- File table -->
    <div v-if="loading" class="spin">加载中…</div>
    <div v-else-if="!list.items.length" class="empty">还没有文件，上传一个试试吧 👆</div>
    <table v-else class="files-table">
      <thead>
        <tr>
          <th style="width:36px"><input type="checkbox" :checked="allChecked" @change="toggleAll" /></th>
          <th>文件名</th>
          <th style="width:86px">类型</th>
          <th style="width:84px">大小</th>
          <th style="width:76px">状态</th>
          <th style="width:60px">浏览</th>
          <th style="width:130px">上传时间</th>
          <th style="width:280px">操作</th>
        </tr>
      </thead>
      <tbody>
        <template v-for="row in displayRows" :key="row.gap ? 'gap' : row.f.id">
          <tr v-if="row.gap" class="gap-row"><td colspan="8"></td></tr>
          <tr v-else :class="{ 'just-uploaded': row.recent }">
            <td><input type="checkbox" :checked="selected.has(row.f.id)" @change="toggle(row.f.id)" /></td>
            <td>
              <router-link class="fname-link" :to="{ name: 'preview', params: { slug: row.f.slug } }">
                <span class="fname" :title="row.f.original_name">{{ row.f.original_name }}</span>
              </router-link>
              <span v-if="row.recent" class="new-badge">刚上传</span>
              <div class="muted" style="font-size:12px">/p/{{ row.f.slug }}</div>
            </td>
            <td><span class="tag" :class="row.f.category">{{ catLabel(row.f.category) }}</span></td>
            <td class="muted">{{ fmtSize(row.f.size) }}</td>
            <td>
              <span :class="row.f.is_shared ? 'badge-on' : 'badge-off'">
                {{ row.f.is_shared ? '分享中' : '已关闭' }}
              </span>
            </td>
            <td class="muted">{{ row.f.view_count }}</td>
            <td class="muted">{{ fmtDate(row.f.created_at) }}</td>
            <td>
              <div class="row-actions">
                <button class="btn sm ghost" @click="copyLink(row.f.slug)">🔗 复制链接</button>
                <button class="btn sm ghost" @click="openStats(row.f)">📊 分享详情</button>
                <button class="btn sm ghost" @click="openSlugEdit(row.f)">✏️ 改链接</button>
                <button class="btn sm ghost" @click="toggleShareOne(row.f)">
                  {{ row.f.is_shared ? '🚫 取消分享' : '✅ 开启分享' }}
                </button>
                <button class="btn sm ghost danger-text" @click="askDeleteOne(row.f)">🗑️ 删除</button>
              </div>
            </td>
          </tr>
        </template>
      </tbody>
    </table>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="pager">
      <button class="btn sm" :disabled="list.page <= 1" @click="goPage(list.page - 1)">上一页</button>
      <span class="muted">第 {{ list.page }} / {{ totalPages }} 页 · 共 {{ list.total }} 项</span>
      <button class="btn sm" :disabled="list.page >= totalPages" @click="goPage(list.page + 1)">下一页</button>
    </div>

    <!-- Site stats -->
    <div v-if="site" class="site-stats">
      <h3 class="site-stats-title">全站统计</h3>
      <div class="stat-grid">
        <div class="stat-box"><div class="num">{{ site.total_files }}</div><div class="lbl">文件总数</div></div>
        <div class="stat-box"><div class="num">{{ site.shared_files }}</div><div class="lbl">分享中</div></div>
        <div class="stat-box"><div class="num">{{ fmtSize(site.total_size) }}</div><div class="lbl">内容总大小</div></div>
        <div class="stat-box"><div class="num">{{ site.total_views }}</div><div class="lbl">总浏览次数</div></div>
        <div class="stat-box"><div class="num">{{ site.total_users }}</div><div class="lbl">注册用户</div></div>
      </div>
    </div>

    <!-- Delete confirm -->
    <Modal v-if="del.open" title="确认删除" @close="del.open = false">
      <p>确定要删除 <strong>{{ del.label }}</strong> 吗？</p>
      <p class="muted">文件及其分享数据将被一并移除，此操作不可恢复。</p>
      <template #footer>
        <button class="btn" @click="del.open = false">取消</button>
        <button class="btn danger" @click="confirmDelete">删除</button>
      </template>
    </Modal>

    <!-- Stats modal -->
    <Modal v-if="statsOpen" title="分享详情" @close="statsOpen = false">
      <div v-if="!stats" class="spin">加载中…</div>
      <template v-else>
        <div class="stat-grid">
          <div class="stat-box"><div class="num">{{ stats.total_views }}</div><div class="lbl">总浏览</div></div>
          <div class="stat-box"><div class="num">{{ stats.unique_ips }}</div><div class="lbl">独立 IP</div></div>
          <div class="stat-box"><div class="num">{{ stats.is_shared ? '开' : '关' }}</div><div class="lbl">分享状态</div></div>
        </div>
        <p class="muted">分享创建于：{{ fmtDate(stats.share_created_at) }}</p>
        <div class="link-box" style="margin-bottom:14px">
          <code>{{ shareUrl(stats.slug) }}</code>
          <button class="btn sm" @click="copyLink(stats.slug)">复制</button>
        </div>
        <h4 style="margin:0 0 8px">访问记录</h4>
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

    <!-- Edit slug modal -->
    <Modal v-if="slugEdit.open" title="修改链接地址" @close="slugEdit.open = false">
      <label>链接地址</label>
      <div class="link-box">
        <span class="muted">{{ shareUrl('') }}</span>
        <input class="input" v-model="slugEdit.value" style="flex:1" />
      </div>
      <p class="muted" style="margin-top:8px">只能包含字母、数字、'-' 和 '_'，需保持唯一。</p>
      <p v-if="slugEdit.error" class="error-text">{{ slugEdit.error }}</p>
      <template #footer>
        <button class="btn" @click="slugEdit.open = false">取消</button>
        <button class="btn primary" @click="saveSlug">保存</button>
      </template>
    </Modal>
  </div>
</template>
