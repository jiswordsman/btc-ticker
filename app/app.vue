<template>
  <UApp class="dark">
    <div class="ticker-shell">
      <NuxtPage />
    </div>
  </UApp>
</template>

<style scoped>
.ticker-shell {
  height: 100dvh;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 24px;
  background:
    radial-gradient(circle at top, rgba(245, 158, 11, 0.16), transparent 40%),
    linear-gradient(180deg, rgba(18, 18, 20, 0.96), rgba(9, 9, 11, 0.98));
  color: rgba(255, 255, 255, 0.92);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.05),
    0 20px 40px rgba(0, 0, 0, 0.32);
  backdrop-filter: blur(24px) saturate(140%);
  -webkit-backdrop-filter: blur(24px) saturate(140%);
}
</style>
