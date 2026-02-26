// SPS Privacy Scanner — Popup

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

function formatGrade(g) {
  if (g === "APlus") return "A+";
  return g || "?";
}

function formatDate(iso) {
  if (!iso) return "";
  const d = new Date(iso);
  return d.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

document.addEventListener("DOMContentLoaded", async () => {
  const states = {
    loading: document.getElementById("loading"),
    disabled: document.getElementById("disabled"),
    result: document.getElementById("result"),
    noscan: document.getElementById("noscan"),
    scanning: document.getElementById("scanning"),
    error: document.getElementById("error"),
  };

  function showState(name) {
    for (const [key, el] of Object.entries(states)) {
      el.classList.toggle("hidden", key !== name);
    }
  }

  // Get current tab
  let tab;
  try {
    const tabs = await chrome.tabs.query({ active: true, currentWindow: true });
    tab = tabs[0];
  } catch {
    showState("disabled");
    return;
  }

  if (!tab || !tab.url) {
    showState("disabled");
    return;
  }

  if (tab.incognito) {
    showState("disabled");
    return;
  }

  // Extract domain
  let domain;
  try {
    const url = new URL(tab.url);
    if (INTERNAL_PROTOCOLS.has(url.protocol)) {
      showState("disabled");
      return;
    }
    domain = url.hostname.replace(/^www\./, "");
  } catch {
    showState("disabled");
    return;
  }

  // Request cached status from background
  showState("loading");
  chrome.runtime.sendMessage({ type: "GET_STATUS", domain }, (resp) => {
    if (chrome.runtime.lastError) {
      showState("disabled");
      return;
    }
    if (resp && resp.data) {
      renderResult(resp.data);
    } else {
      showNoScan();
    }
  });

  function renderResult(data) {
    const grade = formatGrade(data.grade);
    document.getElementById("gradeText").textContent = grade;

    const circle = document.getElementById("gradeCircle");
    const color = GRADE_COLORS[grade] || GRADE_COLORS["?"];
    circle.style.borderColor = color;
    circle.style.color = color;

    document.getElementById("domain").textContent = data.domain || domain;
    const score = data.score ?? data.total_score;
    document.getElementById("scoreLine").textContent =
      `${score}/100 — Grade ${grade}`;
    document.getElementById("scannedAt").textContent = data.scanned_at
      ? `Scanned ${formatDate(data.scanned_at)}`
      : "";

    const d = encodeURIComponent(data.domain || domain);
    document.getElementById("detailLink").href =
      `https://seglamater.app/privacy/scan/${d}`;

    showState("result");
  }

  function showNoScan() {
    document.getElementById("noscanDomain").textContent = domain;
    showState("noscan");
  }

  function showError(msg) {
    document.getElementById("errorMsg").textContent = msg;
    showState("error");
  }

  function triggerScan() {
    showState("scanning");
    chrome.runtime.sendMessage({ type: "SCAN", domain }, (resp) => {
      if (chrome.runtime.lastError) {
        showError("Connection to extension failed");
        return;
      }
      if (resp && resp.error) {
        showError(resp.error);
      } else if (resp && resp.data) {
        renderResult({
          domain: resp.data.domain || domain,
          score: resp.data.total_score,
          grade: resp.data.grade,
          scanned_at: resp.data.scanned_at,
        });
      } else {
        showError("Scan returned no data");
      }
    });
  }

  document.getElementById("scanBtn").addEventListener("click", triggerScan);
  document.getElementById("noscanBtn").addEventListener("click", triggerScan);
  document.getElementById("retryBtn").addEventListener("click", triggerScan);
});
