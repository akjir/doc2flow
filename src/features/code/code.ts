interface D2FCodeFeature {
    readonly copy: (btn: HTMLElement | null) => Promise<void>;
    readonly updateVariables: () => void;
}

interface Window {
    d2f?: {
        code?: D2FCodeFeature;
    };
    d2f_code?: {
        readonly copy: (btn: HTMLElement | null) => Promise<void>;
    };
}

(() => {
    const feedbackTimers = new WeakMap<HTMLElement, number>();

    function showCopiedFeedback(btn: HTMLElement): void {
        btn.classList.add('copied');

        const existingTimer = feedbackTimers.get(btn);
        if (existingTimer !== undefined) {
            window.clearTimeout(existingTimer);
        }

        const timer = window.setTimeout(() => {
            btn.classList.remove('copied');
            feedbackTimers.delete(btn);
        }, 2000);

        feedbackTimers.set(btn, timer);
    }

    function getVariableMap(): Record<string, string> {
        const map: Record<string, string> = {};
        const inputs = document.querySelectorAll<HTMLInputElement>(
            'input.item-table-var-input, input[data-var-key]'
        );
        inputs.forEach((input) => {
            const key = input.dataset['varKey'] ?? input.getAttribute('data-var-key');
            if (typeof key === 'string' && key.trim() !== '') {
                map[key.trim()] = input.value;
            }
        });
        return map;
    }

    function replaceCodeVariables(text: string): string {
        const varMap = getVariableMap();
        if (window.Object.keys(varMap).length === 0) {
            return text;
        }

        return text.replace(/\{\{([A-Za-z0-9_]+)\}\}/g, (match: string, key: string): string => {
            const val = varMap[key];
            if (val !== undefined && val.trim() !== '') {
                return val;
            }
            return match;
        });
    }

    function updateAllCodeVariables(): void {
        const codeElements = document.querySelectorAll<HTMLElement>('.code-block code, pre code');
        codeElements.forEach((codeEl) => {
            if (!codeEl.hasAttribute('data-raw-code')) {
                codeEl.setAttribute('data-raw-code', codeEl.textContent ?? '');
            }
            const rawText = codeEl.getAttribute('data-raw-code') ?? '';
            const replacedText = replaceCodeVariables(rawText);
            if (codeEl.textContent !== replacedText) {
                codeEl.textContent = replacedText;
            }
        });
    }

    function fallbackCopyText(text: string, btn: HTMLElement): void {
        const ta = document.createElement('textarea');
        ta.value = text;
        ta.style.position = 'fixed';
        ta.style.left = '-9999px';
        ta.style.top = '0';
        ta.setAttribute('readonly', '');

        document.body.appendChild(ta);
        ta.select();
        ta.setSelectionRange(0, text.length);

        try {
            const success = document.execCommand('copy');
            if (success) {
                showCopiedFeedback(btn);
            }
        } catch {
            // Silently ignore clipboard failure
        } finally {
            document.body.removeChild(ta);
        }
    }

    async function copyCode(btn: HTMLElement | null): Promise<void> {
        if (!btn) return;

        const wrap = btn.closest('.code-block-wrap') ?? btn.parentElement;
        if (!wrap) return;

        const codeEl = wrap.querySelector('code');
        if (!codeEl) return;

        const text = codeEl.textContent ?? '';

        if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
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

    function setupVariableInputAutoSelect(): void {
        document.addEventListener('focusin', (e: Event) => {
            const target = e.target;
            if (target instanceof HTMLInputElement && target.classList.contains('item-table-var-input')) {
                window.requestAnimationFrame(() => {
                    target.select();
                });
            }
        });
    }

    function setupVariableInputListeners(): void {
        document.addEventListener('input', (e: Event) => {
            const target = e.target;
            if (
                target instanceof HTMLInputElement &&
                (target.classList.contains('item-table-var-input') || target.hasAttribute('data-var-key'))
            ) {
                updateAllCodeVariables();
            }
        });
    }

    function initCopyButtons(): void {
        const codeElements = document.querySelectorAll<HTMLElement>('pre code');
        codeElements.forEach((codeEl) => {
            const pre = codeEl.parentElement;
            if (!pre || pre.querySelector('.copy-btn')) return;

            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'copy-btn';
            btn.setAttribute('aria-label', 'Copy code');
            btn.addEventListener('click', () => {
                void copyCode(btn);
            });

            pre.style.position = 'relative';
            pre.appendChild(btn);
        });
    }

    const feature = {
        copy: copyCode,
        updateVariables: updateAllCodeVariables,
    } satisfies D2FCodeFeature;

    window.d2f = window.d2f ?? {};
    window.d2f.code = feature;
    window.d2f_code = {
        copy: copyCode,
    };

    function init(): void {
        updateAllCodeVariables();
        initCopyButtons();
        setupVariableInputAutoSelect();
        setupVariableInputListeners();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
