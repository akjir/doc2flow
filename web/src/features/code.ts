// WeakMap stores the active timer per button to prevent race conditions
const feedbackTimers = new WeakMap<HTMLElement, number>();

function showCopiedFeedback(btn: HTMLElement): void {
    btn.classList.add('copied');

    // If a timer is already running for this button, clear it
    if (feedbackTimers.has(btn)) {
        window.clearTimeout(feedbackTimers.get(btn));
    }

    const timer = window.setTimeout(() => {
        btn.classList.remove('copied');
        feedbackTimers.delete(btn);
    }, 2000);

    feedbackTimers.set(btn, timer);
}

function getVariableMap(): Record<string, string> {
    const map: Record<string, string> = {};

    const elements = document.querySelectorAll<HTMLElement>('.item-table-var, .var-table, [data-variables]');
    elements.forEach((el) => {
        const rawJson = el.dataset.variables;
        if (typeof rawJson === 'string' && rawJson.length > 0) {
            try {
                const parsed: unknown = JSON.parse(rawJson);
                if (typeof parsed === 'object' && parsed !== null) {
                    const entries = Object.entries(parsed);
                    for (const entry of entries) {
                        const k = entry[0];
                        const v = entry[1];
                        if (typeof k === 'string' && typeof v === 'string') {
                            const trimmedKey = k.trim();
                            if (trimmedKey !== '') {
                                map[trimmedKey] = v;
                            }
                        }
                    }
                }
            } catch {
                // Ignore invalid JSON
            }
        }
    });

    // Live inputs override default values
    const inputs = document.querySelectorAll<HTMLInputElement>('input.item-table-var-input, input[data-var-key]');
    inputs.forEach((input) => {
        const key = input.dataset.varKey || input.getAttribute('data-var-key');
        if (typeof key === 'string' && key.trim() !== '') {
            map[key.trim()] = input.value;
        }
    });

    return map;
}


function replaceCodeVariables(text: string): string {
    const varMap = getVariableMap();
    if (Object.keys(varMap).length === 0) {
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

function fallbackCopyText(text: string, btn: HTMLElement): boolean {
    const ta = document.createElement('textarea');
    ta.value = text;
    // Move out of viewport instead of opacity: 0 (more robust on mobile devices)
    ta.style.position = 'fixed';
    ta.style.left = '-9999px';
    ta.style.top = '0';
    ta.setAttribute('readonly', ''); // Prevents the mobile keyboard from popping up

    document.body.appendChild(ta);
    ta.select();
    ta.setSelectionRange(0, text.length);

    let success = false;
    try {
        success = document.execCommand('copy');
        if (success) {
            showCopiedFeedback(btn);
        }
    } catch (e) {
        console.error('Fallback copy failed', e);
    } finally {
        document.body.removeChild(ta);
    }

    return success;
}

async function copyCode(btn: HTMLElement | null): Promise<void> {
    if (!btn) return;

    const wrap = btn.closest('.code-block-wrap');
    if (!wrap) return;

    const codeEl = wrap.querySelector('code');
    if (!codeEl) return;

    const rawText = codeEl.textContent ?? '';
    const text = replaceCodeVariables(rawText);

    // Use modern Async Clipboard API when available
    if (navigator.clipboard && window.isSecureContext) {
        try {
            await navigator.clipboard.writeText(text);
            showCopiedFeedback(btn);
            return;
        } catch (err) {
            console.warn('Clipboard API failed, falling back to execCommand:', err);
        }
    }

    // Execute fallback if Clipboard API is unavailable or fails
    fallbackCopyText(text, btn);
}

const printRawCodeMap = new Map<HTMLElement, string>();

function preparePrintVariables(): void {
    const codeElements = document.querySelectorAll<HTMLElement>('.code-block code');
    codeElements.forEach((codeEl) => {
        const rawText = codeEl.textContent ?? '';
        printRawCodeMap.set(codeEl, rawText);
        const replacedText = replaceCodeVariables(rawText);
        if (replacedText !== rawText) {
            codeEl.textContent = replacedText;
        }
    });
}

function restorePrintVariables(): void {
    printRawCodeMap.forEach((rawText, codeEl) => {
        codeEl.textContent = rawText;
    });
    printRawCodeMap.clear();
}

function checkCodeVariableWarnings(): void {
    const codeVars = new Set<string>();
    document.querySelectorAll<HTMLElement>('.code-block code').forEach((codeEl) => {
        const text = codeEl.textContent ?? '';
        const matches = text.matchAll(/\{\{([A-Za-z0-9_]+)\}\}/g);
        for (const match of matches) {
            if (match[1]) {
                codeVars.add(match[1]);
            }
        }
    });

    const tableVars = new Set<string>();
    document.querySelectorAll<HTMLInputElement>('input.item-table-var-input, input[data-var-key]').forEach((input) => {
        const key = input.dataset.varKey || input.getAttribute('data-var-key');
        if (key && key.trim() !== '') {
            tableVars.add(key.trim());
        }
    });

    tableVars.forEach((tv) => {
        if (!codeVars.has(tv)) {
            console.warn(`Doc2Flow: Variable "${tv}" in [Variables] table is not used in any code block.`);
        }
    });

    codeVars.forEach((cv) => {
        if (!tableVars.has(cv)) {
            console.warn(`Doc2Flow: Variable "${cv}" in code block is missing from [Variables] table.`);
        }
    });
}

if (typeof window !== 'undefined') {
    window.copyCode = copyCode;
    window.addEventListener('beforeprint', preparePrintVariables);
    window.addEventListener('afterprint', restorePrintVariables);
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', checkCodeVariableWarnings);
    } else {
        checkCodeVariableWarnings();
    }
}