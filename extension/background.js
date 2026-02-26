// SPS Privacy Scanner — Background Service Worker

const API_BASE = "https://seglamater.app/api/privacy";
const CACHE_TTL_MS = 24 * 60 * 60 * 1000; // 24 hours
const SCAN_COOLDOWN_MS = 20_000; // 20 seconds between scans per domain

const GRADE_COLORS = {
  "A+": "#44cc11",
  "A":  "#97ca00",
  "B":  "#007ec6",
  "C":  "#dfb317",
  "D":  "#fe7d37",
  "F":  "#e05d44",
  "?":  "#9f9f9f",
};

const INTERNAL_PROTOCOLS = new Set([
  "chrome:", "chrome-extension:", "about:", "edge:", "brave:",
  "moz-extension:", "file:", "devtools:", "data:", "blob:",
]);

// In-memory dedup for concurrent verify calls
const pendingVerifications = new Map();
// Client-side scan cooldown
const scanCooldowns = new Map();

// --- Domain extraction ---

function extractDomain(url) {
  if (!url) return null;
  try {
    const parsed = new URL(url);
    if (INTERNAL_PROTOCOLS.has(parsed.protocol)) return null;
    return parsed.hostname.replace(/^www\./, "");
  } catch {
    return null;
  }
}

// --- Grade formatting ---

function formatGrade(grade) {
  if (grade === "APlus") return "A+";
  return grade || "?";
}

// --- Cache layer (chrome.storage.local) ---

async function getCached(domain) {
  const key = `sps:${domain}`;
  const result = await chrome.storage.local.get(key);
  const entry = result[key];
  if (!entry) return null;
  if (Date.now() - entry.cached_at > CACHE_TTL_MS) return null;
  return entry;
}

async function setCache(domain, data) {
  const key = `sps:${domain}`;
  await chrome.storage.local.set({
    [key]: {
      domain: data.domain || domain,
      score: data.total_score,
      grade: formatGrade(data.grade),
      scanned_at: data.scanned_at,
      cached_at: Date.now(),
    },
  });
}

// --- Badge rendering ---

async function updateBadge(tabId, grade) {
  const text = grade || "?";
  const color = GRADE_COLORS[text] || GRADE_COLORS["?"];
  try {
    await chrome.action.setBadgeText({ text, tabId });
    await chrome.action.setBadgeBackgroundColor({ color, tabId });
  } catch {
    // Tab may have closed
  }
}

async function clearBadge(tabId) {
  try {
    await chrome.action.setBadgeText({ text: "", tabId });
  } catch {
    // Tab may have closed
  }
}

// --- API calls ---

async function verifyDomain(domain) {
  try {
    const resp = await fetch(
      `${API_BASE}/verify/${encodeURIComponent(domain)}`
    );
    if (resp.status === 404) return null;
    if (!resp.ok) return null;
    return await resp.json();
  } catch (e) {
    console.error("[SPS] Verify failed:", e);
    return null;
  }
}

async function scanDomain(domain) {
  const lastScan = scanCooldowns.get(domain);
  if (lastScan && Date.now() - lastScan < SCAN_COOLDOWN_MS) {
    throw new Error("Please wait before scanning again");
  }
  scanCooldowns.set(domain, Date.now());

  const resp = await fetch(`${API_BASE}/scan`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ domain }),
  });

  if (resp.status === 429) throw new Error("Rate limited — try again in a minute");
  if (resp.status === 503) throw new Error("Scanner service unavailable");
  if (!resp.ok) {
    const err = await resp.json().catch(() => ({}));
    throw new Error(err.detail || "Scan failed");
  }

  const result = await resp.json();
  await setCache(domain, result);
  return result;
}

// --- Core lookup (cache → dedup → verify) ---

async function lookupDomain(domain, tabId) {
  // 1. Check local cache
  const cached = await getCached(domain);
  if (cached) {
    await updateBadge(tabId, cached.grade);
    return cached;
  }

  // 2. Dedup: reuse in-flight verify if another tab already triggered one
  if (pendingVerifications.has(domain)) {
    const result = await pendingVerifications.get(domain);
    await updateBadge(tabId, result ? formatGrade(result.grade) : "?");
    return result;
  }

  // 3. Call verify API
  const promise = verifyDomain(domain);
  pendingVerifications.set(domain, promise);

  try {
    const result = await promise;
    if (result) {
      await setCache(domain, result);
      await updateBadge(tabId, formatGrade(result.grade));
    } else {
      await updateBadge(tabId, "?");
    }
    return result;
  } finally {
    pendingVerifications.delete(domain);
  }
}

// --- Tab event listeners ---

chrome.tabs.onActivated.addListener(async ({ tabId }) => {
  try {
    const tab = await chrome.tabs.get(tabId);
    if (tab.incognito) {
      clearBadge(tabId);
      return;
    }
    const domain = extractDomain(tab.url);
    if (!domain) {
      clearBadge(tabId);
      return;
    }
    await lookupDomain(domain, tabId);
  } catch {
    // Tab may have closed during lookup
  }
});

chrome.tabs.onUpdated.addListener(async (tabId, changeInfo, tab) => {
  if (changeInfo.status !== "complete") return;
  if (tab.incognito) {
    clearBadge(tabId);
    return;
  }
  const domain = extractDomain(tab.url);
  if (!domain) {
    clearBadge(tabId);
    return;
  }
  await lookupDomain(domain, tabId);
});

// --- Message handler (from popup) ---

chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (msg.type === "GET_STATUS") {
    (async () => {
      const cached = await getCached(msg.domain);
      sendResponse({ data: cached, domain: msg.domain });
    })();
    return true;
  }

  if (msg.type === "SCAN") {
    (async () => {
      try {
        const result = await scanDomain(msg.domain);
        const [tab] = await chrome.tabs.query({
          active: true,
          currentWindow: true,
        });
        if (tab) await updateBadge(tab.id, formatGrade(result.grade));
        sendResponse({ data: result });
      } catch (e) {
        sendResponse({ error: e.message });
      }
    })();
    return true;
  }

  if (msg.type === "REFRESH") {
    (async () => {
      try {
        const result = await verifyDomain(msg.domain);
        if (result) {
          await setCache(msg.domain, result);
          const [tab] = await chrome.tabs.query({
            active: true,
            currentWindow: true,
          });
          if (tab) await updateBadge(tab.id, formatGrade(result.grade));
        }
        sendResponse({ data: result });
      } catch (e) {
        sendResponse({ error: e.message });
      }
    })();
    return true;
  }
});
