"use strict";
(() => {
  // src/features/code.ts
  var feedbackTimers = /* @__PURE__ */ new WeakMap();
  function showCopiedFeedback(btn) {
    btn.classList.add("copied");
    const existingTimer = feedbackTimers.get(btn);
    if (existingTimer !== void 0) {
      window.clearTimeout(existingTimer);
    }
    const timer = window.setTimeout(() => {
      btn.classList.remove("copied");
      feedbackTimers.delete(btn);
    }, 2e3);
    feedbackTimers.set(btn, timer);
  }
  function getVariableMap() {
    const map = {};
    const inputs = document.querySelectorAll("input.item-table-var-input, input[data-var-key]");
    inputs.forEach((input) => {
      const key = input.dataset.varKey || input.getAttribute("data-var-key");
      if (typeof key === "string" && key.trim() !== "") {
        map[key.trim()] = input.value;
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
  function updateAllCodeVariables() {
    const codeElements = document.querySelectorAll(".code-block code");
    codeElements.forEach((codeEl) => {
      if (!codeEl.hasAttribute("data-raw-code")) {
        codeEl.setAttribute("data-raw-code", codeEl.textContent ?? "");
      }
      const rawText = codeEl.getAttribute("data-raw-code") ?? "";
      const replacedText = replaceCodeVariables(rawText);
      if (codeEl.textContent !== replacedText) {
        codeEl.textContent = replacedText;
      }
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
    const text = codeEl.textContent ?? "";
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
  function setupVariableInputAutoSelect() {
    document.addEventListener("focusin", (e) => {
      const target = e.target;
      if (target instanceof HTMLInputElement && target.classList.contains("item-table-var-input")) {
        window.requestAnimationFrame(() => {
          target.select();
        });
      }
    });
  }
  function setupVariableInputListeners() {
    document.addEventListener("input", (e) => {
      const target = e.target;
      if (target instanceof HTMLInputElement && (target.classList.contains("item-table-var-input") || target.hasAttribute("data-var-key"))) {
        updateAllCodeVariables();
      }
    });
  }
  if (typeof window !== "undefined") {
    if (document.readyState === "loading") {
      window.addEventListener("DOMContentLoaded", updateAllCodeVariables);
    } else {
      updateAllCodeVariables();
    }
    setupVariableInputAutoSelect();
    setupVariableInputListeners();
    window.d2f_code = {
      copy: copyCode
    };
  }
})();
