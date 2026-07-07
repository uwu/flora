<script setup lang="ts">
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  GridLines
} from '@flora-internal/design-system'
import { Check, ChevronsUpDown, LogOut, Monitor, Moon, Settings, Sun } from '@lucide/vue'
import logoCursiveSvg from '@uwu/flora-branding/logo-cursive.svg'
import logoSvg from '@uwu/flora-branding/logo.svg'
import { useColorMode } from '@vueuse/core'

type Theme = 'light' | 'dark' | 'auto'

const user = {
  name: 'Ari Flora',
  username: '@ariflora',
  image: 'https://api.dicebear.com/9.x/notionists/svg?seed=Ari%20Flora&backgroundColor=e0e7ff'
}

const colorMode = useColorMode({
  initialValue: 'auto',
  storageKey: 'flora-color-scheme',
  selector: 'html',
  attribute: 'class'
})
const themePreference = colorMode.store

const themes: Array<{
  value: Theme
  label: string
  icon: typeof Sun
}> = [
  { value: 'light', label: 'Light', icon: Sun },
  { value: 'dark', label: 'Dark', icon: Moon },
  { value: 'auto', label: 'System', icon: Monitor }
]

function setTheme(theme: Theme) {
  themePreference.value = theme
}

function navigateTo(path: string) {
  window.location.assign(path)
}
</script>

<template>
  <div class="page-root bg-gray-1 text-gray-12">
    <header class="app-nav">
      <GridLines mode="lines-with-dots" variant="navbar-lines" />
      <GridLines mode="lines-with-dots" variant="navbar-dots" />

      <div class="app-nav-inner">
        <a href="/" class="app-brand" aria-label="flora home">
          <img :src="logoSvg" alt="" class="app-logo-mark" />
          <img :src="logoCursiveSvg" alt="flora" class="app-logo-wordmark" />
        </a>
      </div>
    </header>

    <div class="page-frame">
      <GridLines mode="lines" />
      <GridLines mode="lines-with-dots" variant="tabbar-dots" />

      <main class="dashboard-layout" aria-label="Dashboard layout">
        <aside class="dashboard-sidebar-shell" aria-label="Sidebar">
          <div class="sidebar-content">
            <p class="layout-label">Sidebar</p>
          </div>

          <div class="sidebar-footer">
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <button class="user-menu-trigger flora-focus" type="button">
                  <img :src="user.image" alt="" class="user-avatar" />
                  <span class="user-copy">
                    <span class="user-name">{{ user.name }}</span>
                    <span class="user-handle">{{ user.username }}</span>
                  </span>
                  <ChevronsUpDown class="user-menu-chevron" aria-hidden="true" />
                </button>
              </DropdownMenuTrigger>

              <DropdownMenuContent
                side="top"
                align="start"
                :side-offset="4"
                class="user-menu-content"
              >
                <div class="user-menu-header">
                  <img :src="user.image" alt="" class="user-avatar" />
                  <span class="user-copy">
                    <span class="user-name">{{ user.name }}</span>
                    <span class="user-handle">{{ user.username }}</span>
                  </span>
                </div>

                <DropdownMenuSeparator />

                <DropdownMenuItem class="user-menu-item" @select="navigateTo('/settings')">
                  <Settings class="user-menu-icon" aria-hidden="true" />
                  <span>Settings</span>
                </DropdownMenuItem>

                <DropdownMenuSeparator />
                <DropdownMenuLabel class="user-menu-label">Theme</DropdownMenuLabel>

                <DropdownMenuItem
                  v-for="theme in themes"
                  :key="theme.value"
                  class="user-menu-item"
                  @select="setTheme(theme.value)"
                >
                  <component :is="theme.icon" class="user-menu-icon" aria-hidden="true" />
                  <span>{{ theme.label }}</span>
                  <Check
                    class="theme-check"
                    :class="{ 'theme-check-active': themePreference === theme.value }"
                    aria-hidden="true"
                  />
                </DropdownMenuItem>

                <DropdownMenuSeparator />

                <DropdownMenuItem class="user-menu-item" @select="navigateTo('/login')">
                  <LogOut class="user-menu-icon" aria-hidden="true" />
                  <span>Log out</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </aside>
        <section class="dashboard-main-shell" aria-label="Main content">
          <p class="layout-label">Dashboard</p>
        </section>
      </main>
    </div>
  </div>
</template>

<style scoped>
.page-root {
  --grid-max-width: var(--flora-grid-max-width, min(calc(100vw - 64px), 1376px));
  --grid-line-offset: var(--flora-grid-line-offset, 16px);
  --grid-line-color: var(--gray5);
  --grid-dot-color: var(--gray9);
  --grid-dot-fill: var(--gray1);
  --page-max: var(--flora-page-max, min(calc(100vw - 40px), 1040px));
  color-scheme: light dark;
  min-height: 100vh;
  overflow-x: hidden;
}

.page-root::selection {
  background: var(--irisA5);
  color: var(--gray12);
}

.page-root a {
  color: inherit;
  text-decoration: none;
}

.app-nav {
  --nav-fallback-grid-max-width: var(--flora-grid-max-width, min(calc(100vw - 64px), 1376px));
  --grid-max-width: var(--nav-fallback-grid-max-width);
  --grid-line-offset: var(--flora-grid-line-offset, 16px);
  --grid-line-color: var(--gray5);
  --grid-dot-color: var(--gray9);
  --grid-dot-fill: var(--gray1);
  --nav-grid-max-width: var(--grid-max-width);
  --nav-grid-line-offset: var(--grid-line-offset);
  --nav-edge-max: calc(var(--nav-grid-max-width) + var(--nav-grid-line-offset) * 2);
  position: sticky;
  top: 0;
  right: 0;
  left: 0;
  z-index: 100;
  height: 56px;
  background: var(--gray1);
  border-bottom: 1px solid var(--gray5);
  pointer-events: none;
}

.app-nav-inner {
  display: flex;
  align-items: center;
  height: 56px;
  max-width: var(--nav-edge-max);
  margin: 0 auto;
  padding-right: 32px;
  padding-left: 32px;
  pointer-events: auto;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-logo-mark {
  display: block;
  width: 28px;
  height: 30px;
}

.app-logo-wordmark {
  display: block;
  width: 68px;
  height: auto;
}

:global(.dark .app-logo-wordmark) {
  filter: invert(1);
}

.page-frame {
  position: relative;
  width: 100%;
  max-width: var(--grid-max-width);
  margin: 0 auto;
  overflow-y: clip;
}

.dashboard-layout {
  display: grid;
  grid-template-columns: minmax(220px, 280px) minmax(0, 1fr);
  min-height: calc(100vh - 56px);
}

.dashboard-sidebar-shell,
.dashboard-main-shell {
  min-height: calc(100vh - 56px);
}

.dashboard-sidebar-shell {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--gray5);
}

.sidebar-content {
  flex: 1;
}

.sidebar-footer {
  padding: 16px;
  border-top: 1px solid var(--gray5);
}

.user-menu-trigger {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  min-height: 48px;
  padding: 8px;
  color: var(--gray12);
  background: transparent;
  border: 0;
  border-radius: 8px;
  cursor: pointer;
  transition-property: background-color, scale;
  transition-duration: 150ms;
  transition-timing-function: ease-out;
}

.user-menu-trigger:hover,
.user-menu-trigger[data-state='open'] {
  background: var(--gray3);
}

.user-menu-trigger:active {
  scale: 0.96;
}

.user-avatar {
  display: block;
  flex: none;
  width: 32px;
  height: 32px;
  object-fit: cover;
  border-radius: 9999px;
  outline: 1px solid color-mix(in srgb, var(--gray12) 12%, transparent);
  outline-offset: -1px;
}

.user-copy {
  display: grid;
  flex: 1;
  min-width: 0;
  line-height: 1.25;
  text-align: left;
}

.user-name,
.user-handle {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.user-name {
  font-size: 14px;
  font-weight: 500;
}

.user-handle {
  color: var(--gray10);
  font-size: 12px;
}

.user-menu-chevron {
  width: 16px;
  height: 16px;
  margin-left: auto;
  color: var(--gray10);
}

:global(.user-menu-content) {
  width: 208px;
  padding: 4px;
  border-color: var(--gray6);
  border-radius: 6px;
  box-shadow: none;
}

:global(.user-menu-header) {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
}

:global(.user-menu-header .user-avatar) {
  width: 28px;
  height: 28px;
}

:global(.user-menu-item) {
  padding-top: 4px;
  padding-bottom: 4px;
  cursor: pointer;
}

:global(.user-menu-icon) {
  width: 14px;
  height: 14px;
  flex: none;
  color: var(--gray10);
  stroke-width: 1.75px;
}

:global(.user-menu-label) {
  padding-top: 4px;
  padding-bottom: 4px;
  color: var(--gray10);
  font-size: 12px;
}

:global(.user-menu-content [data-slot='dropdown-menu-separator']) {
  margin-top: 2px;
  margin-bottom: 2px;
}

:global(.theme-check) {
  width: 12px;
  height: 12px;
  margin-left: auto;
  color: var(--gray10);
  opacity: 0;
  filter: blur(4px);
  scale: 0.25;
  transition-property: opacity, filter, scale;
  transition-duration: 200ms;
  transition-timing-function: cubic-bezier(0.2, 0, 0, 1);
}

:global(.theme-check-active) {
  opacity: 1;
  filter: blur(0);
  scale: 1;
}

.dashboard-main-shell {
  max-width: var(--page-max);
  width: 100%;
}

.layout-label {
  margin: 0;
  padding: 24px;
  color: var(--gray10);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

@media (max-width: 600px) {
  .page-root {
    --grid-max-width: 100vw;
  }

  .app-nav-inner {
    padding-right: 20px;
    padding-left: 20px;
  }

  .dashboard-layout {
    grid-template-columns: 1fr;
  }

  .dashboard-sidebar-shell {
    min-height: 240px;
    border-right: 0;
    border-bottom: 1px solid var(--gray5);
  }
}

@media (prefers-reduced-motion: reduce) {
  .user-menu-trigger,
  :global(.theme-check) {
    transition-duration: 0ms;
  }
}
</style>
