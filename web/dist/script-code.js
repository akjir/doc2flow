"use strict";
(() => {
  // src/features/code.ts
  var feedbackTimers = /* @__PURE__ */ new WeakMap();
  function showCopiedFeedback(btn) {
    btn.classList.add("copied");
    if (feedbackTimers.has(btn)) {
      window.clearTimeout(feedbackTimers.get(btn));
    }
    const timer = window.setTimeout(() => {
      btn.classList.remove("copied");
      feedbackTimers.delete(btn);
    }, 2e3);
    feedbackTimers.set(btn, timer);
  }
  function getVariableMap() {
    const map = {};
    const elements = document.querySelectorAll(".item-table-var, .var-table, [data-variables]");
    elements.forEach((el) => {
      const rawJson = el.dataset.variables;
      if (typeof rawJson === "string" && rawJson.length > 0) {
        try {
          const parsed = JSON.parse(rawJson);
          if (typeof parsed === "object" && parsed !== null) {
            const entries = Object.entries(parsed);
            for (const entry of entries) {
              const k = entry[0];
              const v = entry[1];
              if (typeof k === "string" && typeof v === "string") {
                const trimmedKey = k.trim();
                if (trimmedKey !== "") {
                  map[trimmedKey] = v;
                }
              }
            }
          }
        } catch {
        }
      }
    });
    return map;
  }
  function replaceCodeVariables(text) {
    const varMap = getVariableMap();
    if (Object.keys(varMap).length === 0) {
      return text;
    }
    return text.replace(/\{\{([A-Za-z0-9_]+)\}\}/g, (match, key) => {
      const val = varMap[key];
      if (val !== void 0 && val.trim() !== "") {
        return val;
      }
      return match;
    });
  }
  function fallbackCopyText(text, btn) {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.left = "-9999px";
    ta.style.top = "0";
    ta.setAttribute("readonly", "");
    document.body.appendChild(ta);
    ta.select();
    ta.setSelectionRange(0, text.length);
    let success = false;
    try {
      success = document.execCommand("copy");
      if (success) {
        showCopiedFeedback(btn);
      }
    } catch (e) {
      console.error("Fallback copy failed", e);
    } finally {
      document.body.removeChild(ta);
    }
    return success;
  }
  async function copyCode(btn) {
    if (!btn)
      return;
    const wrap = btn.closest(".code-block-wrap");
    if (!wrap)
      return;
    const codeEl = wrap.querySelector("code");
    if (!codeEl)
      return;
    const rawText = codeEl.textContent ?? "";
    const text = replaceCodeVariables(rawText);
    if (navigator.clipboard && window.isSecureContext) {
      try {
        await navigator.clipboard.writeText(text);
        showCopiedFeedback(btn);
        return;
      } catch (err) {
        console.warn("Clipboard API failed, falling back to execCommand:", err);
      }
    }
    fallbackCopyText(text, btn);
  }
  if (typeof window !== "undefined") {
    window.copyCode = copyCode;
  }
})();
