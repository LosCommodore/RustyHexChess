<template>
  <main class="not-found">
    <p class="code">404</p>
    <p class="message">Nothing here.</p>
    <router-link to="/" class="home-link">Back to the board</router-link>
  </main>
</template>

<style scoped>
.not-found {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 8px;
  background: #f5f5f5;
  color: #333;
}

.code {
  margin: 0;
  font-size: 72px;
  font-weight: 700;
}

.message {
  margin: 0;
  color: #888;
}

.home-link {
  margin-top: 16px;
  color: #2e6e35;
}
</style>
