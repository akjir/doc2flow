(() => {
  // ../src/features/code/code.ts
  (() => {
    const feedbackTimers = /* @__PURE__ */ new WeakMap();
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
      const inputs = document.querySelectorAll(
        "input.item-table-var-input, input[data-var-key]"
      );
      inputs.forEach((input) => {
        const key = input.dataset["varKey"] ?? input.getAttribute("data-var-key");
        if (typeof key === "string" && key.trim() !== "") {
          map[key.trim()] = input.value;
        }
      });
      return map;
    }
    function replaceCodeVariables(text) {
      const varMap = getVariableMap();
      if (window.Object.keys(varMap).length === 0) {
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
      const codeElements = document.querySelectorAll(".code-block code, pre code");
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
      try {
        const success = document.execCommand("copy");
        if (success) {
          showCopiedFeedback(btn);
        }
      } catch {
      } finally {
        document.body.removeChild(ta);
      }
    }
    async function copyCode(btn) {
      if (!btn)
        return;
      const wrap = btn.closest(".code-block-wrap") ?? btn.parentElement;
      if (!wrap)
        return;
      const codeEl = wrap.querySelector("code");
      if (!codeEl)
        return;
      const text = codeEl.textContent ?? "";
      if (navigator.clipboard && typeof navigator.clipboard.writeText === "function") {
        try {
          await navigator.clipboard.writeText(text);
          showCopiedFeedback(btn);
          return;
        } catch {
          fallbackCopyText(text, btn);
          return;
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
    function initCopyButtons() {
      const codeElements = document.querySelectorAll("pre code");
      codeElements.forEach((codeEl) => {
        const pre = codeEl.parentElement;
        if (!pre || pre.querySelector(".copy-btn"))
          return;
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "copy-btn";
        btn.setAttribute("aria-label", "Copy code");
        btn.addEventListener("click", () => {
          void copyCode(btn);
        });
        pre.style.position = "relative";
        pre.appendChild(btn);
      });
    }
    window.d2f_code = {
      copy: copyCode,
      updateVariables: updateAllCodeVariables
    };
    function init() {
      updateAllCodeVariables();
      initCopyButtons();
      setupVariableInputAutoSelect();
      setupVariableInputListeners();
    }
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", init);
    } else {
      init();
    }
  })();
})();
