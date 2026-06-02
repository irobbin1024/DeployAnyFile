import { reactive } from 'vue'

export const toastState = reactive({ msg: '', type: 'info', timer: null })

export function toast(msg, type = 'info') {
  toastState.msg = msg
  toastState.type = type
  if (toastState.timer) clearTimeout(toastState.timer)
  toastState.timer = setTimeout(() => {
    toastState.msg = ''
  }, 2600)
}
