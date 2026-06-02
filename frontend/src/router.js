import { createRouter, createWebHistory } from 'vue-router'
import { auth } from './store'

const routes = [
  { path: '/login', name: 'login', component: () => import('./views/Login.vue') },
  { path: '/register', name: 'register', component: () => import('./views/Register.vue') },
  { path: '/', name: 'home', component: () => import('./views/Home.vue'), meta: { auth: true } },
  { path: '/admin', name: 'admin', component: () => import('./views/Admin.vue'), meta: { auth: true, admin: true } },
  { path: '/p/:slug', name: 'preview', component: () => import('./views/Preview.vue') },
  { path: '/:pathMatch(.*)*', redirect: '/' },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach((to) => {
  if (to.meta.auth && !auth.isLoggedIn) {
    return { name: 'login', query: { redirect: to.fullPath } }
  }
  if (to.meta.admin && !auth.isAdmin) {
    return { name: 'home' }
  }
  if ((to.name === 'login' || to.name === 'register') && auth.isLoggedIn) {
    return { name: 'home' }
  }
})

export default router
