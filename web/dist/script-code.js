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
  window.d2f.code = {
    showCopiedFeedback,
    fallbackCopyText,
    copyCode
  };
  if (typeof window !== "undefined") {
    window.copyCode = copyCode;
  }
})();
