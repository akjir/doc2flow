(() => {
  // ../src/legacy/features/fields/fields.ts
  (() => {
    function saveFields() {
      const fields = {};
      document.querySelectorAll("input.persistent-field").forEach((input, index) => {
        const key = input.id || "f_" + String(index);
        fields[key] = input.value;
      });
      return { fields };
    }
    function loadFields(state) {
      const fieldsData = state["fields"];
      if (window.d2f.utils.isRecord(fieldsData)) {
        document.querySelectorAll("input.persistent-field").forEach((input, index) => {
          const key = input.id || "f_" + String(index);
          const val = fieldsData[key];
          if (typeof val === "string") {
            input.value = val;
            input.setAttribute("value", val);
          }
        });
      }
      return false;
    }
    function resetFields() {
      document.querySelectorAll(
        "input, textarea"
      ).forEach((el) => {
        if (el.id === "search-input" || el.classList.contains("search-input"))
          return;
        if (el instanceof HTMLInputElement) {
          if (el.type === "checkbox" || el.type === "radio") {
            el.checked = false;
            el.removeAttribute("checked");
          } else {
            el.value = "";
            el.removeAttribute("value");
          }
        } else if (el instanceof HTMLTextAreaElement) {
          el.value = "";
          el.textContent = "";
          el.removeAttribute("value");
        }
      });
    }
    window.d2f.storage.registerSaveHandler(saveFields);
    window.d2f.storage.registerLoadHandler(loadFields);
    const saveStateDebounced = window.d2f.utils.debounce(() => window.d2f.storage.saveState(), 300);
    function handleInputOrChange(e) {
      const target = e.target;
      if (!(target instanceof HTMLInputElement))
        return;
      if (target.classList.contains("persistent-field")) {
        target.setAttribute("value", target.value);
        saveStateDebounced();
      }
    }
    function init() {
      window.d2f.core.registerResetHandler(resetFields);
      document.addEventListener("input", handleInputOrChange);
      document.addEventListener("change", handleInputOrChange);
    }
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", init);
    } else {
      init();
    }
  })();
})();
