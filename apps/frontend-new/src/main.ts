import { createApp } from 'vue'
import './style.css'
import '@flora-internal/design-system/style.css'
import 'virtual:uno.css'
import App from './App.vue'
import { installBrowserLogForwarding } from './lib/debug/browserLogs'

installBrowserLogForwarding()
createApp(App).mount('#app')
