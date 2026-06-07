<template>
  <main class="app-shell"></main>
</template>

<style>
:root {
  color-scheme: light dark;
  --app-background: #ffffff;
  --app-foreground: #111111;
}

@media (prefers-color-scheme: dark) {
  :root {
    --app-background: #111111;
    --app-foreground: #f5f5f5;
  }
}

html,
body,
#app {
  margin: 0;
  min-height: 100%;
}

body {
  background: var(--app-background);
  color: var(--app-foreground);
}

.app-shell {
  min-height: 100vh;
}
</style>
