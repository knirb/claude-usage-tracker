const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const sections = {
  fiveHour: {
    bar: document.getElementById("five-hour-bar"),
    pct: document.getElementById("five-hour-pct"),
    reset: document.getElementById("five-hour-reset"),
  },
  sevenDay: {
    bar: document.getElementById("seven-day-bar"),
    pct: document.getElementById("seven-day-pct"),
    reset: document.getElementById("seven-day-reset"),
  },
  sonnet: {
    bar: document.getElementById("sonnet-bar"),
    pct: document.getElementById("sonnet-pct"),
    reset: document.getElementById("sonnet-reset"),
  },
};

let currentData = null;

function getLevel(pct) {
  if (pct < 50) return "level-low";
  if (pct < 80) return "level-mid";
  return "level-high";
}

function formatCountdown(resetsAt) {
  if (!resetsAt) return "";
  const now = Date.now();
  const reset = new Date(resetsAt).getTime();
  const diff = reset - now;
  if (diff <= 0) return "Resetting soon…";
  const hours = Math.floor(diff / 3600000);
  const mins = Math.floor((diff % 3600000) / 60000);
  if (hours > 0) return `Resets in ${hours} hr ${mins} min`;
  return `Resets in ${mins} min`;
}

function renderBucket(section, bucket, emptyMsg) {
  if (!bucket) {
    section.pct.textContent = "0% used";
    section.bar.style.width = "0%";
    section.bar.className = "progress-fill level-low";
    section.reset.textContent = emptyMsg || "";
    return;
  }
  // utilization is already a percentage (e.g. 36 = 36%)
  const pct = Math.round(bucket.utilization);
  const level = getLevel(pct);
  section.pct.textContent = `${pct}% used`;
  section.bar.style.width = `${Math.min(pct, 100)}%`;
  section.bar.className = `progress-fill ${level}`;
  section.reset.textContent = formatCountdown(bucket.resetsAt);
}

function renderUsage(data) {
  currentData = data;
  document.getElementById("loading").classList.add("hidden");
  document.getElementById("error").classList.add("hidden");
  document.getElementById("usage-sections").classList.remove("hidden");

  renderBucket(sections.fiveHour, data.fiveHour, "No session data");
  renderBucket(sections.sevenDay, data.sevenDay, "No weekly data");
  renderBucket(sections.sonnet, data.sevenDayOpus, "You haven't used Sonnet yet");

  if (data.fetchedAt) {
    const d = new Date(data.fetchedAt);
    const mins = Math.max(0, Math.round((Date.now() - d.getTime()) / 60000));
    document.getElementById("fetched-at").textContent =
      mins < 1 ? "Last updated: just now" : `Last updated: ${mins} min ago`;
  }
}

function showError(msg) {
  document.getElementById("loading").classList.add("hidden");
  const el = document.getElementById("error");
  el.textContent = msg;
  el.classList.remove("hidden");
}

async function fetchUsage() {
  const btn = document.getElementById("refresh-btn");
  btn.classList.add("spinning");
  try {
    const data = await invoke("get_usage");
    renderUsage(data);
  } catch (e) {
    showError(String(e));
  } finally {
    btn.classList.remove("spinning");
  }
}

// Initial load: try cache first, then fetch fresh
async function init() {
  try {
    const cached = await invoke("get_cached_usage");
    if (cached) renderUsage(cached);
  } catch (_) {}
  fetchUsage();
}

// Listen for backend polling updates
listen("usage-updated", (event) => {
  renderUsage(event.payload);
});

// Refresh button
document.getElementById("refresh-btn").addEventListener("click", fetchUsage);

// Update countdowns and "last updated" every 30s
setInterval(() => {
  if (currentData) renderUsage(currentData);
}, 30000);

init();
