import { reactive } from 'vue'

const saved = localStorage.getItem('user')

export const auth = reactive({
  token: localStorage.getItem('token') || '',
  user: saved ? JSON.parse(saved) : null,

  setSession(token, user) {
    this.token = token
    this.user = user
    localStorage.setItem('token', token)
    localStorage.setItem('user', JSON.stringify(user))
  },

  clear() {
    this.token = ''
    this.user = null
    localStorage.removeItem('token')
    localStorage.removeItem('user')
  },

  get isLoggedIn() {
    return !!this.token
  },
  get isAdmin() {
    return !!(this.user && this.user.is_admin)
  },
})
