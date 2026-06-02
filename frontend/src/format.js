export function fmtSize(bytes) {
  if (bytes == null) return '-'
  const units = ['B', 'KB', 'MB', 'GB']
  let n = bytes
  let i = 0
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024
    i++
  }
  return `${n.toFixed(i === 0 ? 0 : 1)} ${units[i]}`
}

export function fmtDate(iso) {
  if (!iso) return '-'
  const d = new Date(iso)
  if (isNaN(d)) return iso
  const p = (x) => String(x).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}

export const CATEGORIES = [
  { value: 'all', label: '全部类型' },
  { value: 'image', label: '图片' },
  { value: 'video', label: '视频' },
  { value: 'audio', label: '音频' },
  { value: 'html', label: 'HTML' },
  { value: 'markdown', label: 'Markdown' },
  { value: 'text', label: '文本' },
  { value: 'other', label: '其他' },
]

export function catLabel(v) {
  return (CATEGORIES.find((c) => c.value === v) || {}).label || v
}

export function shareUrl(slug) {
  return `${window.location.origin}/p/${slug}`
}
