import { auth } from './store'
import router from './router'

async function request(method, path, body, isForm = false) {
  const headers = {}
  if (auth.token) headers['Authorization'] = `Bearer ${auth.token}`

  let payload
  if (isForm) {
    payload = body // FormData
  } else if (body !== undefined) {
    headers['Content-Type'] = 'application/json'
    payload = JSON.stringify(body)
  }

  const res = await fetch(`/api${path}`, { method, headers, body: payload, cache: 'no-store' })

  if (res.status === 401) {
    auth.clear()
    if (router.currentRoute.value.name !== 'login') {
      router.push({ name: 'login' })
    }
    throw new Error('登录已过期，请重新登录')
  }

  const text = await res.text()
  const data = text ? JSON.parse(text) : {}
  if (!res.ok) {
    throw new Error(data.error || `请求失败 (${res.status})`)
  }
  return data
}

export const api = {
  // auth
  login: (username, password) => request('POST', '/auth/login', { username, password }),
  register: (username, password) => request('POST', '/auth/register', { username, password }),
  me: () => request('GET', '/auth/me'),
  changePassword: (old_password, new_password) =>
    request('POST', '/auth/change-password', { old_password, new_password }),

  // users (admin)
  listUsers: () => request('GET', '/users'),
  createUser: (username, password, is_admin) =>
    request('POST', '/users', { username, password, is_admin }),
  deleteUser: (id) => request('DELETE', `/users/${id}`),
  resetUserPassword: (id, new_password) =>
    request('POST', `/users/${id}/reset-password`, { new_password }),

  // files
  listFiles: (params) => {
    const q = new URLSearchParams()
    if (params.category) q.set('category', params.category)
    if (params.search) q.set('search', params.search)
    q.set('page', params.page)
    q.set('page_size', params.page_size)
    return request('GET', `/files?${q.toString()}`)
  },
  upload: (formData) => request('POST', '/files/upload', formData, true),
  deleteFiles: (ids) => request('DELETE', '/files', { ids }),
  setShareBulk: (ids, is_shared) => request('POST', '/files/share', { ids, is_shared }),
  setShareOne: (id, is_shared) => request('PATCH', `/files/${id}/share`, { is_shared }),
  updateSlug: (id, slug) => request('PATCH', `/files/${id}/slug`, { slug }),
  fileStats: (id) => request('GET', `/files/${id}/stats`),
  siteStats: () => request('GET', '/stats'),

  // public
  publicMeta: (slug) => request('GET', `/public/${slug}`),
}
