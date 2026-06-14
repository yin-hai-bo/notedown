<template>
  <main class="app-shell">
    <section class="document-workbench" aria-label="文档编辑区">
      <div class="tab-strip" role="tablist" aria-label="打开的文档">
        <div
          v-for="document in documentState.documents"
          :key="document.id"
          class="tab-item"
          :class="{ active: document.id === documentState.activeDocumentId }"
        >
          <button
            class="document-tab"
            type="button"
            role="tab"
            :aria-selected="document.id === documentState.activeDocumentId"
            @click="setActiveDocument(document.id)"
          >
            <span class="dirty-indicator" aria-hidden="true">{{ isDirty(document) ? "*" : "" }}</span>
            <span class="tab-title">{{ document.title }}</span>
          </button>
          <button
            class="close-tab"
            type="button"
            aria-label="关闭文档"
            title="关闭"
            @click="closeDocument(document)"
          >
            ×
          </button>
        </div>
        <button class="new-tab" type="button" aria-label="新建文档" title="新建" @click="createUntitledDocument">
          +
        </button>
      </div>

      <textarea
        v-if="activeDocument"
        v-model="activeDocument.content"
        class="source-editor"
        spellcheck="false"
        :aria-label="`${activeDocument.title} Markdown 源码`"
      ></textarea>

      <div v-else class="empty-state">
        <button type="button" @click="createUntitledDocument">新建文档</button>
        <button type="button" @click="openDocument">打开文档</button>
      </div>
    </section>
  </main>
</template>

<script setup lang="ts">
import {
  activeDocument,
  closeDocument,
  createUntitledDocument,
  documentState,
  isDirty,
  openDocument,
  setActiveDocument,
} from "./documents";
</script>

<style>
:root {
  color-scheme: light dark;
  --app-background: #ffffff;
  --app-foreground: #111111;
  --app-muted-foreground: #666666;
  --app-border: #d8d8d8;
  --app-tab-background: #eeeeee;
  --app-tab-active-background: #ffffff;
  --app-button-background: #f4f4f4;
  --app-editor-background: #ffffff;
  --app-editor-foreground: #1f1f1f;
  --app-focus: #2563eb;
}

:root[data-theme="system"] {
  color-scheme: light dark;
}

@media (prefers-color-scheme: dark) {
  :root,
  :root[data-theme="system"] {
    --app-background: #111111;
    --app-foreground: #f5f5f5;
    --app-muted-foreground: #a3a3a3;
    --app-border: #3a3a3a;
    --app-tab-background: #242424;
    --app-tab-active-background: #111111;
    --app-button-background: #202020;
    --app-editor-background: #111111;
    --app-editor-foreground: #f5f5f5;
    --app-focus: #60a5fa;
  }
}

:root[data-theme="light"] {
  color-scheme: light;
  --app-background: #ffffff;
  --app-foreground: #111111;
  --app-muted-foreground: #666666;
  --app-border: #d8d8d8;
  --app-tab-background: #eeeeee;
  --app-tab-active-background: #ffffff;
  --app-button-background: #f4f4f4;
  --app-editor-background: #ffffff;
  --app-editor-foreground: #1f1f1f;
  --app-focus: #2563eb;
}

:root[data-theme="dark"] {
  color-scheme: dark;
  --app-background: #111111;
  --app-foreground: #f5f5f5;
  --app-muted-foreground: #a3a3a3;
  --app-border: #3a3a3a;
  --app-tab-background: #242424;
  --app-tab-active-background: #111111;
  --app-button-background: #202020;
  --app-editor-background: #111111;
  --app-editor-foreground: #f5f5f5;
  --app-focus: #60a5fa;
}

html,
body,
#app {
  margin: 0;
  min-height: 100%;
  height: 100%;
}

body {
  background: var(--app-background);
  color: var(--app-foreground);
  font-family: "Segoe UI", system-ui, sans-serif;
}

.app-shell {
  height: 100vh;
  min-width: 0;
}

.document-workbench {
  display: grid;
  grid-template-rows: auto 1fr;
  height: 100%;
  min-width: 0;
}

.tab-strip {
  display: flex;
  align-items: end;
  min-width: 0;
  overflow-x: auto;
  border-bottom: 1px solid var(--app-border);
  background: var(--app-tab-background);
}

.tab-item,
.new-tab,
.empty-state button {
  border: 1px solid var(--app-border);
  background: var(--app-button-background);
  color: var(--app-foreground);
  font: inherit;
}

.tab-item {
  display: inline-grid;
  grid-template-columns: 10px minmax(48px, 180px) 24px;
  align-items: center;
  height: 34px;
  margin: 4px 0 0 4px;
  padding: 0 4px 0 8px;
  border-bottom: 0;
  border-radius: 6px 6px 0 0;
  cursor: default;
}

.tab-item.active {
  background: var(--app-tab-active-background);
}

.document-tab {
  display: contents;
  border: 0;
  background: transparent;
  color: inherit;
  font: inherit;
}

.dirty-indicator {
  color: var(--app-focus);
  font-weight: 700;
}

.tab-title {
  min-width: 0;
  overflow: hidden;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.close-tab,
.new-tab {
  display: inline-grid;
  place-items: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border-radius: 4px;
}

.close-tab {
  border: 0;
  background: transparent;
  color: var(--app-muted-foreground);
  font-size: 18px;
  line-height: 1;
}

.close-tab:hover,
.new-tab:hover,
.empty-state button:hover {
  border-color: var(--app-focus);
  color: var(--app-foreground);
}

.new-tab {
  margin: 4px 8px 5px;
  font-size: 18px;
  line-height: 1;
}

.source-editor {
  width: 100%;
  height: 100%;
  min-width: 0;
  padding: 16px;
  resize: none;
  border: 0;
  outline: none;
  box-sizing: border-box;
  background: var(--app-editor-background);
  color: var(--app-editor-foreground);
  font-family: Consolas, "Cascadia Mono", "Courier New", monospace;
  font-size: 14px;
  line-height: 1.5;
  tab-size: 2;
}

.source-editor:focus {
  box-shadow: inset 0 0 0 2px var(--app-focus);
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  min-height: 0;
}

.empty-state button {
  padding: 6px 12px;
  border-radius: 6px;
}
</style>
